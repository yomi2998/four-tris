// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

pub fn base_dir() -> &'static PathBuf {
    static BASE: OnceLock<PathBuf> = OnceLock::new();
    BASE.get_or_init(|| {
        let markers = ["settings.ini", "colors.ini", "buttons.bmp", "textures"];
        let score = |dir: &PathBuf| {
            markers
                .iter()
                .filter(|m| dir.join(m).exists())
                .count()
        };
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut best = (score(&cwd), cwd.clone());
        if let Ok(exe) = std::env::current_exe() {
            let mut dir = exe.parent().map(|p| p.to_path_buf());
            for _ in 0..5 {
                let Some(d) = dir else { break };
                let s = score(&d);
                if s > best.0 {
                    best = (s, d.clone());
                }
                dir = d.parent().map(|p| p.to_path_buf());
            }
        }
        best.1
    })
}

pub fn resource(name: &str) -> PathBuf {
    base_dir().join(name)
}

use winit::keyboard::KeyCode;

use crate::config::{Config, Skin};
use crate::game::{piece_get_id, piece_get_name, Game, Mode, GRID_H};
use crate::ini::Ini;
use crate::render::{Bitmap, Canvas};
use crate::state::UndoRing;
use crate::{audio::Audio, vk};

pub const FRAME_MS: f64 = 1000.0 / 60.0;
pub const SETTINGS_PANELSIZE: f64 = 900.0;

pub const MODEBUTTON: usize = 0;
pub const SETTBUTTON: usize = 1;
pub const TESTBUTTON: usize = 2;
pub const HOLDBUTTON: usize = 3;
pub const HOLDDELETE: usize = 4;
pub const HOLDCHECK: usize = 5;
pub const NEXTBUTTON: usize = 6;
pub const SHUFBUTTON: usize = 7;
pub const UNDOBUTTON: usize = 8;
pub const REDOBUTTON: usize = 9;
pub const SNAPBUTTON: usize = 10;
pub const HILIBUTTON: usize = 11;
pub const HCLRBUTTON: usize = 12;
pub const ACOLCHECK: usize = 13;
pub const MIRRBUTTON: usize = 14;
pub const BUTTONS: usize = 15;

pub const KEY_MOVE_L: usize = 0;
pub const KEY_MOVE_R: usize = 1;
pub const KEY_MOVE_D: usize = 2;
pub const KEY_DROP: usize = 3;
pub const KEY_HOLD: usize = 4;
pub const KEY_CCW: usize = 5;
pub const KEY_CW: usize = 6;
pub const KEY_180: usize = 7;
pub const KEY_RESET: usize = 8;
pub const KEY_CLEAR_LINES: usize = 9;
pub const KEY_GARBAGE: usize = 10;
pub const KEY_FOUR_WIDE: usize = 11;
pub const KEY_HLIGHT_RESET: usize = 12;
pub const KEY_HLIGHT_MODE: usize = 13;
pub const KEY_PC_1: usize = 14;

pub fn key_edge(i: usize) -> bool {
    i >= KEY_GARBAGE
}

#[derive(Clone, Copy)]
pub struct KeyState {
    pub vk: u32,
    pub pressed: bool,
    pub time: f64,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Main,
    Settings,
}

pub struct InputBox {
    pub kind: InputKind,
    pub text: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputKind {
    Queue,
    Hold,
}

pub struct PaintSession {
    pub erase: bool,
    pub highlight: bool,
    pub last: (f64, f64),
    pub stroke: usize,
    pub stroke_coord: Vec<(i32, i32)>,
    pub prev_cell: (i32, i32),
}

pub struct SnapState {
    pub image: Bitmap,
    pub drag_start: Option<(f64, f64)>,
    pub drag_cur: (f64, f64),
}

pub struct App {
    pub cfg: Config,
    pub colors_ini: Ini,
    pub skin: Skin,
    pub skins: Vec<String>,
    pub game: Game,
    pub ring: UndoRing,
    pub audio: Audio,
    pub canvas: Canvas,
    pub texture: Option<Bitmap>,
    pub texture_tile: usize,
    pub icons: Option<Bitmap>,
    pub app_icon: Option<Bitmap>,

    pub wsize: [f64; 2],
    pub grid_x: f64,
    pub grid_y: f64,
    pub gbounds: [f64; 4],

    pub screen: Screen,
    pub user_scale: f64,
    pub hidpi: f64,
    pub view_off: (f64, f64),
    pub current_view: f64,

    pub keys: [KeyState; 21],
    pub das_dir: u8,
    pub t_arr: f64,
    pub t_sds: f64,
    pub t_input: f64,
    pub t_gravity: f64,
    pub epoch: Instant,
    pub focused: bool,

    pub mouse: [f64; 2],
    pub mouse_phys: (f64, f64),
    pub edit_color: u8,
    pub ctrl_down: bool,
    pub pending_settings_click: Option<usize>,
    pub buttons: [[bool; 2]; BUTTONS],
    pub paint: [[bool; 2]; 9],
    pub settings_hover: [[bool; 2]; 26],

    pub capture: Option<(usize, usize)>,
    pub input_box: Option<InputBox>,
    pub paint_session: Option<PaintSession>,
    pub drag_slider: Option<usize>,
    pub snap: Option<SnapState>,
    pub redraw_pending: bool,
    pub transition: Option<(f64, Bitmap)>,
    pub snap_request: bool,
    pub window_sync_request: bool,
    pub comment_final_drawn: bool,
}

impl App {
    pub fn new() -> App {
        let cfg = Config::load();
        let colors_ini = Ini::load(&resource("colors.ini"));
        let skins = crate::config::skin_names(&colors_ini);
        let skin_name = if skins.iter().any(|s| *s == cfg.skin) {
            cfg.skin.clone()
        } else {
            skins[0].clone()
        };
        let skin = Skin::load(&colors_ini, &skin_name);

        let static_bag = std::fs::read_to_string(resource("piece_list.txt")).unwrap_or_default();

        let mut game = Game::new(
            cfg.grid_x,
            cfg.grid_y,
            cfg.bag_type,
            cfg.garbage.clone(),
            static_bag,
            cfg.static_bag,
            cfg.highlight_clear,
            cfg.infinite_hold,
        );
        game.bag_seed = seed_from_clock();

        let wsize = [
            (2.0 * 95.0 + cfg.grid_x as f64 * cfg.cell_size as f64).max(300.0),
            (15.0 * 2.0 + (cfg.grid_y as f64 + 2.0) * cfg.cell_size as f64).max(620.0),
        ];
        let grid_x = 95.0 + ((wsize[0] - 190.0) - cfg.cell_size as f64 * cfg.grid_x as f64) / 2.0;
        let grid_y = 12.0 + 2.0 * cfg.cell_size as f64
            + ((wsize[1] - 33.0) - cfg.cell_size as f64 * (cfg.grid_y as f64 + 2.0)) / 2.0;
        let gbounds = [
            grid_x,
            grid_y,
            cfg.grid_x as f64 * cfg.cell_size as f64,
            cfg.grid_y as f64 * cfg.cell_size as f64,
        ];

        let mut keys = [KeyState { vk: 0, pressed: false, time: 0.0 }; 21];
        for (i, item) in keys.iter_mut().enumerate() {
            item.vk = cfg.keybinds[i];
        }

        #[cfg(not(test))]
        let audio = Audio::new(cfg.volume);
        #[cfg(test)]
        let audio = Audio::silent();

        let mut app = App {
            canvas: Canvas::new(cfg.scale),
            colors_ini,
            skin,
            skins,
            game,
            ring: UndoRing::new(100),
            audio,
            texture: None,
            texture_tile: 0,
            icons: None,
            app_icon: None,
            cfg,
            wsize,
            grid_x,
            grid_y,
            gbounds,
            screen: Screen::Main,
            user_scale: 1.0,
            hidpi: 1.0,
            view_off: (0.0, 0.0),
            current_view: 0.0,
            keys,
            das_dir: 0,
            t_arr: 0.0,
            t_sds: 0.0,
            t_input: 0.0,
            t_gravity: 0.0,
            epoch: Instant::now(),
            focused: true,
            mouse: [-1.0, -1.0],
            mouse_phys: (0.0, 0.0),
            edit_color: 8,
            ctrl_down: false,
            pending_settings_click: None,
            buttons: [[false; 2]; BUTTONS],
            paint: [[false; 2]; 9],
            settings_hover: [[false; 2]; 26],
            capture: None,
            input_box: None,
            paint_session: None,
            drag_slider: None,
            snap: None,
            redraw_pending: true,
            transition: None,
            snap_request: false,
            window_sync_request: false,
            comment_final_drawn: false,
        };

        app.load_texture();
        app.icons = load_bitmap(&resource("buttons.bmp"));
        app.app_icon = load_bitmap(&resource("icon.png"));
        app.clear_board();
        app
    }

    pub fn now_ms(&self) -> f64 {
        self.epoch.elapsed().as_secs_f64() * 1000.0
    }

    pub fn comment_animating(&self) -> bool {
        match self.game.active_comment {
            Some((start, duration, _, _, _)) => self.now_ms() < start + duration as f64,
            None => false,
        }
    }

    pub fn total_scale(&self) -> f64 {
        self.user_scale * self.hidpi
    }

    pub fn load_texture(&mut self) {
        let path = resource("textures").join(&self.cfg.texture);
        self.texture = load_bitmap(&path);
        self.texture_tile = self.texture.as_ref().map_or(0, |t| t.h / 10);
    }

    pub fn texture_files(&self) -> Vec<String> {
        let mut names = Vec::new();
        if let Ok(entries) = std::fs::read_dir(resource("textures")) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.to_ascii_lowercase().ends_with(".png") {
                        names.push(name.to_string());
                    }
                }
            }
        }
        names.sort();
        names
    }

    pub fn clear_board(&mut self) {
        self.game.clear_board();
        self.piece_reset_timers();
    }

    pub fn piece_reset_timers(&mut self) {
        let now = self.now_ms();
        self.keys[KEY_MOVE_L].time = now - self.cfg.das as f64;
        self.keys[KEY_MOVE_R].time = now - self.cfg.das as f64;
        self.keys[KEY_MOVE_D].time = now - self.cfg.sdd as f64;
        self.t_gravity = now + gravity_step(self.game.gravity);
        self.t_arr = 0.0;
        self.t_sds = 0.0;
    }

    pub fn switch_mode(&mut self) {
        let next = match self.game.mode {
            Mode::Training => Mode::Cheese,
            Mode::Cheese => Mode::Four,
            Mode::Four => Mode::Master,
            Mode::Master => Mode::PC,
            Mode::PC => Mode::Training,
        };
        self.set_mode(next);
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.game.set_mode(mode);
        self.piece_reset_timers();
        match mode {
            Mode::Training | Mode::Cheese | Mode::Four | Mode::Master => {
                self.game.pending_comment = None;
                self.game.active_comment = None;
            }
            Mode::PC => {
                self.game.pending_comment =
                    Some(("PC MODE".to_string(), "Use KEYS 1-7 to set the Nth. PC.".to_string(), 1750));
            }
        }
    }

    pub fn button_rect(&self, index: usize) -> [f64; 4] {
        let al = 10.0;
        let ar = self.wsize[0] - 85.0;
        let at = 10.0;
        let ab = self.wsize[1] - 16.0;
        match index {
            MODEBUTTON => [al, ab - 80.0, 75.0, 35.0],
            SETTBUTTON => [al, ab - 40.0, 75.0, 35.0],
            TESTBUTTON => [al, ab - 120.0, 75.0, 35.0],
            HOLDBUTTON => [al, at + 150.0, 75.0, 80.0],
            HOLDDELETE => [al + 50.0, at + 155.0, 20.0, 20.0],
            HOLDCHECK => [al, at + 232.0, 75.0, 18.0],
            NEXTBUTTON => [ar, at, 75.0, 240.0],
            SHUFBUTTON => [ar + 50.0, at + 5.0, 20.0, 20.0],
            UNDOBUTTON => [ar, at + 250.0, 35.0, 35.0],
            REDOBUTTON => [ar + 40.0, at + 250.0, 35.0, 35.0],
            SNAPBUTTON => [ar, ab - 45.0, 75.0, 40.0],
            MIRRBUTTON => [ar, ab - 90.0, 75.0, 40.0],
            HILIBUTTON => [ar, at + 295.0, 75.0, 35.0],
            HCLRBUTTON => [ar, at + 335.0, 75.0, 35.0],
            _ => [ar, at + 397.0, 75.0, 18.0],
        }
    }

    pub fn paint_rect(&self, index: usize) -> [f64; 4] {
        let ar = self.wsize[0] - 85.0;
        let at = 10.0;
        let col = index % 3;
        let row = index / 3;
        [ar + 10.0 + col as f64 * 20.0, at + 330.0 + row as f64 * 20.0, 15.0, 15.0]
    }

    pub fn in_rect(&self, rect: &[f64; 4], x: f64, y: f64) -> bool {
        x >= rect[0] && x < rect[0] + rect[2] && y >= rect[1] && y < rect[1] + rect[3]
    }

    pub fn mouse_in_grid(&self) -> bool {
        let [gx, gy, gw, gh] = self.gbounds;
        let (x, y) = (self.mouse[0], self.mouse[1]);
        x >= gx && x < gx + gw && y >= gy && y < gy + gh
    }

    pub fn mouse_cell(&self) -> (i32, i32) {
        let x = ((self.mouse[0] - self.grid_x) / self.cfg.cell_size as f64).floor() as i32;
        let y = ((self.mouse[1] - self.grid_y) / self.cfg.cell_size as f64).floor() as i32 + GRID_H as i32;
        (x, y)
    }

    pub fn keybind_action(&mut self, index: usize) {
        let lost = self.game.lost;
        match index {
            KEY_MOVE_L if lost => {}
            KEY_MOVE_R if lost => {}
            KEY_MOVE_D if lost => {}
            KEY_DROP if lost => {}
            KEY_CCW if lost => {}
            KEY_CW if lost => {}
            KEY_180 if lost => {}
            KEY_MOVE_L => {
                self.game.play_sound("move");
                self.t_arr = 0.0;
                self.das_dir = if self.cfg.das_cancel || !self.keys[KEY_MOVE_R].pressed {
                    b'L'
                } else {
                    0
                };
                self.game.piece_move(0, -1, 0);
            }
            KEY_MOVE_R => {
                self.game.play_sound("move");
                self.t_arr = 0.0;
                self.das_dir = if self.cfg.das_cancel || !self.keys[KEY_MOVE_L].pressed {
                    b'R'
                } else {
                    0
                };
                self.game.piece_move(0, 1, 0);
            }
            KEY_MOVE_D => {
                self.t_sds = 0.0;
                self.game.piece_move(0, 0, 1);
            }
            KEY_DROP => {
                while self.game.piece_move(0, 0, 1) {}
                crate::state::new_undo(&mut self.game, &mut self.ring);
                self.game.play_sound("drop");
                self.game.moves += 1;
                let piece = self.game.bag_get_piece();
                self.game.piece_freeze(piece, self.game.piece_a, self.game.piece_x, self.game.piece_y);
                self.game.check_lines();
                self.game.piece_next();
                self.piece_reset_timers();
            }
            KEY_HOLD => {
                self.game.piece_hold();
                self.piece_reset_timers();
            }
            KEY_CCW => {
                self.game.piece_move(1, 0, 0);
            }
            KEY_CW => {
                self.game.piece_move(3, 0, 0);
            }
            KEY_180 => {
                self.game.piece_move(2, 0, 0);
            }
            KEY_RESET => {
                self.clear_board();
            }
            KEY_CLEAR_LINES => {
                if self.game.has_full_lines() {
                    crate::state::new_undo(&mut self.game, &mut self.ring);
                }
                self.game.grid_clear_full_lines();
            }
            KEY_GARBAGE => self.game.grid_spawn_garbage(),
            KEY_FOUR_WIDE => self.game.grid_spawn_4w(),
            KEY_HLIGHT_RESET => {
                self.game.highlight_reset();
                self.game.hlight_on = false;
            }
            KEY_HLIGHT_MODE => {
                self.game.highlight_mode = !self.game.highlight_mode;
                self.game.changed = true;
            }
            i @ KEY_PC_1..=20 => {
                if self.game.mode == Mode::PC {
                    let leftover = i - KEY_PC_1 + 1;
                    self.game.pc_set_leftover(leftover);
                    let names = ["1st", "2nd", "3rd", "4th.", "5th.", "6th.", "7th."];
                    let n = self.game.pc_leftover;
                    self.game.pending_comment = Some((
                        format!("{} PC", names[leftover - 1]),
                        format!("Bag leftover: {} piece{}", n, if n == 1 { "." } else { "s." }),
                        1000,
                    ));
                    self.piece_reset_timers();
                }
            }
            _ => {}
        }
    }

    pub fn handle_key_down(&mut self, vk: u32, ctrl: bool, shift: bool, alt: bool) {
        if self.capture.is_some() {
            let (kb, _stt) = self.capture.unwrap();
            if vk == vk::VK_ESCAPE {
                self.capture = None;
                self.game.changed = true;
                return;
            }
            self.cfg.keybinds[kb] = vk;
            self.keys[kb].vk = vk;
            self.cfg.save_num("SETTINGS", crate::config::VK_KEYS[kb], vk as f64);
            self.game.changed = true;
            self.capture = None;
            return;
        }

        if ctrl {
            match (vk, shift) {
                (0x5A, false) => {
                    self.undo();
                    return;
                }
                (0x5A, true) | (0x59, _) => {
                    self.redo();
                    return;
                }
                (0x43, _) => {
                    self.copy_state();
                    return;
                }
                (0x56, _) => {
                    self.paste_state();
                    return;
                }
                (0x51, _) => {
                    self.open_queue_input();
                    return;
                }
                _ => {}
            }
        }

        if self.input_box.is_some() {
            self.input_box_key(vk, shift);
            return;
        }

        if vk == 0 {
            return;
        }

        if vk == vk::VK_ESCAPE && self.screen == Screen::Settings {
            self.close_settings();
            return;
        }

        for i in 0..self.keys.len() {
            if self.keys[i].vk == vk && !self.keys[i].pressed {
                self.keys[i].pressed = true;
                self.keys[i].time = self.now_ms();
                if !key_edge(i) {
                    self.keybind_action(i);
                }
            }
        }
        let _ = alt;
    }

    pub fn handle_key_up(&mut self, vk: u32) {
        if vk == 0 {
            return;
        }
        for i in 0..self.keys.len() {
            if self.keys[i].vk == vk && self.keys[i].pressed {
                self.keys[i].pressed = false;
                self.keys[i].time = self.now_ms();
                if key_edge(i) {
                    self.keybind_action(i);
                }
            }
        }
    }

    pub fn undo(&mut self) {
        crate::state::undo(&mut self.game, &mut self.ring);
        self.piece_reset_timers();
    }

    pub fn redo(&mut self) {
        crate::state::redo(&mut self.game, &mut self.ring);
        self.piece_reset_timers();
    }

    pub fn copy_state(&mut self) {
        let text = crate::state::state_encode(&self.game);
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
    }

    pub fn paste_state(&mut self) {
        let clipboard = arboard::Clipboard::new();
        if let Ok(mut clipboard) = clipboard {
            if let Ok(text) = clipboard.get_text() {
                if crate::state::state_decode(&mut self.game, &text.trim()) {
                    if self.cfg.shuffle_bag {
                        if self.cfg.shuffle_hold {
                            self.game.hold_shuffle();
                        }
                        self.game.bag_shuffle();
                        self.game.bag_reseed();
                    }
                    self.piece_reset_timers();
                    return;
                }
            }
            if let Ok(image) = clipboard.get_image() {
                let bmp = Bitmap::from_rgba8(
                    image.width as usize,
                    image.height as usize,
                    image.bytes.into_owned().as_slice(),
                );
                self.fill_board_from_bitmap(bmp);
            }
        }
    }

    pub fn open_queue_input(&mut self) {
        let text: String = self.game.bag.iter().map(|&p| piece_get_name(p)).collect();
        self.input_box = Some(InputBox { kind: InputKind::Queue, text });
        self.game.changed = true;
    }

    pub fn open_hold_input(&mut self) {
        let text = if self.game.piece_h < 0 {
            String::new()
        } else {
            piece_get_name(self.game.piece_h).to_string()
        };
        self.input_box = Some(InputBox { kind: InputKind::Hold, text });
        self.game.changed = true;
    }

    pub fn input_box_text(&mut self, text: &str) {
        let Some(box_state) = &mut self.input_box else { return };
        for ch in text.chars() {
            if !ch.is_control() && box_state.text.chars().count() < 30000 {
                box_state.text.push(ch);
                self.game.changed = true;
            }
        }
    }

    pub fn input_box_key(&mut self, vk: u32, _shift: bool) {
        let Some(box_state) = &mut self.input_box else { return };
        match vk {
            vk::VK_RETURN => {
                let text = box_state.text.clone();
                let kind = box_state.kind;
                self.input_box = None;
                match kind {
                    InputKind::Queue => {
                        self.game.bag_load_from_string(&text);
                        self.piece_reset_timers();
                    }
                    InputKind::Hold => {
                        let first = text.trim().chars().next();
                        self.game.piece_h = match first {
                            None => -1,
                            Some(c) => piece_get_id(c),
                        };
                        self.game.swapped = false;
                        self.game.changed = true;
                    }
                }
            }
            vk::VK_ESCAPE => {
                self.input_box = None;
                self.game.changed = true;
            }
            vk::VK_BACK => {
                box_state.text.pop();
                self.game.changed = true;
            }
            _ => {}
        }
    }

    pub fn fill_board_from_bitmap(&mut self, bmp: Bitmap) {
        crate::state::new_undo(&mut self.game, &mut self.ring);
        self.game.piece_reset();
        let gx = self.game.grid_w();
        let gh = self.game.grid_h();
        for col in &mut self.game.grid {
            for cell in col {
                *cell = 0;
            }
        }
        self.game.changed = true;

        if bmp.w == 0 || bmp.h == 0 {
            return;
        }
        let cell = bmp.w as f64 / gx as f64;
        let center = cell / 2.0;
        let offset = bmp.h as f64 - (bmp.h as f64 / cell).floor() * cell;
        let k = (gh as f64 - (bmp.h as f64 / cell).floor()).max(0.0) as usize;

        for i in 0..gx {
            for j in 0..gh.saturating_sub(k) {
                let sx = ((i as f64 * cell) + center).floor() as usize;
                let sy = ((offset + j as f64 * cell) + center).floor() as usize;
                let pixel = bmp
                    .px
                    .get(sy.min(bmp.h - 1) * bmp.w + sx.min(bmp.w - 1))
                    .copied()
                    .unwrap_or(0);
                let (h, s, l) = rgb_to_hsl255(unpack_r(pixel), unpack_g(pixel), unpack_b(pixel));
                let value = if l > 50.0 {
                    if s < 20.0 {
                        8
                    } else {
                        nearest_piece_hue(h)
                    }
                } else {
                    0
                };
                self.game.grid[i][j + k] = value;
            }
        }
    }

    pub fn step(&mut self) {
        let now = self.now_ms();
        self.update_hover();

        if self.screen == Screen::Settings {
            self.redraw_pending = true;
        }

        while now >= self.t_input {
            self.game_input(now);
            self.t_input += FRAME_MS;
        }
        if self.t_input > now + FRAME_MS {
            self.t_input = now;
        }

        let step = gravity_step(self.game.gravity);
        if step.is_finite() {
            while now >= self.t_gravity {
                self.t_gravity += step;
                if !self.game.piece_move(0, 0, 1) {
                    if self.t_gravity < now {
                        self.t_gravity = now;
                    }
                    break;
                }
            }
        }

        for name in self.game.drain_sounds() {
            self.audio.play(name);
        }

        if let Some((title, text, time)) = self.game.pending_comment.take() {
            self.game.active_comment = Some((now, time, title, text, false));
            self.comment_final_drawn = false;
        }
        if let Some((start, time, _, _, _)) = self.game.active_comment {
            if now >= start + time as f64 && !self.comment_final_drawn {
                self.comment_final_drawn = true;
                self.redraw_pending = true;
            }
        }
        if self.comment_animating() || self.transition.is_some() {
            self.redraw_pending = true;
        }

        if self.game.changed {
            self.redraw_pending = true;
            self.game.changed = false;
        }
    }

    pub fn game_input(&mut self, now: f64) {
        let das = self.cfg.das as f64;
        let arr = self.cfg.arr as f64;
        let sdd = self.cfg.sdd as f64;
        let sds = self.cfg.sds as f64;

        if !self.keys[KEY_MOVE_L].pressed && !self.keys[KEY_MOVE_R].pressed {
            self.das_dir = 0;
        }
        match self.das_dir {
            b'L' => {
                if self.keys[KEY_MOVE_L].pressed {
                    let kt = self.keys[KEY_MOVE_L].time;
                    while kt + das + self.t_arr < now {
                        if !self.game.piece_move(0, -1, 0) {
                            break;
                        }
                        self.t_arr += arr;
                        if self.cfg.arr > 15 {
                            self.game.play_sound("move");
                        }
                    }
                }
                if !self.keys[KEY_MOVE_L].pressed && self.keys[KEY_MOVE_R].pressed {
                    self.keys[KEY_MOVE_R].time = self.keys[KEY_MOVE_L].time;
                    self.das_dir = b'R';
                    self.t_arr = 0.0;
                }
            }
            b'R' => {
                if self.keys[KEY_MOVE_R].pressed {
                    let kt = self.keys[KEY_MOVE_R].time;
                    while kt + das + self.t_arr < now {
                        if !self.game.piece_move(0, 1, 0) {
                            break;
                        }
                        self.t_arr += arr;
                        if self.cfg.arr > 15 {
                            self.game.play_sound("move");
                        }
                    }
                }
                if !self.keys[KEY_MOVE_R].pressed && self.keys[KEY_MOVE_L].pressed {
                    self.keys[KEY_MOVE_L].time = self.keys[KEY_MOVE_R].time;
                    self.das_dir = b'L';
                    self.t_arr = 0.0;
                }
            }
            _ => {}
        }

        if !self.keys[KEY_MOVE_D].pressed {
            self.t_sds = 0.0;
        } else {
            let kt = self.keys[KEY_MOVE_D].time;
            while kt + sdd + self.t_sds < now {
                if !self.game.piece_move(0, 0, 1) {
                    break;
                }
                self.t_sds += sds;
            }
        }
    }

    pub fn scaling(&mut self, scale: f64) {
        self.user_scale = (scale * 1e6).round() / 1e6;
        self.canvas.scale = self.total_scale();
        self.cfg.save_num("SETTINGS", "SCALE", self.user_scale);
    }

    pub fn layout_update(&mut self, phys_w: f64, phys_h: f64, hidpi: f64) {
        if phys_w < 1.0 || phys_h < 1.0 || hidpi < 0.05 {
            return;
        }
        let logical_w = phys_w / hidpi;
        let logical_h = phys_h / hidpi;
        let scale_h = logical_h / self.wsize[1];
        let scale_w = logical_w / self.wsize[0];
        let scale = scale_h.min(scale_w);
        if (scale - self.user_scale).abs() > 0.0005 {
            self.scaling(scale);
        }
        let content_w = (self.wsize[0] * self.total_scale()).round();
        let content_h = (self.wsize[1] * self.total_scale()).round();
        self.view_off = (
            ((phys_w - content_w) / 2.0).max(0.0),
            ((phys_h - content_h) / 2.0).max(0.0),
        );
        self.canvas.set_off((self.view_off.0 as i32, self.view_off.1 as i32));
        self.game.changed = true;
    }

    pub fn request_reset_size(&mut self) {
        self.scaling(1.0);
        self.window_sync_request = true;
    }
}

impl App {
    pub fn mouse_panel(&self) -> (f64, f64) {
        (self.mouse[0], self.mouse[1] + self.current_view)
    }

    pub fn update_hover(&mut self) {
        if self.screen == Screen::Settings {
            let (mx, my) = self.mouse_panel();
            for i in 0..26 {
                let hovered = self.in_rect(&self.settings_item_rect(i), mx, my);
                if self.settings_hover[i][0] != hovered {
                    self.settings_hover[i][0] = hovered;
                    self.game.changed = true;
                }
            }
            return;
        }
        let (mx, my) = (self.mouse[0], self.mouse[1]);
        let mut target = BUTTONS;
        let mut target_area = f64::INFINITY;
        for i in 0..BUTTONS {
            let rect = self.button_rect(i);
            if self.in_rect(&rect, mx, my) && rect[2] * rect[3] < target_area {
                target_area = rect[2] * rect[3];
                target = i;
            }
        }
        for i in 0..BUTTONS {
            let hovered = i == target;
            if self.buttons[i][0] != hovered {
                self.buttons[i][0] = hovered;
                self.game.changed = true;
            }
        }
        for i in 0..9 {
            let hovered = self.in_rect(&self.paint_rect(i), mx, my);
            if self.paint[i][0] != hovered {
                self.paint[i][0] = hovered;
                self.game.changed = true;
            }
        }
    }

    pub fn on_mouse_down(&mut self, left: bool) {
        if self.screen == Screen::Settings {
            if self.capture.is_some() || self.drag_slider.is_some() {
                return;
            }
            for i in 0..26 {
                if self.settings_hover[i][0] {
                    match i {
                        14 | 15 | 19 | 24 | 25 => {
                            self.drag_slider = Some(i);
                            self.slider_update(i);
                        }
                        13 => {
                            self.cfg.render_textures = !self.cfg.render_textures;
                            self.cfg.save_bool("SETTINGS", "RENDER_TEXTURES", self.cfg.render_textures);
                            self.game.changed = true;
                        }
                        16 => {
                            self.cfg.das_cancel = !self.cfg.das_cancel;
                            self.cfg.save_bool("SETTINGS", "DAS_CANCELLATION", self.cfg.das_cancel);
                            self.game.changed = true;
                        }
                        17 => {
                            self.cfg.ghost_piece = !self.cfg.ghost_piece;
                            self.cfg.save_bool("SETTINGS", "GHOST_PIECE", self.cfg.ghost_piece);
                            self.game.changed = true;
                        }
                        _ => {
                            self.pending_settings_click = Some(i);
                        }
                    }
                    return;
                }
            }
            return;
        }

        if self.mouse_in_grid() && self.input_box.is_none() {
            if self.game.highlight_mode {
                self.paint_session = Some(PaintSession {
                    erase: !left,
                    highlight: true,
                    last: (self.mouse[0], self.mouse[1]),
                    stroke: 0,
                    stroke_coord: Vec::new(),
                    prev_cell: (-1, -1),
                });
                self.game.hlight_on = true;
                let c = if left { crate::config::Skin::REV as u8 } else { 0 };
                self.paint_cell(c);
            } else {
                crate::state::new_undo(&mut self.game, &mut self.ring);
                self.paint_session = Some(PaintSession {
                    erase: !left,
                    highlight: false,
                    last: (self.mouse[0], self.mouse[1]),
                    stroke: 0,
                    stroke_coord: Vec::new(),
                    prev_cell: (-1, -1),
                });
                let c = if left { self.edit_color } else { 0 };
                self.paint_cell(c);
            }
        }
    }

    fn paint_cell(&mut self, c: u8) {
        let Some(session) = self.paint_session.as_mut() else { return };
        let om = session.last;
        let cur = (self.mouse[0], self.mouse[1]);
        let ctrl = self.ctrl_down;
        let highlight = session.highlight;
        let mut stroke = session.stroke;
        let mut coords = std::mem::take(&mut session.stroke_coord);
        let mut prev = session.prev_cell;
        let cell = self.cfg.cell_size as f64;

        for k in 1..=7 {
            let mx = om.0 + (cur.0 - om.0) * k as f64 / 7.0;
            let my = om.1 + (cur.1 - om.1) * k as f64 / 7.0;
            let cx = ((mx - self.grid_x) / cell).floor() as i32;
            let cy = ((my - self.grid_y) / cell).floor() as i32 + GRID_H as i32;
            if cx < 0 || cy < 0 || cx >= self.game.grid_w() as i32 || cy >= self.game.grid_h() as i32 {
                continue;
            }
            if (cx, cy) == prev {
                continue;
            }
            prev = (cx, cy);
            let (ux, uy) = (cx as usize, cy as usize);
            if highlight {
                if ctrl {
                    self.game.highlight_reset();
                    break;
                }
                if self.game.hlight[ux][uy] != c {
                    self.game.hlight[ux][uy] = c;
                    self.game.changed = true;
                }
            } else if ctrl {
                for (i, col) in self.game.grid.iter_mut().enumerate() {
                    col[uy] = c;
                    if i == ux {
                        col[uy] = 0;
                    }
                }
                self.game.changed = true;
            } else if self.game.grid[ux][uy] != c {
                self.game.grid[ux][uy] = c;
                self.game.changed = true;
                if stroke < 4 {
                    coords.push((cx, cy));
                }
                stroke += 1;
                if c == 8 && self.cfg.auto_color {
                    self.game.auto_color(cx, cy, stroke, &coords);
                }
                if c == 0 && self.cfg.auto_color {
                    self.game.auto_color(cx - 1, cy, 0, &[]);
                    self.game.auto_color(cx + 1, cy, 0, &[]);
                    self.game.auto_color(cx, cy - 1, 0, &[]);
                    self.game.auto_color(cx, cy + 1, 0, &[]);
                }
            }
        }

        if let Some(session) = self.paint_session.as_mut() {
            session.stroke = stroke;
            session.stroke_coord = coords;
            session.last = cur;
            session.prev_cell = prev;
        }
    }

    pub fn on_mouse_move(&mut self) {
        if let Some(index) = self.drag_slider {
            self.slider_update(index);
            return;
        }
        if self.paint_session.is_some() {
            self.paint_session_step();
        }
    }

    fn paint_session_step(&mut self) {
        let Some(session) = self.paint_session.as_ref() else { return };
        let c = if session.highlight {
            if session.erase {
                0
            } else {
                crate::config::Skin::REV as u8
            }
        } else if session.erase {
            0
        } else {
            self.edit_color
        };
        self.paint_cell(c);
    }

    pub fn on_mouse_up(&mut self, left: bool) {
        if self.paint_session.take().is_some() {
            return;
        }
        if self.drag_slider.take().is_some() {
            return;
        }
        if self.screen == Screen::Settings {
            if let Some(i) = self.pending_settings_click.take() {
                let (mx, my) = self.mouse_panel();
                if self.in_rect(&self.settings_item_rect(i), mx, my) {
                    self.settings_release_action(i);
                }
            }
            return;
        }
        if !left {
            return;
        }
        if !self.game.highlight_mode {
            let (mx, my) = (self.mouse[0], self.mouse[1]);
            for i in 0..9 {
                if self.in_rect(&self.paint_rect(i), mx, my) {
                    if i == 8 {
                        crate::state::new_undo(&mut self.game, &mut self.ring);
                        self.game.fill_color(8);
                    } else {
                        self.edit_color = (i + 1) as u8;
                        self.game.changed = true;
                    }
                    return;
                }
            }
        }
        for i in 0..BUTTONS {
            if self.buttons[i][0] {
                self.button_action(i);
                return;
            }
        }
    }

    pub fn settings_release_action(&mut self, i: usize) {
        match i {
            0..=9 | 23 => {
                let kb = settings_to_keybind(i);
                self.capture = Some((kb, i));
                self.game.changed = true;
            }
            10 => self.set_skin(-1),
            12 => self.set_skin(1),
            20 => self.set_texture(-1),
            22 => self.set_texture(1),
            18 => self.request_reset_size(),
            _ => {}
        }
    }

    pub fn button_action(&mut self, index: usize) {
        match index {
            MODEBUTTON => self.switch_mode(),
            SETTBUTTON => self.open_settings(),
            TESTBUTTON => {}
            SHUFBUTTON => self.game.bag_shuffle(),
            NEXTBUTTON => self.open_queue_input(),
            HOLDCHECK => {
                self.cfg.infinite_hold = !self.cfg.infinite_hold;
                self.game.infinite_hold = self.cfg.infinite_hold;
                if self.cfg.infinite_hold {
                    self.game.swapped = false;
                }
                self.cfg.save_bool("SETTINGS", "INFINITE_HOLD", self.cfg.infinite_hold);
                self.game.changed = true;
            }
            HOLDDELETE => {
                self.game.hold_reset();
            }
            HOLDBUTTON => self.open_hold_input(),
            SNAPBUTTON => self.start_snap(),
            MIRRBUTTON => {
                crate::state::new_undo(&mut self.game, &mut self.ring);
                self.game.grid_mirror();
                if self.cfg.mirror_queue {
                    self.game.bag_mirror();
                    self.game.hold_mirror();
                }
            }
            UNDOBUTTON => self.undo(),
            REDOBUTTON => self.redo(),
            HILIBUTTON => {
                self.game.highlight_mode = !self.game.highlight_mode;
                self.game.changed = true;
            }
            HCLRBUTTON => {
                if self.game.highlight_mode {
                    self.game.highlight_reset();
                    self.game.hlight_on = false;
                }
            }
            ACOLCHECK => {
                self.cfg.auto_color = !self.cfg.auto_color;
                self.cfg.save_bool("SETTINGS", "AUTO_COLOR", self.cfg.auto_color);
                self.game.changed = true;
            }
            _ => {}
        }
    }

    pub fn on_wheel(&mut self, down: bool, ctrl: bool, alt: bool) {
        if self.screen == Screen::Settings {
            self.scroll(if down { 100.0 } else { -100.0 });
            return;
        }
        if ctrl {
            self.game.grid_shift(if down { -1 } else { 1 });
        }
        if alt {
            self.game.grid_roll(if down { -1 } else { 1 });
        }
    }

    pub fn scroll(&mut self, amount: f64) {
        self.current_view += amount;
        let max = SETTINGS_PANELSIZE - self.wsize[1];
        if self.current_view > max {
            self.current_view = max;
        }
        if self.current_view < 0.0 {
            self.current_view = 0.0;
        }
        self.game.changed = true;
    }

    pub fn open_settings(&mut self) {
        self.screen = Screen::Settings;
        self.current_view = 0.0;
        self.game.changed = true;
    }

    pub fn close_settings(&mut self) {
        self.screen = Screen::Main;
        self.current_view = 0.0;
        self.capture = None;
        self.drag_slider = None;
        self.pending_settings_click = None;
        self.game.changed = true;
    }

    pub fn start_snap(&mut self) {
        self.snap_request = true;
    }

    pub fn middle_click(&mut self) {
        if self.input_box.is_none() && self.mouse_in_grid() {
            let (cx, cy) = self.mouse_cell();
            if self.game.block_in_bounds(cx, cy) {
                let current = self.game.hlight[cx as usize][cy as usize];
                let c = if current == 0 { crate::config::Skin::REV as u8 } else { 0 };
                self.game.hlight[cx as usize][cy as usize] = c;
                self.game.hlight_on = true;
                self.game.changed = true;
            }
        }
    }
}


pub fn settings_to_keybind(i: usize) -> usize {
    match i {
        0 => 0,
        1 => 1,
        2 => 5,
        3 => 7,
        4 => 6,
        5 => 4,
        6 => 2,
        7 => 3,
        8 => 8,
        9 => 13,
        23 => 12,
        _ => 0,
    }
}

impl App {
    pub fn set_skin(&mut self, direction: i32) {
        if self.skins.is_empty() {
            return;
        }
        let mut index = self.skins.iter().position(|s| *s == self.skin.name).unwrap_or(0) as i32;
        index += direction;
        let len = self.skins.len() as i32;
        if index < 1 {
            index = len - 1;
        }
        if index > len - 1 {
            index = 0;
        }
        self.skin = Skin::load(&self.colors_ini, &self.skins[index as usize]);
        self.cfg.skin = self.skin.name.clone();
        self.cfg.save_str("SETTINGS", "SKIN", &self.skin.name);
        self.game.changed = true;
    }

    pub fn set_texture(&mut self, direction: i32) {
        let files = self.texture_files();
        if files.is_empty() {
            return;
        }
        let mut index = files.iter().position(|f| *f == self.cfg.texture).unwrap_or(0) as i32;
        index = (index + direction).rem_euclid(files.len() as i32);
        self.cfg.texture = files[index as usize].clone();
        let texture = self.cfg.texture.clone();
        self.cfg.save_str("SETTINGS", "TEXTURE", &texture);
        self.load_texture();
        self.game.changed = true;
    }

    pub fn slider_update(&mut self, i: usize) {
        let rect = self.settings_item_rect(i);
        let t = ((self.mouse[0] - rect[0]) / rect[2]).clamp(0.0, 1.0);
        let (min, max, key): (u32, u32, &str) = match i {
            14 => (0, 32, "ARR"),
            15 => (0, 256, "DAS"),
            19 => (0, 100, "VOLUME"),
            24 => (0, 32, "SDS"),
            _ => (0, 256, "SDD"),
        };
        let value = min + (t * (max - min) as f64).round() as u32;
        match i {
            14 => self.cfg.arr = value,
            15 => self.cfg.das = value,
            19 => {
                self.cfg.volume = value;
                self.audio.set_volume(value);
            }
            24 => self.cfg.sds = value,
            _ => self.cfg.sdd = value,
        }
        self.cfg.save_num("SETTINGS", key, value as f64);
        self.game.changed = true;
    }

    pub fn settings_item_rect(&self, i: usize) -> [f64; 4] {
        let align_c = self.wsize[0] / 2.0;
        let y0 = 47.0;
        let y1 = 237.0;
        let y2 = 532.0;
        let y3 = 812.0;
        match i {
            10 => [align_c - 85.0, y0, 35.0, 35.0],
            11 => [align_c - 45.0, y0, 90.0, 35.0],
            12 => [align_c + 50.0, y0, 35.0, 35.0],
            20 => [align_c - 140.0, y0 + 40.0, 35.0, 35.0],
            21 => [align_c - 100.0, y0 + 40.0, 200.0, 35.0],
            22 => [align_c + 105.0, y0 + 40.0, 35.0, 35.0],
            13 => [align_c - 75.0, y0 + 80.0, 140.0, 19.0],
            18 => [align_c - 45.0, y0 + 110.0, 90.0, 35.0],
            0 => [align_c - 92.0, y1, 90.0, 35.0],
            1 => [align_c + 2.0, y1, 90.0, 35.0],
            6 => [align_c - 92.0, y1 + 40.0, 90.0, 35.0],
            7 => [align_c + 2.0, y1 + 40.0, 90.0, 35.0],
            5 => [align_c - 45.0, y1 + 80.0, 90.0, 35.0],
            2 => [align_c - 140.0, y1 + 125.0, 90.0, 35.0],
            3 => [align_c - 45.0, y1 + 125.0, 90.0, 35.0],
            4 => [align_c + 50.0, y1 + 125.0, 90.0, 35.0],
            8 => [align_c - 45.0, y1 + 170.0, 90.0, 35.0],
            9 => [align_c - 92.0, y1 + 215.0, 90.0, 35.0],
            23 => [align_c + 2.0, y1 + 215.0, 90.0, 35.0],
            14 => [align_c - 100.0, y2, 200.0, 35.0],
            15 => [align_c - 100.0, y2 + 45.0, 200.0, 35.0],
            16 => [align_c - 70.0, y2 + 90.0, 130.0, 19.0],
            24 => [align_c - 100.0, y2 + 125.0, 200.0, 35.0],
            25 => [align_c - 100.0, y2 + 170.0, 200.0, 35.0],
            17 => [align_c - 70.0, y2 + 215.0, 130.0, 19.0],
            _ => [align_c - 100.0, y3, 200.0, 35.0],
        }
    }
}

pub fn gravity_step(gravity: u32) -> f64 {
    if gravity == 0 {
        f64::INFINITY
    } else {
        1000.0 / gravity as f64
    }
}

fn seed_from_clock() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    nanos % 65536
}

pub fn unpack_r(c: u32) -> u8 {
    (c >> 16) as u8
}
pub fn unpack_g(c: u32) -> u8 {
    (c >> 8) as u8
}
pub fn unpack_b(c: u32) -> u8 {
    c as u8
}

const PIECE_HUES: [(f64, u8); 7] = [
    (0.0, 5),
    (21.25, 6),
    (42.5, 4),
    (85.0, 3),
    (127.5, 1),
    (170.0, 2),
    (212.5, 7),
];

pub fn nearest_piece_hue(h: f64) -> u8 {
    let mut best = 5u8;
    let mut best_d = f64::MAX;
    for (hue, id) in PIECE_HUES {
        let raw = (h - hue).abs();
        let d = raw.min(255.0 - raw);
        if d < best_d {
            best_d = d;
            best = id;
        }
    }
    best
}

pub fn rgb_to_hsl255(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let l = (max + min) / 2.0;
    if max == min {
        return (0.0, 0.0, l * 255.0);
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if max == rf {
        (gf - bf) / d + if gf < bf { 6.0 } else { 0.0 }
    } else if max == gf {
        (bf - rf) / d + 2.0
    } else {
        (rf - gf) / d + 4.0
    };
    (h / 6.0 * 255.0, s * 255.0, l * 255.0)
}

pub fn load_bitmap(path: &std::path::Path) -> Option<Bitmap> {
    let data = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&data).ok()?;
    let rgba = img.to_rgba8();
    Some(Bitmap::from_rgba8(rgba.width() as usize, rgba.height() as usize, rgba.as_raw()))
}

pub fn vk_from_keycode(code: KeyCode) -> u32 {
    let vk = match code {
        KeyCode::Backspace => 0x08,
        KeyCode::Tab => 0x09,
        KeyCode::Enter => 0x0D,
        KeyCode::Escape => 0x1B,
        KeyCode::Space => 0x20,
        KeyCode::PageUp => 0x21,
        KeyCode::PageDown => 0x22,
        KeyCode::End => 0x23,
        KeyCode::Home => 0x24,
        KeyCode::ArrowLeft => 0x25,
        KeyCode::ArrowUp => 0x26,
        KeyCode::ArrowRight => 0x27,
        KeyCode::ArrowDown => 0x28,
        KeyCode::Insert => 0x2D,
        KeyCode::Delete => 0x2E,
        KeyCode::Digit0 => 0x30,
        KeyCode::Digit1 => 0x31,
        KeyCode::Digit2 => 0x32,
        KeyCode::Digit3 => 0x33,
        KeyCode::Digit4 => 0x34,
        KeyCode::Digit5 => 0x35,
        KeyCode::Digit6 => 0x36,
        KeyCode::Digit7 => 0x37,
        KeyCode::Digit8 => 0x38,
        KeyCode::Digit9 => 0x39,
        KeyCode::KeyA => 0x41,
        KeyCode::KeyB => 0x42,
        KeyCode::KeyC => 0x43,
        KeyCode::KeyD => 0x44,
        KeyCode::KeyE => 0x45,
        KeyCode::KeyF => 0x46,
        KeyCode::KeyG => 0x47,
        KeyCode::KeyH => 0x48,
        KeyCode::KeyI => 0x49,
        KeyCode::KeyJ => 0x4A,
        KeyCode::KeyK => 0x4B,
        KeyCode::KeyL => 0x4C,
        KeyCode::KeyM => 0x4D,
        KeyCode::KeyN => 0x4E,
        KeyCode::KeyO => 0x4F,
        KeyCode::KeyP => 0x50,
        KeyCode::KeyQ => 0x51,
        KeyCode::KeyR => 0x52,
        KeyCode::KeyS => 0x53,
        KeyCode::KeyT => 0x54,
        KeyCode::KeyU => 0x55,
        KeyCode::KeyV => 0x56,
        KeyCode::KeyW => 0x57,
        KeyCode::KeyX => 0x58,
        KeyCode::KeyY => 0x59,
        KeyCode::KeyZ => 0x5A,
        KeyCode::Numpad0 => 0x60,
        KeyCode::Numpad1 => 0x61,
        KeyCode::Numpad2 => 0x62,
        KeyCode::Numpad3 => 0x63,
        KeyCode::Numpad4 => 0x64,
        KeyCode::Numpad5 => 0x65,
        KeyCode::Numpad6 => 0x66,
        KeyCode::Numpad7 => 0x67,
        KeyCode::Numpad8 => 0x68,
        KeyCode::Numpad9 => 0x69,
        KeyCode::NumpadMultiply => 0x6A,
        KeyCode::NumpadAdd => 0x6B,
        KeyCode::NumpadSubtract => 0x6D,
        KeyCode::NumpadDecimal => 0x6E,
        KeyCode::NumpadDivide => 0x6F,
        KeyCode::F1 => 0x70,
        KeyCode::F2 => 0x71,
        KeyCode::F3 => 0x72,
        KeyCode::F4 => 0x73,
        KeyCode::F5 => 0x74,
        KeyCode::F6 => 0x75,
        KeyCode::F7 => 0x76,
        KeyCode::F8 => 0x77,
        KeyCode::F9 => 0x78,
        KeyCode::F10 => 0x79,
        KeyCode::F11 => 0x7A,
        KeyCode::F12 => 0x7B,
        KeyCode::Semicolon => 0xBA,
        KeyCode::Equal => 0xBB,
        KeyCode::Comma => 0xBC,
        KeyCode::Minus => 0xBD,
        KeyCode::Period => 0xBE,
        KeyCode::Slash => 0xBF,
        KeyCode::Backquote => 0xC0,
        KeyCode::BracketLeft => 0xDB,
        KeyCode::Backslash => 0xDC,
        KeyCode::BracketRight => 0xDD,
        KeyCode::Quote => 0xDE,
        KeyCode::ShiftLeft => 0xA0,
        KeyCode::ShiftRight => 0xA1,
        KeyCode::ControlLeft => 0xA2,
        KeyCode::ControlRight => 0xA3,
        KeyCode::AltLeft => 0xA4,
        KeyCode::AltRight => 0xA5,
        KeyCode::SuperLeft => 0x5B,
        KeyCode::SuperRight => 0x5C,
        KeyCode::CapsLock => 0x14,
        KeyCode::NumLock => 0x90,
        KeyCode::PrintScreen => 0x2C,
        KeyCode::NumpadEnter => 0x0D,
        _ => 0,
    };
    vk
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(r: u8, g: u8, b: u8) -> u32 {
        ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }

    fn classify(r: u8, g: u8, b: u8) -> u8 {
        let pixel = rgb(r, g, b);
        let (h, s, l) = rgb_to_hsl255(unpack_r(pixel), unpack_g(pixel), unpack_b(pixel));
        if l > 50.0 {
            if s < 20.0 {
                8
            } else {
                nearest_piece_hue(h)
            }
        } else {
            0
        }
    }

    #[test]
    fn guideline_piece_colors() {
        assert_eq!(classify(0x00, 0xD0, 0xFF), 1); // I
        assert_eq!(classify(0x40, 0x80, 0xFF), 2); // J
        assert_eq!(classify(0x40, 0xD0, 0x40), 3); // S
        assert_eq!(classify(0xFF, 0xE0, 0x20), 4); // O
        assert_eq!(classify(0xFF, 0x40, 0x20), 5); // Z
        assert_eq!(classify(0xFF, 0x80, 0x20), 6); // L
        assert_eq!(classify(0xA0, 0x40, 0xF0), 7); // T
    }

    #[test]
    fn guideline_pure_colors() {
        assert_eq!(classify(0, 255, 255), 1); // cyan I
        assert_eq!(classify(0, 0, 255), 2); // blue J
        assert_eq!(classify(0, 255, 0), 3); // green S
        assert_eq!(classify(255, 255, 0), 4); // yellow O
        assert_eq!(classify(255, 0, 0), 5); // red Z
        assert_eq!(classify(255, 165, 0), 6); // orange L
        assert_eq!(classify(160, 32, 240), 7); // purple T
    }

    #[test]
    fn jstris_like_shades() {
        assert_eq!(classify(105, 88, 186), 2); // dark blue J, was purple
        assert_eq!(classify(181, 57, 65), 5); // muted red Z, was gray
        assert_eq!(classify(179, 161, 55), 4); // muted yellow O
        assert_eq!(classify(64, 188, 141), 1); // teal-green S leans cyan
    }

    #[test]
    fn empty_and_garbage() {
        assert_eq!(classify(10, 10, 10), 0); // dark outline -> empty
        assert_eq!(classify(0, 0, 0), 0); // black -> empty
        assert_eq!(classify(128, 128, 128), 8); // garbage gray
        assert_eq!(classify(204, 204, 204), 8); // light gray
        assert_eq!(classify(255, 255, 255), 8); // white
    }

    #[test]
    fn reset_clears_top_out_in_pc_mode() {
        let mut app = super::App::new();
        loop {
            if matches!(app.game.mode, super::Mode::PC) {
                break;
            }
            app.switch_mode();
        }
        app.game.lose_game();
        assert!(app.game.lost);
        app.keybind_action(super::KEY_RESET);
        assert!(!app.game.lost);
        app.step();
        assert!(!app.game.lost);
    }

    #[test]
    fn input_box_receives_each_typed_char_once() {
        let mut app = App::new();
        app.open_queue_input();
        let prefill = app.input_box.as_ref().unwrap().text.clone();
        app.handle_key_down(0x49, false, false, false);
        if app.input_box.is_some() {
            app.input_box_text("I");
        }
        let expected = format!("{prefill}I");
        assert_eq!(app.input_box.as_ref().map(|b| b.text.as_str()), Some(expected.as_str()));
        app.input_box_key(vk::VK_RETURN, false);
        assert!(app.input_box.is_none());
        let last = app.game.bag.last().copied();
        assert_eq!(last, Some(piece_get_id('I')));
        assert_eq!(app.game.bag.len(), prefill.chars().count() + 1);
    }

    #[test]
    fn nested_buttons_win_hover_over_their_container() {
        let mut app = App::new();
        let hold = app.button_rect(HOLDBUTTON);
        app.mouse = [hold[0] + 60.0, hold[1] + 10.0];
        app.update_hover();
        assert!(app.buttons[HOLDDELETE][0]);
        assert!(!app.buttons[HOLDBUTTON][0]);

        let next = app.button_rect(NEXTBUTTON);
        app.mouse = [next[0] + 60.0, next[1] + 10.0];
        app.update_hover();
        assert!(app.buttons[SHUFBUTTON][0]);
        assert!(!app.buttons[NEXTBUTTON][0]);

        app.mouse = [hold[0] + 10.0, hold[1] + 60.0];
        app.update_hover();
        assert!(app.buttons[HOLDBUTTON][0]);
        assert!(!app.buttons[HOLDDELETE][0]);
    }
}
