// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

use crate::app::*;
use crate::config::Skin;
use crate::game::{piece_get_shape, GRID_H, TEXTURE_MAP};
use crate::render::{pack_rgb, Align, Bitmap, ICON_DELETE, ICON_ERASER, ICON_MIRROR, ICON_REDO, ICON_SHUFFLE, ICON_SNAP, ICON_UNDO};

pub const FONT9: f64 = 14.0;
pub const FONT10: f64 = 15.0;
pub const FONT20: f64 = 30.0;
pub const FONT30: f64 = 52.0;
pub const FONT50: f64 = 75.0;

const BUTTON_TEXT: [&str; 3] = ["TRAINING  MODE  ", "        SETTINGS", "COPY BOARD"];

fn mode_button_text(mode: crate::game::Mode) -> &'static str {
    match mode {
        crate::game::Mode::Training => "TRAINING  MODE  ",
        crate::game::Mode::Cheese => " CHEESE   MODE  ",
        crate::game::Mode::Four => "  FOUR    MODE  ",
        crate::game::Mode::Master => " MASTER   MODE  ",
        crate::game::Mode::PC => "   PC     MODE  ",
    }
}
const SEPARATORS: [(&str, f64); 4] =
    [("COLORS", 5.0), ("KEYBINDS", 195.0), ("GAMEPLAY", 490.0), ("SOUND", 770.0)];

const SETTINGS_LABELS: [&str; 26] = [
    "MOVE LEFT",
    "MOVE RIGHT",
    "ROTATE CCW",
    "ROTATE 180",
    "ROTATE CW",
    "HOLD PIECE",
    "SOFT DROP",
    "HARD DROP",
    "RESET GAME",
    "HLIGHT MODE",
    "<",
    "SKIN",
    ">",
    "USE CUSTOM TEXTURES",
    "ARR (ms)",
    "DAS (ms)",
    "DAS CANCELLATION",
    "SHOW GHOST PIECE",
    "RESET SIZE",
    "VOLUME",
    "<",
    "TEXTURE",
    ">",
    "HLIGHT CLEAR",
    "SOFTDROP SPEED (ms)",
    "SOFTDROP DELAY (ms)",
];

impl App {
    pub fn color(&self, index: usize) -> u32 {
        self.skin.colors[index]
    }

    pub fn draw_frame(&mut self) {
        if let Some((start, _old)) = &self.transition {
            let now = self.now_ms();
            let elapsed = now - *start;
            if elapsed >= 150.0 {
                self.transition = None;
            }
        }

        if self.snap.is_some() {
            self.draw_snap();
        } else if self.screen == Screen::Settings {
            self.draw_settings();
        } else {
            self.draw_game();
        }

        if let Some((kind, text)) = self.input_box.as_ref().map(|b| (b.kind, b.text.clone())) {
            self.draw_input_box(kind, &text);
        }

        if let Some((start, old)) = self.transition.take() {
            let now = self.now_ms();
            let elapsed = (now - start).min(150.0);
            let y = (self.wsize[1] * (1.0 - elapsed / 150.0)).floor();
            let snapshot = self.canvas.snapshot();
            let scale = self.total_scale();
            self.canvas.blit_bitmap(&old, 0, 0);
            let top = (y * scale).floor() as i32;
            self.canvas.set_clip((0, top, i32::MAX, i32::MAX));
            self.canvas.restore_from(&snapshot);
            self.canvas.set_clip((0, 0, i32::MAX, i32::MAX));
            self.transition = Some((start, old));
            self.redraw_pending = true;
        }
    }

    pub fn draw_game(&mut self) {
        self.game.changed = false;

        self.canvas.clear(self.color(Skin::BKG));
        self.canvas.fill_rect(0.0, 0.0, self.wsize[0], self.wsize[1], self.color(Skin::BKG));
        self.draw_grid();
        if self.cfg.ghost_piece {
            self.draw_guide();
        }
        self.draw_piece();
        self.draw_highlight();

        self.draw_next();
        self.draw_hold();
        self.draw_score();

        self.draw_snap_button();
        self.draw_mirror_button();
        self.draw_undo_button();
        self.draw_buttons();

        if self.game.highlight_mode {
            self.draw_highlight_buttons();
        } else {
            self.draw_paint_buttons();
        }

        self.draw_checkboxes();
        self.draw_attack();
        self.draw_combo();

        self.draw_lose();
        self.draw_perfect();
        self.draw_comment();
    }

    fn draw_grid(&mut self) {
        let [gx, gy, gw, gh] = self.gbounds;
        let cs = self.cfg.cell_size as f64;
        self.canvas
            .fill_rect(gx, gy - cs * 2.0, gw - 1.0, gh + cs * 2.0 - 1.0, self.color(Skin::F));

        let grid_h = GRID_H;
        for j in grid_h - 2..=grid_h {
            for i in 0..self.game.grid_w() {
                let v = self.game.grid[i][j];
                if v != 0 {
                    self.draw_block(i as i32, j as i32, v);
                }
            }
        }
        for j in grid_h..self.game.grid_h() {
            for i in 0..self.game.grid_w() {
                self.draw_block(i as i32, j as i32, self.game.grid[i][j]);
            }
        }
    }

    fn draw_piece(&mut self) {
        let piece = self.game.bag_get_piece();
        let shape = piece_get_shape(piece, self.game.piece_a);
        let (px, py) = (self.game.piece_x, self.game.piece_y);
        for i in 0..4 {
            for j in 0..4 {
                if shape[i][j] == 0 {
                    continue;
                }
                self.draw_block(px + i as i32, py + j as i32, (piece + 1) as u8);
            }
        }
    }

    fn draw_guide(&mut self) {
        let piece = self.game.bag_get_piece();
        let shape = piece_get_shape(piece, self.game.piece_a);
        let (px, pa) = (self.game.piece_x, self.game.piece_a);
        let mut y = self.game.piece_y;
        loop {
            if !self.game.piece_fits(piece, pa, px, y + 1) {
                break;
            }
            y += 1;
        }
        for i in 0..4 {
            for j in 0..4 {
                if shape[i][j] == 0 {
                    continue;
                }
                self.draw_block(px + i as i32, y + j as i32, 9);
            }
        }
    }

    fn draw_block(&mut self, i: i32, j: i32, k: u8) {
        let cs = self.cfg.cell_size as f64;
        let x = self.grid_x + i as f64 * cs;
        let y = self.grid_y + (j - GRID_H as i32) as f64 * cs;
        let s = cs - self.skin.style as f64;
        self.draw_mini_block(x, y, s, k as usize);
    }

    fn draw_mini_block(&mut self, x: f64, y: f64, s: f64, k: usize) {
        if self.cfg.render_textures {
            if let (Some(tex), true) = (&self.texture, self.texture_tile > 0) {
                let tile = TEXTURE_MAP[k];
                let off = self.texture_tile * tile;
                self.canvas.texture_block(x, y, s, s, tex, off, 0, self.texture_tile, self.texture_tile);
                return;
            }
        }
        if k == 9 {
            let piece = self.game.bag_get_piece();
            let color = self.color(((piece + 1) as usize).min(Skin::T));
            self.canvas.frame_rect(x, y, s - 1.0, s - 1.0, color);
        } else {
            self.canvas.fill_rect(x, y, s, s, self.color(k));
        }
    }

    fn draw_highlight(&mut self) {
        let w = self.game.grid_w();
        let h = self.game.grid_h();
        for j in GRID_H..h {
            let full = (0..w).all(|i| self.game.grid[i][j] != 0);
            if full {
                self.draw_hrow(j);
            }
        }

        for i in 0..w {
            for j in (GRID_H - 2)..h {
                let v = self.game.hlight[i][j];
                if v == 0 {
                    continue;
                }
                let color = self.color(v as usize);
                if !(self.game.block_is_neighbour(i as i32 - 1, j as i32, v)) {
                    self.draw_vedge(i as i32, j as i32, 0, color);
                }
                if !(self.game.block_is_neighbour(i as i32 + 1, j as i32, v)) {
                    self.draw_vedge(i as i32, j as i32, 1, color);
                }
                if !(self.game.block_is_neighbour(i as i32, j as i32 - 1, v)) {
                    self.draw_hedge(i as i32, j as i32, 0, color);
                }
                if !(self.game.block_is_neighbour(i as i32, j as i32 + 1, v)) {
                    self.draw_hedge(i as i32, j as i32, 1, color);
                }
            }
        }
    }

    fn draw_hedge(&mut self, x: i32, y: i32, edge_type: i32, color: u32) {
        let s = 3.0;
        let cs = self.cfg.cell_size as f64;
        let px = self.grid_x + x as f64 * cs;
        let py = self.grid_y + (y - GRID_H as i32) as f64 * cs - s + edge_type as f64 * (cs + s - 1.0);
        self.canvas.fill_rect(px, py, cs, s, color);
    }

    fn draw_vedge(&mut self, x: i32, y: i32, edge_type: i32, color: u32) {
        let s = 3.0;
        let cs = self.cfg.cell_size as f64;
        let px = self.grid_x + x as f64 * cs - s + edge_type as f64 * (cs + s - 1.0);
        let py = self.grid_y + (y - GRID_H as i32) as f64 * cs;
        self.canvas.fill_rect(px, py, s, cs, color);
    }

    fn draw_hrow(&mut self, row: usize) {
        let [gx, gy, gw, gh] = self.gbounds;
        let cs = self.cfg.cell_size as f64;
        let y = gy + (row - GRID_H) as f64 * cs;
        self.canvas
            .fill_rect_alpha(gx, y, gw, cs, self.color(Skin::BKG), 96);
        let _ = gh;
    }

    fn draw_next(&mut self) {
        let size = 14.0;
        let distance = 2.7857;
        let b = self.button_rect(NEXTBUTTON);

        self.canvas
            .fill_rect(b[0], b[1], b[2], b[3], self.color(Skin::E));
        if self.buttons[NEXTBUTTON][0] {
            self.canvas.frame_rect(b[0], b[1], b[2], b[3], self.color(Skin::REV));
        }
        self.canvas.draw_text(
            "NEXT",
            (b[0] + 12.0, b[1] + 10.0, 55.0, 20.0),
            Align::Left,
            FONT10,
            self.color(Skin::TXT),
            None,
        );

        let sep = self.game.bag_get_separator();
        let s_pos = *sep.get(1).unwrap_or(&0);

        for k in 1..=6 {
            let Some(&piece) = self.game.bag.get(k) else { break };
            if piece == -1 {
                break;
            }

            if k < 6 {
                let shape = piece_get_shape(piece, 0);
                let x = if piece != 0 && piece != 3 {
                    b[0] + 10.0 + size / 2.0
                } else {
                    b[0] + 10.0
                };
                let mut y = if piece == 0 { b[1] - size / 2.0 } else { b[1] };
                y += size * distance * k as f64;

                for i in 0..4 {
                    for j in 0..4 {
                        if shape[i][j] == 0 {
                            continue;
                        }
                        self.draw_mini_block(
                            x + i as f64 * size,
                            y + j as f64 * size,
                            size - self.skin.style as f64,
                            (piece + 1) as usize,
                        );
                    }
                }
            }

            if k == s_pos {
                self.canvas.fill_rect(
                    b[0] + 5.0,
                    b[1] + size * distance * k as f64 - size / 2.0,
                    b[2] - 10.0,
                    1.0,
                    self.color(Skin::REV),
                );
            }
        }

        let b = self.button_rect(SHUFBUTTON);
        if self.buttons[SHUFBUTTON][0] {
            self.canvas.frame_rect(b[0], b[1], b[2], b[3], self.color(Skin::REV));
        }
        if let Some(icons) = &self.icons {
            let icons = icons.clone();
            self.canvas.xor_blit(b[0] + 1.0, b[1] + 1.0, &icons, ICON_SHUFFLE.0, ICON_SHUFFLE.1, ICON_SHUFFLE.2, ICON_SHUFFLE.3);
        }
    }

    fn draw_hold(&mut self) {
        let size = 14.0;
        let b = self.button_rect(HOLDBUTTON);

        self.canvas
            .fill_rect(b[0], b[1], b[2], b[3], self.color(Skin::E));
        if self.buttons[HOLDBUTTON][0] {
            self.canvas.frame_rect(b[0], b[1], b[2], b[3], self.color(Skin::REV));
        }
        self.canvas.draw_text(
            "HOLD",
            (b[0] + 12.0, b[1] + 10.0, 55.0, 20.0),
            Align::Left,
            FONT10,
            self.color(Skin::TXT),
            None,
        );

        if self.game.piece_h != -1 {
            let shape = piece_get_shape(self.game.piece_h, 0);
            for i in 0..4 {
                for j in 0..4 {
                    if shape[i][j] == 0 {
                        continue;
                    }
                    self.draw_mini_block(
                        b[0] + 10.0 + i as f64 * size,
                        b[1] + 35.0 + j as f64 * size,
                        size - self.skin.style as f64,
                        (self.game.piece_h + 1) as usize,
                    );
                }
            }
        }

        if self.game.swapped {
            self.canvas
                .fill_rect_alpha(b[0], b[1], b[2], b[3], self.color(Skin::E), 128);
        }

        if self.game.piece_h != -1 {
            let b = self.button_rect(HOLDDELETE);
            if let Some(icons) = &self.icons {
                let icons = icons.clone();
                self.canvas.xor_blit(b[0] + 3.0, b[1] + 3.0, &icons, ICON_DELETE.0, ICON_DELETE.1, ICON_DELETE.2, ICON_DELETE.3);
            }
            if self.buttons[HOLDDELETE][0] {
                self.canvas.frame_rect(b[0], b[1], b[2], b[3], self.color(Skin::REV));
            }
        }
    }

    fn draw_score(&mut self) {
        let m = self.game.moves.max(1) as f64;
        let app = self.game.damage as f64 / m + 1e-8;
        let app_text = format_number_short(app);
        let app_text = if app < 1e-6 { "0".to_string() } else { app_text };

        let x = 10.0;
        self.canvas
            .fill_rect(x, 10.0, 75.0, 140.0, self.color(Skin::BOX));

        let labels = [
            ("CLEAR", format!("{:06}", self.game.lines.min(999999))),
            ("ATTACK", format!("{:06}", self.game.damage.min(999999))),
            ("PIECES", format!("{:06}", self.game.moves.min(999999))),
            ("APP", format!("{:0>6}", app_text)),
        ];
        for (i, (label, value)) in labels.iter().enumerate() {
            let y = 20.0 + i as f64 * 30.0;
            self.canvas.draw_text(
                label,
                (x + 12.0, y, 55.0, 15.0),
                Align::Left,
                FONT10,
                self.color(Skin::TXT),
                None,
            );
            self.canvas.draw_text(
                value,
                (x + 12.0, y + 12.0, 55.0, 20.0),
                Align::Left,
                FONT10,
                self.color(Skin::TXT),
                None,
            );
        }
    }

    fn draw_buttons(&mut self) {
        for (i, text) in BUTTON_TEXT.iter().enumerate() {
            let text = if i == MODEBUTTON {
                mode_button_text(self.game.mode)
            } else {
                text
            };
            let b = self.button_rect(i);
            self.canvas
                .fill_rect(b[0], b[1], b[2], b[3], self.color(Skin::BOX));
            if self.buttons[i][0] {
                self.canvas.frame_rect(b[0], b[1], b[2], b[3], self.color(Skin::TXT));
            }
            let half = text.len() / 2;
            let (a, _c) = if half % 2 == 1 { (6.0, 6.0) } else { (9.0, 9.0) };
            self.canvas.draw_text(
                &text[..half],
                (b[0] + a, b[1] + 5.0, 65.0, 11.0),
                Align::Left,
                FONT9,
                self.color(Skin::TXT),
                None,
            );
            self.canvas.draw_text(
                &text[half..],
                (b[0] + a, b[1] + 16.0, 65.0, 11.0),
                Align::Left,
                FONT9,
                self.color(Skin::TXT),
                None,
            );
        }
    }

    fn draw_snap_button(&mut self) {
        let b = self.button_rect(SNAPBUTTON);
        self.canvas
            .fill_rect(b[0], b[1], b[2], b[3], self.color(Skin::BOX));
        if self.buttons[SNAPBUTTON][0] {
            self.canvas.frame_rect(b[0], b[1], b[2], b[3], self.color(Skin::TXT));
        }
        if let Some(icons) = &self.icons {
            let icons = icons.clone();
            self.canvas.xor_blit(b[0] + 8.0, b[1] + 3.0, &icons, ICON_SNAP.0, ICON_SNAP.1, ICON_SNAP.2, ICON_SNAP.3);
        }
    }

    fn draw_mirror_button(&mut self) {
        let b = self.button_rect(MIRRBUTTON);
        self.canvas
            .fill_rect(b[0], b[1], b[2], b[3], self.color(Skin::BOX));
        if self.buttons[MIRRBUTTON][0] {
            self.canvas.frame_rect(b[0], b[1], b[2], b[3], self.color(Skin::TXT));
        }
        if let Some(icons) = &self.icons {
            let icons = icons.clone();
            self.canvas.xor_blit(b[0] + 8.0, b[1] + 3.0, &icons, ICON_MIRROR.0, ICON_MIRROR.1, ICON_MIRROR.2, ICON_MIRROR.3);
        }
    }

    fn draw_undo_button(&mut self) {
        let x = self.wsize[0] - 85.0;
        let y = 260.0;

        self.canvas.fill_rect(x, y, 35.0, 35.0, self.color(Skin::BOX));
        self.canvas.fill_rect(x + 40.0, y, 35.0, 35.0, self.color(Skin::BOX));
        if self.buttons[UNDOBUTTON][0] {
            self.canvas.frame_rect(x, y, 35.0, 35.0, self.color(Skin::TXT));
        }
        if self.buttons[REDOBUTTON][0] {
            self.canvas.frame_rect(x + 40.0, y, 35.0, 35.0, self.color(Skin::TXT));
        }

        if let Some(icons) = &self.icons {
            let icons = icons.clone();
            self.canvas.xor_blit(x + 10.0, y + 12.0, &icons, ICON_UNDO.0, ICON_UNDO.1, ICON_UNDO.2, ICON_UNDO.3);
            self.canvas.xor_blit(x + 50.0, y + 12.0, &icons, ICON_REDO.0, ICON_REDO.1, ICON_REDO.2, ICON_REDO.3);
        }

        if self.ring.undo_max == 0 {
            self.canvas
                .fill_rect_alpha(x, y, 36.0, 36.0, self.color(Skin::BOX), 190);
        }
        if self.ring.redo_max == 0 {
            self.canvas
                .fill_rect_alpha(x + 40.0, y, 36.0, 36.0, self.color(Skin::BOX), 190);
        }
    }

    fn draw_paint_buttons(&mut self) {
        let ar = self.wsize[0] - 85.0;
        let at = 10.0;

        self.canvas
            .fill_rect(ar, at + 295.0, 75.0, 100.0, self.color(Skin::BOX));
        if self.buttons[HILIBUTTON][0] {
            self.canvas
                .frame_rect(ar, at + 295.0, 75.0, 100.0, self.color(Skin::TXT));
        }
        self.canvas.draw_text(
            "COLOR",
            (ar + 12.0, at + 305.0, 55.0, 20.0),
            Align::Left,
            FONT10,
            self.color(Skin::TXT),
            None,
        );

        for i in 0..8 {
            let b = self.paint_rect(i);
            self.canvas
                .fill_rect(b[0], b[1], b[2], b[3], self.color(i + 1));
        }

        self.canvas
            .fill_rect_alpha(ar + 10.0, at + 330.0, 55.0, 55.0, self.color(Skin::BOX), 140);

        for i in 0..8 {
            let b = self.paint_rect(i);
            if self.paint[i][0] {
                self.canvas.frame_rect(b[0], b[1], b[2], b[3], self.color(Skin::TXT));
            }
        }

        if self.edit_color >= 1 && self.edit_color <= 8 {
            let b = self.paint_rect((self.edit_color - 1) as usize);
            self.canvas
                .fill_rect(b[0], b[1], b[2], b[3], self.color(self.edit_color as usize));
            self.canvas.frame_rect(b[0] - 1.0, b[1] - 1.0, b[2] + 2.0, b[3] + 2.0, self.color(Skin::TXT));
        }

        let b = self.paint_rect(8);
        if let Some(icons) = &self.icons {
            let icons = icons.clone();
            self.canvas.xor_blit(b[0] + 1.0, b[1] + 1.0, &icons, ICON_ERASER.0, ICON_ERASER.1, ICON_ERASER.2, ICON_ERASER.3);
        }
    }

    fn draw_highlight_buttons(&mut self) {
        let ar = self.wsize[0] - 85.0;
        let at = 10.0;

        self.canvas
            .fill_rect(ar, at + 295.0, 75.0, 35.0, self.color(Skin::BOX));
        if self.buttons[HILIBUTTON][0] {
            self.canvas
                .frame_rect(ar, at + 295.0, 75.0, 35.0, self.color(Skin::TXT));
        }
        self.canvas.draw_text(
            "H-LIGHT",
            (ar + 12.0, at + 305.0, 55.0, 20.0),
            Align::Left,
            FONT10,
            self.color(Skin::TXT),
            None,
        );

        if self.game.hlight_on {
            self.canvas
                .fill_rect(ar, at + 335.0, 75.0, 35.0, self.color(Skin::BOX));
            if self.buttons[HCLRBUTTON][0] {
                self.canvas
                    .frame_rect(ar, at + 335.0, 75.0, 35.0, self.color(Skin::TXT));
            }
            self.canvas.draw_text(
                " CLEAR",
                (ar + 12.0, at + 345.0, 55.0, 20.0),
                Align::Left,
                FONT10,
                self.color(Skin::TXT),
                None,
            );
        }
    }

    fn draw_checkboxes(&mut self) {
        let b = self.button_rect(HOLDCHECK);
        self.canvas.draw_text(
            "INFINITE",
            (b[0] + 16.0, b[1] + 3.0, 55.0, 15.0),
            Align::Left,
            FONT10,
            self.color(Skin::TXT),
            None,
        );
        self.canvas.frame_rect(b[0] + 2.0, b[1] + 4.0, 11.0, 11.0, self.color(Skin::TXT));
        if self.cfg.infinite_hold {
            self.canvas
                .fill_rect(b[0] + 4.0, b[1] + 6.0, 7.0, 7.0, self.color(Skin::G));
        }

        if self.game.highlight_mode {
            return;
        }

        let b = self.button_rect(ACOLCHECK);
        self.canvas.draw_text(
            "AUTOCOLR",
            (b[0] + 16.0, b[1] + 3.0, 55.0, 15.0),
            Align::Left,
            FONT10,
            self.color(Skin::TXT),
            None,
        );
        self.canvas.frame_rect(b[0] + 2.0, b[1] + 4.0, 11.0, 11.0, self.color(Skin::TXT));
        if self.cfg.auto_color {
            self.canvas
                .fill_rect(b[0] + 4.0, b[1] + 6.0, 7.0, 7.0, self.color(Skin::G));
        }
    }

    fn draw_attack(&mut self) {
        if self.game.attack_text.is_empty() {
            return;
        }
        let x = 10.0;
        let y = 310.0;
        let text = self.game.attack_text.clone();
        let first: String = text.chars().take(6).collect::<String>().trim().to_string();
        let second: String = text.chars().skip(6).collect::<String>().trim().to_string();
        let first = if !self.game.b2b_text.is_empty() {
            format!("B2B {}", first)
        } else {
            first
        };
        self.canvas.draw_text(
            &first,
            (x, y, 75.0, 30.0),
            Align::Center,
            FONT10,
            self.color(Skin::TXT),
            None,
        );
        self.canvas.draw_text(
            &second,
            (x, y + 11.0, 75.0, 30.0),
            Align::Center,
            FONT10,
            self.color(Skin::TXT),
            None,
        );
    }

    fn draw_combo(&mut self) {
        if self.game.clear_combo < 2 {
            return;
        }
        let x = 10.0;
        let y = 260.0;
        self.canvas.draw_text(
            &format!("x{}", self.game.clear_combo - 1),
            (x, y + 10.0, 75.0, 30.0),
            Align::Center,
            FONT20,
            self.color(Skin::TXT),
            None,
        );
    }

    fn draw_lose(&mut self) {
        if !self.game.lost {
            return;
        }
        let [gx, gy, _, gh] = self.gbounds;
        self.canvas.draw_text(
            "TOP OUT",
            (gx + 5.0, gy + gh / 2.0 - 31.0, self.gbounds[2], gh),
            Align::Center,
            FONT50,
            self.color(Skin::F),
            None,
        );
        self.canvas.draw_text(
            "TOP OUT",
            (gx, gy + gh / 2.0 - 36.0, self.gbounds[2], gh),
            Align::Center,
            FONT50,
            self.color(Skin::REV),
            None,
        );
        let hint = format!("PRESS {} TO RESET", crate::vk::vkey_name(self.cfg.keybinds[KEY_RESET]));
        self.canvas.draw_text(
            &hint,
            (gx, gy + gh / 2.0 + 26.0, self.gbounds[2], gh),
            Align::Center,
            FONT20,
            self.color(Skin::TXT),
            None,
        );
    }

    fn draw_perfect(&mut self) {
        if !self.game.perfect {
            return;
        }
        let [gx, gy, _, gh] = self.gbounds;
        self.canvas.draw_text(
            "PERFECT",
            (gx, gy + gh / 2.0 - 1.0, self.gbounds[2], gh),
            Align::Center,
            FONT30,
            self.color(Skin::L),
            None,
        );
        self.canvas.draw_text(
            "CLEAR",
            (gx, gy + gh / 2.0 + 25.0, self.gbounds[2], gh),
            Align::Center,
            FONT50,
            self.color(Skin::TXT),
            None,
        );
    }

    fn draw_comment(&mut self) {
        let Some((start, duration, title, comment, ended)) = self.game.active_comment.clone() else {
            return;
        };
        let now = self.now_ms();
        let timer = now - start;
        let ab = self.wsize[1] - 16.0;
        let ended = ended || timer >= duration as f64;

        if ended {
            self.canvas
                .fill_rect(10.0, ab, self.wsize[0] - 20.0, 20.0, self.color(Skin::BOX));
            self.canvas.draw_text(
                &comment,
                (10.0, ab, self.wsize[0] - 20.0, 20.0),
                Align::Center,
                FONT9,
                self.color(Skin::TXT),
                None,
            );
        } else {
            self.canvas
                .fill_rect(10.0, ab + 10.0, self.wsize[0] - 20.0, 20.0, self.color(Skin::BOX));
        }

        if timer < duration as f64 {
            let t = (duration as f64 - timer) / duration as f64;
            let y = self.wsize[1] / 7.0 * popup(t);
            let mut x = 4.0;

            let ended = ended || t < 0.5;
            self.game.active_comment = Some((start, duration, title.clone(), comment.clone(), ended));

            self.canvas
                .fill_rect(10.0, self.wsize[1] - y, self.wsize[0] - 20.0, y, self.color(Skin::BOX));
            self.canvas
                .frame_rect(10.0, self.wsize[1] - y, self.wsize[0] - 20.0, y + 5.0, self.color(Skin::TXT));

            self.canvas.draw_text(
                &title,
                (10.0, self.wsize[1] - y + x, self.wsize[0] - 20.0, 55.0),
                Align::Center,
                FONT30,
                self.color(Skin::TXT),
                None,
            );

            x += if title.is_empty() { 14.0 } else { 53.0 };

            self.canvas.draw_text(
                &comment,
                (10.0, self.wsize[1] - y + x, self.wsize[0] - 20.0, y),
                Align::Center,
                FONT20,
                self.color(Skin::TXT),
                None,
            );
        } else {
            self.game.active_comment = Some((start, duration, title, comment, true));
        }
    }

    pub fn draw_settings(&mut self) {
        self.game.changed = false;

        self.canvas.clear(self.color(Skin::BKG));
        self.canvas.set_scroll(0.0, self.current_view);
        if self.wsize[0] < 400.0 {
            self.canvas
                .fill_rect(0.0, 0.0, self.wsize[0], SETTINGS_PANELSIZE, self.color(Skin::BKG));
            self.draw_separators();
        } else {
            self.canvas.fill_rect(
                self.wsize[0] / 7.0,
                0.0,
                self.wsize[0] * 5.0 / 7.0,
                SETTINGS_PANELSIZE,
                self.color(Skin::BKG),
            );
            self.draw_separators();
            self.draw_pieces_anim();
        }

        for i in 0..=12 {
            self.draw_settings_button(i);
        }

        self.draw_settings_checkbox(13);
        self.draw_settings_slider(14, self.cfg.arr as f64 / 32.0);
        self.draw_settings_slider(15, self.cfg.das as f64 / 256.0);
        self.draw_settings_checkbox(16);
        self.draw_settings_slider(24, self.cfg.sds as f64 / 32.0);
        self.draw_settings_slider(25, self.cfg.sdd as f64 / 256.0);
        self.draw_settings_checkbox(17);
        self.draw_settings_slider(19, self.cfg.volume as f64 / 100.0);

        for i in 20..=23 {
            self.draw_settings_button(i);
        }
        self.draw_settings_button(18);

        self.canvas.set_scroll(0.0, 0.0);

        if self.capture.is_some() {
            self.draw_key_capture();
        }
    }

    fn draw_separators(&mut self) {
        let bkg = self.color(Skin::BKG);
        let color = pack_rgb(255 - unpack_r(0xFFFFFF ^ bkg), 255 - unpack_g(0xFFFFFF ^ bkg), 255 - unpack_b(0xFFFFFF ^ bkg));
        let text_color = 0xFFFFFF ^ bkg;
        let _ = color;
        for (label, y) in SEPARATORS {
            self.canvas.draw_text(
                label,
                (self.wsize[0] / 7.0 + 5.0, y - 14.0, 300.0, 48.0),
                Align::Left,
                FONT30,
                text_color,
                None,
            );
            self.canvas
                .fill_rect(0.0, y + 34.0, self.wsize[0], 3.0, self.color(Skin::BOX));
        }
    }

    fn draw_settings_button(&mut self, s: usize) {
        let b = self.settings_item_rect(s);
        self.canvas
            .fill_rect(b[0], b[1], b[2], b[3], self.color(Skin::BOX));
        if self.settings_hover[s][0] {
            self.canvas.frame_rect(b[0], b[1], b[2], b[3], self.color(Skin::TXT));
        }

        self.canvas.draw_text(
            SETTINGS_LABELS[s],
            (b[0], b[1] + 5.0, b[2], b[3] / 2.0),
            Align::Center,
            FONT9,
            self.color(Skin::TXT),
            None,
        );
        let value = self.settings_value_str(s);
        self.canvas.draw_text(
            &value,
            (b[0], b[1] + b[3] / 2.0, b[2], b[3] / 2.0),
            Align::Center,
            FONT9,
            self.color(Skin::Z),
            None,
        );
    }

    fn settings_value_str(&self, s: usize) -> String {
        match s {
            0..=9 | 23 => crate::vk::vkey_name(self.cfg.keybinds[settings_to_keybind(s)]).to_string(),
            11 => self.skin.name.clone(),
            21 => self.cfg.texture.clone(),
            13 => crate::ini::bool_str(self.cfg.render_textures).to_string(),
            16 => crate::ini::bool_str(self.cfg.das_cancel).to_string(),
            17 => crate::ini::bool_str(self.cfg.ghost_piece).to_string(),
            14 => format!("{}", self.cfg.arr),
            15 => format!("{}", self.cfg.das),
            24 => format!("{}", self.cfg.sds),
            25 => format!("{}", self.cfg.sdd),
            19 => format!("{}", self.cfg.volume),
            18 => crate::ini::format_num(self.user_scale),
            _ => String::new(),
        }
    }

    fn draw_settings_slider(&mut self, s: usize, v: f64) {
        let b = self.settings_item_rect(s);
        let v = v.clamp(0.0, 1.0);

        self.canvas
            .fill_rect(b[0], b[1], b[2], b[3], self.color(Skin::BOX));
        if self.settings_hover[s][0] {
            self.canvas.frame_rect(b[0], b[1], b[2], b[3], self.color(Skin::TXT));
        }

        self.canvas.draw_text(
            SETTINGS_LABELS[s],
            (b[0], b[1] + 5.0, b[2], b[3] / 2.0),
            Align::Center,
            FONT9,
            self.color(Skin::TXT),
            None,
        );
        let value = self.settings_value_str(s);
        self.canvas.draw_text(
            &value,
            (b[0], b[1] + b[3] / 2.0, b[2], b[3] / 2.0),
            Align::Center,
            FONT9,
            self.color(Skin::Z),
            None,
        );

        self.canvas
            .fill_rect(b[0], b[1] + b[3], b[2], 3.0, self.color(Skin::BKG));
        self.canvas
            .fill_rect(b[0], b[1] + b[3], v * b[2], 3.0, self.color(Skin::Z));
    }

    fn draw_settings_checkbox(&mut self, s: usize) {
        let b = self.settings_item_rect(s);

        self.canvas
            .fill_rect(b[0], b[1], b[2], b[3], self.color(Skin::BKG));
        self.canvas.draw_text(
            SETTINGS_LABELS[s],
            (b[0] + 16.0, b[1] + 3.0, b[2], b[3]),
            Align::Left,
            FONT9,
            self.color(Skin::TXT),
            None,
        );
        self.canvas.frame_rect(b[0] + 2.0, b[1] + 4.0, 11.0, 11.0, self.color(Skin::TXT));
        let checked = match s {
            13 => self.cfg.render_textures,
            16 => self.cfg.das_cancel,
            _ => self.cfg.ghost_piece,
        };
        if checked {
            self.canvas
                .fill_rect(b[0] + 4.0, b[1] + 6.0, 7.0, 7.0, self.color(Skin::G));
        }
    }

    fn draw_pieces_anim(&mut self) {
        let time = self.now_ms();
        let timings = [1130.0, 570.0, 2589.0, 900.0, 783.0, 340.0, 1309.0];
        let size = 14.0;

        self.canvas
            .fill_rect(0.0, 0.0, self.wsize[0] * (1.0 / 7.0), SETTINGS_PANELSIZE, self.color(Skin::E));
        self.canvas.fill_rect(
            self.wsize[0] * (6.0 / 7.0),
            0.0,
            self.wsize[0] * (1.0 / 7.0),
            SETTINGS_PANELSIZE,
            self.color(Skin::E),
        );

        let mut y = 55.0 + self.current_view / self.user_scale.max(0.001);
        for k in 0..7 {
            let angle = ((time / timings[k]).floor() as i64).rem_euclid(4) as i32;
            let shape = piece_get_shape(k as i32, angle);

            for i in 0..4 {
                for j in 0..4 {
                    if shape[i][j] == 0 {
                        continue;
                    }
                    let mut x = 14.0;
                    if k == 0 || k == 3 {
                        x = 8.0;
                    }
                    self.draw_mini_block(x + i as f64 * size, y + j as f64 * size, size - self.skin.style as f64, k + 1);
                    let mut x2 = self.wsize[0] - 55.0;
                    if k == 0 || k == 3 {
                        x2 = self.wsize[0] - 62.0;
                    }
                    self.draw_mini_block(x2 + i as f64 * size, y + j as f64 * size, size - self.skin.style as f64, k + 1);
                }
            }
            y += (self.wsize[1] - 55.0) / 7.0;
        }
    }

    fn draw_key_capture(&mut self) {
        let Some((_, stt)) = self.capture else { return };
        let b = self.settings_item_rect(stt);
        self.canvas
            .fill_rect_alpha(0.0, 0.0, b[0], SETTINGS_PANELSIZE, self.color(Skin::BOX), 128);
        self.canvas
            .fill_rect_alpha(b[0], 0.0, b[2], b[1], self.color(Skin::BOX), 128);
        self.canvas.fill_rect_alpha(
            b[0] + b[2],
            0.0,
            self.wsize[0],
            SETTINGS_PANELSIZE,
            self.color(Skin::BOX),
            128,
        );
        self.canvas.fill_rect_alpha(
            b[0],
            b[1] + b[3],
            b[2],
            SETTINGS_PANELSIZE,
            self.color(Skin::BOX),
            128,
        );
    }

    fn draw_input_box(&mut self, kind: InputKind, text_buf: &str) {
        let w = self.wsize[0];
        let h = self.wsize[1];
        self.canvas
            .fill_rect_alpha(0.0, 0.0, w, h, self.color(Skin::BOX), 128);
        let bw = 250.0_f64.min(w - 20.0);
        let bh = 130.0_f64.min(h - 20.0);
        let bx = (w - bw) / 2.0;
        let by = (h - bh) / 2.0;
        self.canvas
            .fill_rect(bx, by, bw, bh, self.color(Skin::BOX));
        self.canvas.frame_rect(bx, by, bw, bh, self.color(Skin::TXT));
        let title = match kind {
            InputKind::Queue => "Set the queue (TLJZSOI)",
            InputKind::Hold => "Set the hold piece. (TLJZSOI)",
        };
        self.canvas.draw_text(
            title,
            (bx + 10.0, by + 10.0, bw - 20.0, 20.0),
            Align::Left,
            FONT10,
            self.color(Skin::TXT),
            None,
        );
        self.canvas.frame_rect(bx + 10.0, by + 40.0, bw - 20.0, 34.0, self.color(Skin::TXT));
        let mut text = text_buf.to_string();
        text.push('|');
        self.canvas.draw_text(
            &text,
            (bx + 16.0, by + 46.0, bw - 32.0, 26.0),
            Align::Left,
            FONT20,
            self.color(Skin::TXT),
            None,
        );
        self.canvas.draw_text(
            "ENTER: apply   ESC: cancel",
            (bx + 10.0, by + bh - 26.0, bw - 20.0, 16.0),
            Align::Left,
            FONT9,
            self.color(Skin::TXT),
            None,
        );
    }

    fn draw_snap(&mut self) {
        let Some(snap) = &self.snap else { return };
        let saved_scale = self.canvas.scale;
        let saved_off = self.canvas.off;
        self.canvas.scale = 1.0;
        self.canvas.set_off((0, 0));
        let image = snap.image.clone();
        let w = self.canvas.w as f64;
        let h = self.canvas.h as f64;
        self.canvas.blit_bitmap_scaled(&image, 0.0, 0.0, w, h);
        self.canvas.fill_rect_alpha(0.0, 0.0, w, h, 0x000000, 128);

        if let Some((sx, sy)) = snap.drag_start {
            let (cx, cy) = snap.drag_cur;
            let left = sx.min(cx);
            let top = sy.min(cy);
            let bw = (cx - sx).abs();
            let bh = (cy - sy).abs();
            self.canvas
                .fill_rect_alpha(left, top, bw, bh, 0x000000, 0);
            self.draw_snap_region(&image, left, top, bw, bh, w, h);
            self.canvas.frame_rect(left - 1.0, top - 1.0, bw + 2.0, bh + 2.0, 0xFFFFFF);
        } else {
            self.canvas.draw_text(
                "Drag to select the region, ESC to cancel",
                (0.0, 8.0, w, 20.0),
                Align::Center,
                FONT10,
                0xFFFFFF,
                None,
            );
        }
        self.canvas.scale = saved_scale;
        self.canvas.set_off(saved_off);
    }

    fn draw_snap_region(&mut self, image: &Bitmap, left: f64, top: f64, bw: f64, bh: f64, w: f64, h: f64) {
        if image.w == 0 || image.h == 0 || bw <= 0.0 || bh <= 0.0 {
            return;
        }
        let sw = image.w as f64;
        let sh = image.h as f64;
        let x0 = (left / w * sw).floor() as i32;
        let y0 = (top / h * sh).floor() as i32;
        let x1 = ((left + bw) / w * sw).ceil() as i32;
        let y1 = ((top + bh) / h * sh).ceil() as i32;
        let sx0 = x0.clamp(0, image.w as i32 - 1) as usize;
        let sy0 = y0.clamp(0, image.h as i32 - 1) as usize;
        let sx1 = x1.clamp(1, image.w as i32) as usize;
        let sy1 = y1.clamp(1, image.h as i32) as usize;
        let dw = (sx1 - sx0) as f64 / sw * w;
        let dh = (sy1 - sy0) as f64 / sh * h;
        self.canvas.texture_block(left, top, dw, dh, image, sx0, sy0, sx1 - sx0, sy1 - sy0);
    }
}

pub fn popup(x: f64) -> f64 {
    let x = 1.0 - x;
    if x < 1.0 / 5.0 {
        return x * x * 25.0;
    }
    if x < 4.0 / 5.0 {
        return 1.0;
    }
    if x < 1.0 {
        return (1.0 - x) * (1.0 - x) * 25.0;
    }
    0.0
}

pub fn format_number_short(v: f64) -> String {
    let rounded = (v * 10000.0).round() / 10000.0;
    format!("{:.4}", rounded).chars().take(6).collect()
}
