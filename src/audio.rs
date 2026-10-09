// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

use std::io::Cursor;

pub const SOUND_NAMES: [&str; 11] = [
    "move", "rotate", "drop", "hold", "kick", "clear", "tetris", "tspin", "btb", "fall", "lose",
];

const FILES: [&str; 11] = [
    "se_move.wav",
    "se_rotate.wav",
    "se_hdrop.wav",
    "se_hold.wav",
    "se_spin.wav",
    "se_clear_line.wav",
    "se_clear_tetris.wav",
    "se_clear_spin.wav",
    "se_clear_btb.wav",
    "se_down.wav",
    "se_lose.wav",
];

pub struct Audio {
    sinks: Vec<Option<rodio::Sink>>,
    volume: u32,
    _stream: Option<rodio::OutputStream>,
}

impl Audio {
    pub fn new(volume: u32) -> Audio {
        let mut audio = Audio { sinks: Vec::new(), volume, _stream: None };
        match rodio::OutputStream::try_default() {
            Ok((stream, handle)) => {
                for _ in 0..SOUND_NAMES.len() {
                    audio.sinks.push(rodio::Sink::try_new(&handle).ok());
                }
                audio._stream = Some(stream);
                audio.set_volume(volume);
            }
            Err(_) => {
                audio.sinks = (0..SOUND_NAMES.len()).map(|_| None).collect();
            }
        }
        audio
    }

    pub fn set_volume(&mut self, volume: u32) {
        self.volume = volume;
        for sink in &self.sinks {
            if let Some(sink) = sink {
                sink.set_volume(volume as f32 / 100.0);
            }
        }
    }

    pub fn play(&self, name: &str) {
        if self.volume == 0 {
            return;
        }
        let index = match SOUND_NAMES.iter().position(|&n| n == name) {
            Some(i) => i,
            None => return,
        };
        let data = match load_sound_file(FILES[index]) {
            Some(data) => data,
            None => return,
        };
        if let Some(sink) = &self.sinks[index] {
            sink.clear();
            if let Ok(source) = rodio::Decoder::new_wav(Cursor::new(data)) {
                sink.append(source);
            }
        }
    }
}

fn load_sound_file(name: &str) -> Option<Vec<u8>> {
    std::fs::read(crate::app::resource("se").join(name)).ok()
}
