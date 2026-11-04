//! Prototype's built-in video recorder, the GPU side: while a recording
//! runs, each frame copies the presented image (or the 3D view before the
//! interface) into one of a few readback buffers, maps it without waiting,
//! and hands the pixels to the encoder thread a frame or two later. Nothing
//! waits on the GPU: when every buffer is still in flight, or the encoder is
//! behind, the frame is simply not recorded (the encoder repeats the last
//! one to keep its frame rate), so recording never slows the viewer down.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::SyncSender;
use std::time::Instant;

/// Buffers in flight at once: enough for the two frames the GPU runs
/// behind, plus one being read.
const SLOTS: usize = 4;

/// One recorded frame: tightly packed 4-byte pixels, top row first.
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// The pixels are BGRA (the usual swapchain order) rather than RGBA.
    pub bgra: bool,
    pub pixels: Vec<u8>,
    /// When the frame was drawn.
    pub at: Instant,
}

/// What the viewer asks of every frame while it records.
#[derive(Clone)]
pub struct Request {
    /// The 3D view alone (no menus, windows or name tags' interface).
    pub scene_only: bool,
    /// Where the frames go: full (the encoder is behind) drops the frame.
    pub sink: SyncSender<Frame>,
}

const PENDING: u8 = 0;
const MAPPED: u8 = 1;
const FAILED: u8 = 2;

enum State {
    Free,
    /// Copied in this frame's commands, mapped once they are submitted.
    Encoded(Instant),
    InFlight(Arc<AtomicU8>, Instant),
}

struct Slot {
    buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    row: u32,
    state: State,
}

#[derive(Default)]
pub(crate) struct Ring {
    slots: Vec<Slot>,
}

impl Ring {
    /// Hands over the frames whose copy has arrived, and frees their buffer.
    pub fn collect(&mut self, device: &wgpu::Device, sink: Option<&SyncSender<Frame>>, bgra: bool) {
        if self.slots.iter().any(|s| matches!(s.state, State::InFlight(..))) {
            let _ = device.poll(wgpu::PollType::Poll);
        }
        for slot in &mut self.slots {
            let State::InFlight(flag, at) = &slot.state else {
                continue;
            };
            match flag.load(Ordering::Acquire) {
                PENDING => continue,
                MAPPED => {
                    if let Some(sink) = sink
                        && let Ok(view) = slot.buffer.get_mapped_range(..)
                    {
                        let (w, h, row) = (slot.width as usize, slot.height as usize, slot.row as usize);
                        let mut pixels = Vec::with_capacity(w * h * 4);
                        for y in 0..h {
                            pixels.extend_from_slice(&view[y * row..y * row + w * 4]);
                        }
                        drop(view);
                        // full: the encoder is behind, this frame is skipped
                        let _ = sink.try_send(Frame {
                            width: slot.width,
                            height: slot.height,
                            bgra,
                            pixels,
                            at: *at,
                        });
                    }
                    slot.buffer.unmap();
                }
                _ => {}
            }
            slot.state = State::Free;
        }
        if sink.is_none() && self.slots.iter().all(|s| matches!(s.state, State::Free)) {
            // the recording ended: its memory goes back
            self.slots.clear();
        }
    }

    /// Copies `texture` (`width` × `height`) into a free buffer, when one is.
    pub fn encode(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, texture: &wgpu::Texture, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        let row = (width * 4).div_ceil(256) * 256;
        let index = match self.slots.iter().position(|s| matches!(s.state, State::Free) && s.width == width && s.height == height) {
            Some(i) => i,
            None => {
                // a resized window: the old buffers of the other size go
                self.slots
                    .retain(|s| !(matches!(s.state, State::Free) && (s.width != width || s.height != height)));
                if self.slots.len() >= SLOTS {
                    return;
                }
                self.slots.push(Slot {
                    buffer: device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("recording"),
                        size: u64::from(row) * u64::from(height),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                        mapped_at_creation: false,
                    }),
                    width,
                    height,
                    row,
                    state: State::Free,
                });
                self.slots.len() - 1
            }
        };
        let slot = &mut self.slots[index];
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &slot.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        slot.state = State::Encoded(Instant::now());
    }

    /// Once the frame's commands are submitted: the copies are mapped, read
    /// when they arrive (`collect`).
    pub fn after_submit(&mut self) {
        for slot in &mut self.slots {
            if let State::Encoded(at) = slot.state {
                let flag = Arc::new(AtomicU8::new(PENDING));
                let done = flag.clone();
                slot.buffer.map_async(wgpu::MapMode::Read, .., move |r| {
                    done.store(if r.is_ok() { MAPPED } else { FAILED }, Ordering::Release);
                });
                slot.state = State::InFlight(flag, at);
            }
        }
    }
}
