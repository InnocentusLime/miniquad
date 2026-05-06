use std::sync::mpsc::{Receiver, Sender, channel};

use cpal::Sample;
use cpal::traits::DeviceTrait;
use glam::*;

pub type BufferID = usize;
pub type BufferData = Box<[I16Vec2]>;
pub type PlaybackID = usize;

const EQUILIBRIUM: Vec2 = vec2(f32::EQUILIBRIUM, f32::EQUILIBRIUM);

pub fn start_backend(
    config: &cpal::StreamConfig,
    device: &cpal::Device,
    buffers_capacity: usize,
    playback_capacity: usize,
) -> (Sender<AudioCommand>, cpal::Stream) {
    let err_fn = |err| tracing::error!("an error occurred on stream: {err}");
    let (cmd, mut backend) = AudioBackend::new(buffers_capacity, playback_capacity);
    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                backend.process_commands();
                backend.write_samples_and_advance_playbacks(data);
            },
            err_fn,
            None,
        )
        .expect("failed to spawn a device stream");
    (cmd, stream)
}

pub enum AudioCommand {
    /// Uploads audio data to the specified slot.
    SetAudioData { buffer_id: BufferID, data: BufferData },
    /// Starts a new playback at the specified slot of a specified sound.
    /// The sound will be played right away.
    StartPlayback { playback_id: PlaybackID, src: BufferID, looping: bool, volume: f32 },
    /// Toggles whether a playback is paused or not.
    SetPlaybackPause { playback_id: PlaybackID, pause: bool },
    /// Restarts the playback
    RestartPlayback { playback_id: PlaybackID },
}

struct AudioBackend {
    cmds: Receiver<AudioCommand>,
    buffers: Vec<BufferData>,
    playbacks: Vec<SoundPlayback>,
}

impl AudioBackend {
    fn new(buffers_capacity: usize, playback_capacity: usize) -> (Sender<AudioCommand>, Self) {
        let (snd, rcv) = channel();
        (
            snd,
            AudioBackend {
                cmds: rcv,
                buffers: vec![Box::new([]); buffers_capacity],
                playbacks: vec![SoundPlayback::default(); playback_capacity],
            },
        )
    }

    fn write_samples_and_advance_playbacks(&mut self, data: &mut [f32]) {
        for chunk in data.chunks_exact_mut(2) {
            let sample = self
                .playbacks
                .iter()
                .map(|playback| {
                    let Some(data) = self.buffers.get(playback.src) else {
                        return EQUILIBRIUM;
                    };
                    playback.get_sample(data)
                })
                .sum::<Vec2>();

            chunk[0] = sample.x;
            chunk[1] = sample.y;

            self.playbacks.iter_mut().for_each(|playback| {
                let Some(data) = self.buffers.get(playback.src) else {
                    return;
                };
                playback.advance(data);
            })
        }
    }

    fn process_commands(&mut self) {
        // NOTE: this is just a rought prototype. This implementation
        //       can risk getting too busy.
        for cmd in self.cmds.try_iter() {
            match cmd {
                AudioCommand::SetAudioData { buffer_id, data: new_data } => {
                    let Some(data) = self.buffers.get_mut(buffer_id) else {
                        continue;
                    };
                    *data = new_data;
                }
                AudioCommand::StartPlayback { playback_id, src, looping, volume } => {
                    let Some(playback) = self.playbacks.get_mut(playback_id) else {
                        continue;
                    };
                    *playback =
                        SoundPlayback { src, volume, looping, pause: false, current_sample: 0 };
                }
                AudioCommand::SetPlaybackPause { playback_id, pause } => {
                    let Some(playback) = self.playbacks.get_mut(playback_id) else {
                        continue;
                    };
                    playback.pause = pause;
                }
                AudioCommand::RestartPlayback { playback_id } => {
                    let Some(playback) = self.playbacks.get_mut(playback_id) else {
                        continue;
                    };
                    playback.current_sample = 0;
                    playback.pause = false;
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct SoundPlayback {
    src: BufferID,
    // volume from the interval [0; 1]
    volume: f32,
    looping: bool,
    pause: bool,

    current_sample: usize,
}

impl SoundPlayback {
    fn advance(&mut self, data: &BufferData) {
        if self.pause {
            return;
        }

        let next_sample = self.current_sample.wrapping_add(1);
        if next_sample < data.len() {
            self.current_sample = next_sample;
        } else if self.looping {
            self.current_sample = 0
        } else {
            self.pause = true;
        }
    }

    fn get_sample(&self, data: &BufferData) -> Vec2 {
        if self.pause {
            return EQUILIBRIUM;
        }
        let Some(sample) = data.get(self.current_sample).copied() else {
            return EQUILIBRIUM;
        };
        let sample = vec2(f32::from_sample(sample.x), f32::from_sample(sample.y));
        sample * self.volume
    }
}

impl Default for SoundPlayback {
    fn default() -> Self {
        Self { src: 0, volume: 0.0, looping: false, pause: true, current_sample: 0 }
    }
}
