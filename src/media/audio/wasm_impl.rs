use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};

use crossbeam_channel::Receiver;

use wasm_bindgen::JsError;

use super::{AudioCommand, AudioManagerHandle, AudioSink, SoundEffect, SOUND_EFFECT_FILES};

pub struct RodioAudioManager {
    ctx: Option<web_sys::AudioContext>,
    gain: Option<web_sys::GainNode>,
    buffers: HashMap<SoundEffect, web_sys::AudioBuffer>,
    volume: AtomicU8,
}

impl RodioAudioManager {
    pub fn new() -> Self {
        let ctx = web_sys::AudioContext::new().ok();
        let gain = ctx.as_ref().and_then(|c| c.create_gain().ok());
        if let (Some(ctx), Some(g)) = (&ctx, &gain) {
            let _ = g.connect_with_audio_node(&ctx.destination());
        }
        RodioAudioManager {
            ctx,
            gain,
            buffers: HashMap::new(),
            volume: AtomicU8::new(70),
        }
    }

    pub fn load_default_audio_assets(_root: &std::path::Path) -> Self {
        let mut mgr = Self::new();
        if let Some(ctx) = &mgr.ctx {
            for (effect, filename) in SOUND_EFFECT_FILES {
                if let Some(wav) = crate::resources::builtin_audio_bytes(filename) {
                    if let Ok(buf) = decode_wav(ctx, wav) {
                        mgr.buffers.insert(effect, buf);
                    }
                }
            }
        }
        mgr
    }

    pub fn has_samples(&self) -> bool {
        !self.buffers.is_empty()
    }

    pub fn asset_root(&self) -> &std::path::Path {
        std::path::Path::new("")
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
        let ctx = match &self.ctx {
            Some(c) => c,
            None => return,
        };
        let buffer = match self.buffers.get(&effect) {
            Some(b) => b,
            None => return,
        };
        let source = match ctx.create_buffer_source() {
            Ok(s) => s,
            Err(_) => return,
        };
        source.set_buffer(Some(buffer));
        if let Some(g) = &self.gain {
            let _ = source.connect_with_audio_node(g);
        } else {
            let _ = source.connect_with_audio_node(&ctx.destination());
        }
        let _ = source.start();
    }

    fn set_volume(&self, volume_percent: u8) {
        let v = volume_percent.min(100) as f32 / 100.0;
        self.volume.store(volume_percent.min(100), Ordering::Relaxed);
        if let Some(g) = &self.gain {
            let _ = g.gain().set_value(v * v);
        }
    }
}

fn decode_wav(ctx: &web_sys::AudioContext, data: &[u8]) -> Result<web_sys::AudioBuffer, JsError> {
    if data.len() < 44 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err(JsError::new("not a WAV file"));
    }

    let mut channels = 1u16;
    let mut sample_rate = 44100u32;
    let mut bits_per_sample = 16u16;
    let mut raw_data = &[][..];

    let mut offset = 12;
    while offset + 8 <= data.len() {
        let chunk_len =
            u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()) as usize;
        match &data[offset..offset + 4] {
            b"fmt " => {
                if data.len() >= offset + 8 + 16 {
                    let fmt = offset + 8;
                    let fmt_audio =
                        u16::from_le_bytes(data[fmt..fmt + 2].try_into().unwrap());
                    if fmt_audio != 1 {
                        return Err(JsError::new("not PCM"));
                    }
                    channels =
                        u16::from_le_bytes(data[fmt + 2..fmt + 4].try_into().unwrap());
                    sample_rate =
                        u32::from_le_bytes(data[fmt + 4..fmt + 8].try_into().unwrap());
                    bits_per_sample =
                        u16::from_le_bytes(data[fmt + 14..fmt + 16].try_into().unwrap());
                }
            }
            b"data" => {
                let start = offset + 8;
                let end = data.len().min(start + chunk_len);
                raw_data = &data[start..end];
            }
            _ => {}
        }
        offset += 8 + chunk_len + (chunk_len & 1);
    }

    if raw_data.is_empty() {
        return Err(JsError::new("no data chunk"));
    }

    let bytes_per_sample = (bits_per_sample / 8) as usize;
    let total_samples = raw_data.len() / bytes_per_sample;
    let frames = total_samples / channels as usize;
    if frames == 0 {
        return Err(JsError::new("empty audio data"));
    }

    let all_floats: Vec<f32> = match bits_per_sample {
        16 => raw_data
            .chunks(2)
            .map(|c| {
                let s = i16::from_le_bytes([c[0], c[1]]);
                s as f32 / 32768.0
            })
            .collect(),
        8 => raw_data.iter().map(|&b| (b as f32 - 128.0) / 128.0).collect(),
        _ => return Err(JsError::new("unsupported bit depth")),
    };

    let buf = ctx
        .create_buffer(channels as u32, frames as u32, sample_rate as f32)
        .map_err(|e| JsError::new(&format!("create_buffer failed: {:?}", e)))?;

    for ch in 0..channels as usize {
        let channel_data: Vec<f32> = all_floats
            .iter()
            .skip(ch)
            .step_by(channels as usize)
            .copied()
            .collect();
        buf.copy_to_channel(&channel_data, ch as i32)
            .map_err(|e| JsError::new(&format!("copy_to_channel failed: {:?}", e)))?;
    }

    Ok(buf)
}
