// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

use crate::ini::{format_num, Ini};

pub const VK_KEYS: [&str; 21] = [
    "KB0", "KB1", "KB2", "KB3", "KB4", "KB5", "KB6", "KB7", "KB8", "KB9", "", "", "KB17", "KB18",
    "", "", "", "", "", "", "",
];

#[derive(Clone)]
pub struct Config {
    pub ini: Ini,
    pub scale: f64,
    pub grid_x: usize,
    pub grid_y: usize,
    pub cell_size: usize,
    pub arr: u32,
    pub das: u32,
    pub sdd: u32,
    pub sds: u32,
    pub garbage: Vec<u32>,
    pub bag_type: u8,
    pub volume: u32,
    pub keybinds: [u32; 21],
    pub auto_color: bool,
    pub ghost_piece: bool,
    pub infinite_hold: bool,
    pub render_textures: bool,
    pub das_cancel: bool,
    pub mirror_queue: bool,
    pub static_bag: bool,
    pub shuffle_bag: bool,
    pub shuffle_hold: bool,
    pub highlight_clear: bool,
    pub skin: String,
    pub texture: String,
}

impl Config {
    pub fn load() -> Config {
        let ini = Ini::load(&crate::app::resource("settings.ini"));
        let mut keybinds = [0u32; 21];
        let defaults = [37u32, 39, 40, 38, 67, 90, 88, 160, 115, 13, 0, 0, 8, 72, 49, 50, 51, 52, 53, 54, 55];
        for (i, name) in VK_KEYS.iter().enumerate() {
            keybinds[i] = if name.is_empty() {
                defaults[i]
            } else {
                ini.read_num("SETTINGS", name, defaults[i])
            };
        }

        let garbage_string = ini.read_str("SETTINGS", "GARBAGE", "1");
        let mut garbage: Vec<u32> = garbage_string
            .split(',')
            .filter_map(|p| p.trim().parse::<u32>().ok())
            .collect();
        if garbage.is_empty() {
            garbage.push(1);
        }

        let mut bag_type = ini.read_num::<u8>("SETTINGS", "BAG_TYPE", 0);
        if bag_type > 2 {
            bag_type = 0;
        }

        Config {
            ini: Ini::load(std::path::Path::new("settings.ini")),
            scale: ini.read_num("SETTINGS", "SCALE", 1.0f64).max(0.05),
            grid_x: ini.read_num("OTHER", "CELL_AMOUNT_X", 10).clamp(4, 32),
            grid_y: ini.read_num("OTHER", "CELL_AMOUNT_Y", 20).clamp(4, 28),
            cell_size: ini.read_num("OTHER", "CELL_SIZE", 30).max(1),
            arr: ini.read_num("SETTINGS", "ARR", 17u32),
            das: ini.read_num("SETTINGS", "DAS", 133u32),
            sdd: ini.read_num("SETTINGS", "SDD", 67u32),
            sds: ini.read_num("SETTINGS", "SDS", 1u32),
            garbage,
            bag_type,
            volume: ini.read_num("SETTINGS", "VOLUME", 70u32),
            keybinds,
            auto_color: ini.read_bool("SETTINGS", "AUTO_COLOR", true),
            ghost_piece: ini.read_bool("SETTINGS", "GHOST_PIECE", true),
            infinite_hold: ini.read_bool("SETTINGS", "INFINITE_HOLD", false),
            render_textures: ini.read_bool("SETTINGS", "RENDER_TEXTURES", false),
            das_cancel: ini.read_bool("SETTINGS", "DAS_CANCELLATION", true),
            mirror_queue: ini.read_bool("SETTINGS", "MIRROR_QUEUE", true),
            static_bag: ini.read_bool("OTHER", "STATIC_BAG", false),
            shuffle_bag: ini.read_bool("OTHER", "SHUFFLE_BAG", false),
            shuffle_hold: ini.read_bool("OTHER", "SHUFFLE_HOLD", false),
            highlight_clear: ini.read_bool("OTHER", "HIGHLIGHT_CLEAR", true),
            skin: ini.read_str("SETTINGS", "SKIN", "DEFAULT"),
            texture: ini.read_str("SETTINGS", "TEXTURE", "template.png"),
        }
    }

    pub fn save_num(&mut self, section: &str, key: &str, value: f64) {
        self.ini_write(section, key, &format_num(value));
    }

    pub fn save_str(&mut self, section: &str, key: &str, value: &str) {
        self.ini_write(section, key, value);
    }

    pub fn save_bool(&mut self, section: &str, key: &str, value: bool) {
        self.ini_write(section, key, if value { "True" } else { "False" });
    }

    fn ini_write(&mut self, section: &str, key: &str, value: &str) {
        self.ini.write(section, key, value);
    }
}

#[derive(Clone)]
pub struct Skin {
    pub name: String,
    pub colors: [u32; 14],
    pub style: u8,
}

impl Skin {
    pub fn load(ini: &Ini, name: &str) -> Skin {
        let defs: [(&str, u32); 13] = [
            ("E", 0x000000),
            ("I", 0x00D0FF),
            ("J", 0x4080FF),
            ("S", 0x40D040),
            ("O", 0xFFE020),
            ("Z", 0xFF4020),
            ("L", 0xFF8020),
            ("T", 0xA040F0),
            ("G", 0xCCCCCC),
            ("F", 0x2F3136),
            ("BKG", 0x2F3136),
            ("BOX", 0x000000),
            ("TXT", 0xFFFFFF),
        ];
        let mut colors = [0u32; 14];
        for (i, (key, def)) in defs.iter().enumerate() {
            let raw = ini.read_num::<u32>(name, key, *def);
            colors[i] = raw & 0xFFFFFF;
        }
        colors[13] = 0xFFFFFF ^ colors[0];
        let mut style = ini.read_num::<u8>(name, "STYLE", 1);
        if style > 1 {
            style = 1;
        }
        Skin { name: name.to_string(), colors, style }
    }

    pub const E: usize = 0;
    pub const Z: usize = 5;
    pub const L: usize = 6;
    pub const T: usize = 7;
    pub const G: usize = 8;
    pub const F: usize = 9;
    pub const BKG: usize = 10;
    pub const BOX: usize = 11;
    pub const TXT: usize = 12;
    pub const REV: usize = 13;
}

pub fn skin_names(ini: &Ini) -> Vec<String> {
    let mut names = Vec::new();
    for line in &ini.lines {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            names.push(t[1..t.len() - 1].to_string());
        }
    }
    if names.is_empty() {
        names.push("DEFAULT".to_string());
    }
    names
}
