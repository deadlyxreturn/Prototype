//! Prototype's built-in video recorder, the GPU side: while a recording
//! runs, each frame copies the presented image (and/or the 3D view before
//! the interface) into one of a few readback buffers, maps it without
//! waiting, and hands the pixels to the encoder thread a frame or two
//! later. Nothing waits on the GPU: when every buffer is still in flight, or
//! the encoder is behind, the frame is simply not recorded (the encoder
//! repeats the last one to keep its frame rate), so recording never slows
//! the viewer down.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::SyncSender;
use std::time::Instant;

/// Buffers in flight at once: enough for the two frames the GPU runs
/// behind, plus one being read.
const SLOTS: usize = 4;

/// What a video shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The 3D view alone.
    World,
    /// The 3D view with the bars, the windows left out (`Request::holes`
    /// are filled with the 3D view behind them).
    Bars,
    /// The whole screen as it is.
    Everything,
}

/// One recorded frame: tightly packed 4-byte pixels, top row first.
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// The pixels are BGRA (the usual swapchain order) rather than RGBA.
    pub bgra: bool,
    pub pixels: Vec<u8>,
    /// `Mode::Bars`: the 3D view of the same frame, for the holes.
    pub scene: Option<Vec<u8>>,
    /// Rectangles (x0, y0, x1, y1 in pixels) to take from `scene`.
    pub holes: Vec<[u32; 4]>,
    /// The mouse pointer, in pixels, when it is drawn into the video.
    pub cursor: Option<[f32; 2]>,
    /// When the frame was drawn.
    pub at: Instant,
}

/// What the viewer asks of every frame while it records.
#[derive(Clone)]
pub struct Request {
    pub mode: Mode,
    pub holes: Vec<[u32; 4]>,
    pub cursor: Option<[f32; 2]>,
    /// Where the frames go: full (the encoder is behind) drops the frame.
    pub sink: SyncSender<Frame>,
}

impl Request {
    /// Whether the 3D view is copied, and whether the whole screen is.
    pub fn wants(&self) -> (bool, bool) {
        match self.mode {
            Mode::World => (true, false),
            Mode::Bars => (true, true),
            Mode::Everything => (false, true),
        }
    }
}

const PENDING: u8 = 0;
const MAPPED: u8 = 1;

#[derive(PartialEq)]
enum State {
    Free,
    /// Copied in this frame's commands, mapped once they are submitted.
    Encoded,
    InFlight,
}

struct Slot {
    /// The image the video shows (the scene in `Mode::World`, the screen
    /// otherwise).
    main: wgpu::Buffer,
    /// `Mode::Bars`: the scene, for the windows' holes.
    scene: Option<wgpu::Buffer>,
    width: u32,
    height: u32,
    row: u32,
    state: State,
    flags: Vec<Arc<AtomicU8>>,
    holes: Vec<[u32; 4]>,
    cursor: Option<[f32; 2]>,
    at: Instant,
}

#[derive(Default)]
pub(crate) struct Ring {
    slots: Vec<Slot>,
    /// The slot this frame copies into.
    current: Option<usize>,
}

fn read(buffer: &wgpu::Buffer, w: usize, h: usize, row: usize) -> Option<Vec<u8>> {
    let view = buffer.get_mapped_range(..).ok()?;
    let mut pixels = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        pixels.extend_from_slice(&view[y * row..y * row + w * 4]);
    }
    Some(pixels)
}

impl Ring {
    /// Hands over the frames whose copies have arrived, and frees their
    /// buffers.
    pub fn collect(&mut self, device: &wgpu::Device, sink: Option<&SyncSender<Frame>>, bgra: bool) {
        self.current = None;
        if self.slots.iter().any(|s| s.state == State::InFlight) {
            let _ = device.poll(wgpu::PollType::Poll);
        }
        for slot in &mut self.slots {
            if slot.state != State::InFlight {
                continue;
            }
            let states: Vec<u8> = slot.flags.iter().map(|f| f.load(Ordering::Acquire)).collect();
            if states.contains(&PENDING) {
                continue;
            }
            if states.iter().all(|s| *s == MAPPED)
                && let Some(sink) = sink
            {
                let (w, h, row) = (slot.width as usize, slot.height as usize, slot.row as usize);
                let main = read(&slot.main, w, h, row);
                let scene = slot.scene.as_ref().and_then(|b| read(b, w, h, row));
                if let Some(pixels) = main {
                    // full: the encoder is behind, this frame is skipped
                    let _ = sink.try_send(Frame {
                        width: slot.width,
                        height: slot.height,
                        bgra,
                        pixels,
                        scene,
                        holes: std::mem::take(&mut slot.holes),
                        cursor: slot.cursor,
                        at: slot.at,
                    });
                }
            }
            for (buffer, state) in std::iter::once(&slot.main).chain(slot.scene.as_ref()).zip(&states) {
                if *state == MAPPED {
                    buffer.unmap();
                }
            }
            slot.flags.clear();
            slot.state = State::Free;
        }
        if sink.is_none() && self.slots.iter().all(|s| s.state == State::Free) {
            // the recording ended: its memory goes back
            self.slots.clear();
        }
    }

    /// The slot of this frame: a free one of this size (made when needed),
    /// none when all are in flight.
    fn slot(&mut self, device: &wgpu::Device, request: &Request, width: u32, height: u32) -> Option<usize> {
        if let Some(i) = self.current {
            return Some(i);
        }
        let row = (width * 4).div_ceil(256) * 256;
        let needs_scene = request.mode == Mode::Bars;
        let fits = |s: &Slot| s.state == State::Free && s.width == width && s.height == height && s.scene.is_some() == needs_scene;
        let index = match self.slots.iter().position(fits) {
            Some(i) => i,
            None => {
                // a resized window or another mode: the free buffers that
                // no longer fit go
                self.slots.retain(|s| s.state != State::Free || fits(s));
                if self.slots.len() >= SLOTS {
                    return None;
                }
                let buffer = || {
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("recording"),
                        size: u64::from(row) * u64::from(height),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                        mapped_at_creation: false,
                    })
                };
                self.slots.push(Slot {
                    main: buffer(),
                    scene: needs_scene.then(buffer),
                    width,
                    height,
                    row,
                    state: State::Free,
                    flags: Vec::new(),
                    holes: Vec::new(),
                    cursor: None,
                    at: Instant::now(),
                });
                self.slots.len() - 1
            }
        };
        let slot = &mut self.slots[index];
        slot.state = State::Encoded;
        slot.holes = request.holes.clone();
        slot.cursor = request.cursor;
        slot.at = Instant::now();
        self.current = Some(index);
        Some(index)
    }

    /// Copies the 3D view (`scene`) or the screen into this frame's slot.
    #[allow(clippy::too_many_arguments)]
    pub fn encode(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        request: &Request,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        scene: bool,
    ) {
        if width == 0 || height == 0 {
            return;
        }
        let Some(index) = self.slot(device, request, width, height) else {
            return;
        };
        let slot = &self.slots[index];
        let buffer = match (request.mode, scene) {
            (Mode::Bars, true) => slot.scene.as_ref(),
            _ => Some(&slot.main),
        };
        let Some(buffer) = buffer else { return };
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(slot.row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Once the frame's commands are submitted: the copies are mapped, read
    /// when they arrive (`collect`).
    pub fn after_submit(&mut self) {
        for slot in &mut self.slots {
            if slot.state != State::Encoded {
                continue;
            }
            for buffer in std::iter::once(&slot.main).chain(slot.scene.as_ref()) {
                let flag = Arc::new(AtomicU8::new(PENDING));
                let done = flag.clone();
                buffer.map_async(wgpu::MapMode::Read, .., move |r| {
                    done.store(if r.is_ok() { MAPPED } else { 2 }, Ordering::Release);
                });
                slot.flags.push(flag);
            }
            slot.state = State::InFlight;
        }
        self.current = None;
    }
}
