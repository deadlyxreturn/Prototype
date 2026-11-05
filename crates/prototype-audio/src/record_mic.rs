//! Prototype's recorder: the microphone, recorded beside the video. Its own
//! input stream (voice keeps its own), on its own thread (a cpal stream
//! stays on the thread that made it), mono samples at the device's rate.

use crate::devices::{self, Direction};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::time::Duration;

pub struct MicRecording {
    stop: Arc<AtomicBool>,
    /// Mono blocks at `rate`.
    pub samples: Receiver<Vec<f32>>,
    pub rate: u32,
}

impl Drop for MicRecording {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

fn build<T>(device: &cpal::Device, config: &cpal::StreamConfig, tx: SyncSender<Vec<f32>>) -> Option<cpal::Stream>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = usize::from(config.channels).max(1);
    device
        .build_input_stream::<T, _, _>(
            config.clone(),
            move |data: &[T], _| {
                let mono: Vec<f32> = data
                    .chunks(channels)
                    .map(|f| f.iter().map(|s| f32::from_sample(*s)).sum::<f32>() / channels as f32)
                    .collect();
                // the recorder behind: this block is dropped, never waited for
                let _ = tx.try_send(mono);
            },
            |e| log::warn!("recorder microphone: {e}"),
            None,
        )
        .ok()
}

/// Starts recording `name` (or the default microphone); None when there is
/// no microphone to open.
pub fn record_microphone(name: Option<String>) -> Option<MicRecording> {
    let (tx, rx) = sync_channel::<Vec<f32>>(512);
    let (ready_tx, ready_rx) = sync_channel::<Option<u32>>(1);
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    std::thread::Builder::new()
        .name("recorder-mic".into())
        .spawn(move || {
            let opened = (|| {
                let picked = devices::pick(Direction::Input, name.as_deref())?;
                let supported = picked.device.default_input_config().ok()?;
                let config = supported.config();
                let stream = match supported.sample_format() {
                    SampleFormat::F32 => build::<f32>(&picked.device, &config, tx),
                    SampleFormat::I16 => build::<i16>(&picked.device, &config, tx),
                    SampleFormat::U16 => build::<u16>(&picked.device, &config, tx),
                    SampleFormat::I32 => build::<i32>(&picked.device, &config, tx),
                    SampleFormat::F64 => build::<f64>(&picked.device, &config, tx),
                    _ => None,
                }?;
                stream.play().ok()?;
                Some((stream, config.sample_rate))
            })();
            match opened {
                Some((stream, rate)) => {
                    let _ = ready_tx.send(Some(rate));
                    while !stopped.load(Ordering::Acquire) {
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    drop(stream);
                }
                None => {
                    let _ = ready_tx.send(None);
                }
            }
        })
        .ok()?;
    let rate = ready_rx.recv_timeout(Duration::from_secs(3)).ok().flatten()?;
    Some(MicRecording { stop, samples: rx, rate })
}
