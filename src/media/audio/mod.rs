use crossbeam_channel::Sender;

pub use crate::core::game_loop::SoundEffect;

const SOUND_COUNT: usize = 11;

const SOUND_EFFECT_FILES: [(SoundEffect, &str); SOUND_COUNT] = [
    (SoundEffect::Move, "se_move.wav"),
    (SoundEffect::Rotate, "se_rotate.wav"),
    (SoundEffect::HardDrop, "se_hdrop.wav"),
    (SoundEffect::Hold, "se_hold.wav"),
    (SoundEffect::Kick, "se_spin.wav"),
    (SoundEffect::Clear, "se_clear_line.wav"),
    (SoundEffect::Tetris, "se_clear_tetris.wav"),
    (SoundEffect::TSpin, "se_clear_spin.wav"),
    (SoundEffect::BackToBack, "se_clear_btb.wav"),
    (SoundEffect::Fall, "se_down.wav"),
    (SoundEffect::Lose, "se_lose.wav"),
];

pub trait AudioSink {
    fn play(&self, effect: SoundEffect);
    fn set_volume(&self, volume_percent: u8);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCommand {
    Play(SoundEffect),
}

#[derive(Clone)]
pub struct AudioManagerHandle {
    tx: Sender<AudioCommand>,
}

impl AudioManagerHandle {
    pub fn send(&self, command: AudioCommand) {
        let _ = self.tx.send(command);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod rodio_impl;
#[cfg(not(target_arch = "wasm32"))]
pub use rodio_impl::RodioAudioManager;

#[cfg(target_arch = "wasm32")]
mod wasm_impl;
#[cfg(target_arch = "wasm32")]
pub use wasm_impl::RodioAudioManager;
