mod backend;
mod conv;
mod error;

pub use error::*;

use std::{cell::RefCell, rc::Rc, sync::mpsc::Sender};

use cpal::traits::HostTrait;

use crate::audio::backend::{AudioCommand, BufferID, PlaybackID, start_backend};

// A little bit more buffers allowed than active playbacks.
const BUFFERS_CAPACITY: usize = 72;
// Max 64 active playbacks should be more that enough
const PLAYBACK_CAPACITY: usize = 64;
const SAMPLE_RATE: u32 = 44100;

pub type Result<T> = std::result::Result<T, Error>;

pub struct AlContext {
    cmd_send: Sender<AudioCommand>,
    _stream: cpal::Stream,

    state: RefCell<AudioContextState>,
}

impl AlContext {
    pub(crate) fn new() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find output device");
        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate: SAMPLE_RATE,

            // 128 is the sweet spot for real-time enough audio (for games).
            #[cfg(not(target_family = "wasm"))]
            buffer_size: cpal::BufferSize::Fixed(128),

            // On WASM with cpal's webaudio backend a fixed buffer of 128 is not enough.
            // There will be a lot of cracks and pops.
            // Use a bigger buffer.
            #[cfg(target_family = "wasm")]
            buffer_size: cpal::BufferSize::Fixed(2048),
        };
        let (cmd_send, stream) =
            start_backend(&config, &device, BUFFERS_CAPACITY, PLAYBACK_CAPACITY);

        AlContext {
            cmd_send,
            _stream: stream,

            state: RefCell::new(AudioContextState {
                next_buffer_id: 0,
                buffer_id_freelist: Vec::new(),

                next_playback_id: 0,
                playback_id_freelist: Vec::new(),
            }),
        }
    }

    // TODO: resampling
    pub fn new_buffer<R>(self: &Rc<Self>, data: hound::WavReader<R>) -> Result<Rc<Buffer>>
    where
        R: std::io::Read,
    {
        let spec = data.spec();
        let data = conv::normalize_audio(data.into_samples(), spec)?;
        let data = data.into_boxed_slice();

        let mut state = self.state.borrow_mut();
        let buffer_id = match state.buffer_id_freelist.pop() {
            Some(x) => x,
            None => {
                if state.next_buffer_id < BUFFERS_CAPACITY - 1 {
                    let res = state.next_buffer_id;
                    state.next_buffer_id += 1;
                    res
                } else {
                    return Err(Error::OutOfSoundBufferCapacity);
                }
            }
        };
        self.cmd_send
            .send(AudioCommand::SetAudioData { buffer_id, data })
            .map_err(|_| Error::AudioBackendUnavailable)?;

        Ok(Rc::new(Buffer { al_ctx: self.clone(), buffer_id }))
    }

    pub fn new_playback(
        self: &Rc<Self>,
        buffer: &Rc<Buffer>,
        looping: bool,
        volume: f32,
    ) -> Result<Playback> {
        let buffer = buffer.clone();
        let mut state = self.state.borrow_mut();
        let playback_id = match state.playback_id_freelist.pop() {
            Some(x) => x,
            None => {
                if state.next_playback_id < PLAYBACK_CAPACITY - 1 {
                    let res = state.next_playback_id;
                    state.next_playback_id += 1;
                    res
                } else {
                    return Err(Error::OutOfSoundBufferCapacity);
                }
            }
        };
        self.cmd_send
            .send(AudioCommand::StartPlayback {
                playback_id,
                src: buffer.buffer_id,
                looping,
                volume,
            })
            .map_err(|_| Error::AudioBackendUnavailable)?;

        Ok(Playback { al_ctx: self.clone(), _buffer: buffer, playback_id })
    }
}

struct AudioContextState {
    next_buffer_id: BufferID,
    buffer_id_freelist: Vec<BufferID>,

    next_playback_id: PlaybackID,
    playback_id_freelist: Vec<PlaybackID>,
}

pub struct Buffer {
    al_ctx: Rc<AlContext>,
    buffer_id: BufferID,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let mut state = self.al_ctx.state.borrow_mut();
        state.buffer_id_freelist.push(self.buffer_id);
    }
}

pub struct Playback {
    al_ctx: Rc<AlContext>,
    _buffer: Rc<Buffer>,
    playback_id: PlaybackID,
}

impl Playback {
    pub fn set_pause(&self, pause: bool) {
        // TODO: log?
        let _ = self
            .al_ctx
            .cmd_send
            .send(AudioCommand::SetPlaybackPause { playback_id: self.playback_id, pause });
    }

    pub fn restart(&self) {
        // TODO: log?
        let _ = self
            .al_ctx
            .cmd_send
            .send(AudioCommand::RestartPlayback { playback_id: self.playback_id });
    }
}

impl Drop for Playback {
    fn drop(&mut self) {
        let mut state = self.al_ctx.state.borrow_mut();
        state.playback_id_freelist.push(self.playback_id);
    }
}
