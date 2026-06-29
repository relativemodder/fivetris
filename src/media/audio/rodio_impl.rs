use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crossbeam_channel::Receiver;

use super::{AudioCommand, AudioManagerHandle, AudioSink, SoundEffect, SOUND_COUNT, SOUND_EFFECT_FILES};

fn effect_index(effect: SoundEffect) -> usize {
    effect as usize
}
use crate::resources;

pub struct RodioAudioManager {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    samples: [Option<Vec<u8>>; SOUND_COUNT],
    asset_root: PathBuf,
    volume: AtomicU8,
}

impl RodioAudioManager {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default()
            .ok()
            .map(|(s, h)| (Some(s), Some(h)))
            .unwrap_or((None, None));
        RodioAudioManager {
            _stream: stream,
            handle,
            samples: [
                None, None, None, None, None, None, None, None, None, None, None,
            ],
            asset_root: PathBuf::new(),
            volume: AtomicU8::new(70),
        }
    }

    pub fn load_default_audio_assets(root: &Path) -> Self {
        let mut mgr = Self::new();
        mgr.asset_root = root.to_path_buf();
        mgr.load_samples(root);
        mgr
    }

    fn load_samples(&mut self, root: &Path) {
        for (effect, filename) in SOUND_EFFECT_FILES {
            let path = root.join(filename);
            if let Ok(data) = std::fs::read(&path) {
                self.samples[effect_index(effect)] = Some(data);
            } else if let Some(data) = resources::builtin_audio_bytes(filename) {
                self.samples[effect_index(effect)] = Some(data.to_vec());
            }
        }
    }

    pub fn has_samples(&self) -> bool {
        self.samples.iter().any(|s| s.is_some())
    }

    pub fn asset_root(&self) -> &Path {
        &self.asset_root
    }

    fn decode_sample(sample: &[u8]) -> Option<Decoder<Cursor<Vec<u8>>>> {
        Decoder::new_wav(Cursor::new(sample.to_vec())).ok()
    }

    pub fn command_channel() -> (AudioManagerHandle, Receiver<AudioCommand>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (AudioManagerHandle { tx }, rx)
    }

    pub fn process_commands(&self, rx: &Receiver<AudioCommand>) {
        while let Ok(command) = rx.try_recv() {
            match command {
                AudioCommand::Play(effect) => self.play(effect),
            }
        }
    }
}

impl AudioSink for RodioAudioManager {
    fn play(&self, effect: SoundEffect) {
        let handle = match &self.handle {
            Some(h) => h,
            None => return,
        };
        let data = match &self.samples[effect_index(effect)] {
            Some(d) => d,
            None => return,
        };
        let sink = match Sink::try_new(handle) {
            Ok(s) => s,
            Err(_) => return,
        };
        let linear = self.volume.load(Ordering::Relaxed) as f32 / 100.0;
        sink.set_volume(linear * linear);
        if let Some(source) = Self::decode_sample(data) {
            let source = source.buffered();
            sink.append(source);
            sink.detach();
        }
    }

    fn set_volume(&self, volume_percent: u8) {
        self.volume
            .store(volume_percent.min(100), Ordering::Relaxed);
    }
}
