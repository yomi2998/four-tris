// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

use crate::game::Game;

#[derive(Clone)]
pub struct SaveState {
    pub damage: u32,
    pub lines: u32,
    pub moves: u32,
    pub piece_h: i32,
    pub swapped: bool,
    pub btb: bool,
    pub clear_combo: u32,
    pub bag_seed: u32,
    pub grid: Vec<Vec<u8>>,
    pub bag: Vec<i32>,
}

pub struct UndoRing {
    pub slots: Vec<Option<SaveState>>,
    pub index: usize,
    pub undo_max: usize,
    pub redo_max: usize,
}

impl UndoRing {
    pub fn new(size: usize) -> UndoRing {
        UndoRing { slots: vec![None; size], index: 0, undo_max: 0, redo_max: 0 }
    }
}

pub fn save_state(game: &Game) -> SaveState {
    SaveState {
        damage: game.damage,
        lines: game.lines,
        moves: game.moves,
        piece_h: game.piece_h,
        swapped: game.swapped,
        btb: game.btb,
        clear_combo: game.clear_combo,
        bag_seed: game.bag_seed,
        grid: game.grid.clone(),
        bag: game.bag.clone(),
    }
}

pub fn load_state(game: &mut Game, state: &SaveState) {
    game.stats_reset();
    game.damage = state.damage;
    game.lines = state.lines;
    game.moves = state.moves;
    game.piece_h = state.piece_h;
    game.swapped = state.swapped;
    game.btb = state.btb;
    game.clear_combo = state.clear_combo;
    game.bag_seed = state.bag_seed;
    game.grid = state.grid.clone();
    game.bag = state.bag.clone();
    game.piece_reset();

    game.changed = true;
    game.lost = false;
    game.perfect = false;
    game.attack_text.clear();
}

pub fn new_undo(game: &mut Game, ring: &mut UndoRing) {
    let len = ring.slots.len();
    ring.slots[ring.index] = Some(save_state(game));
    ring.redo_max = 0;
    ring.undo_max += 1;
    ring.index = (ring.index + 1) % len;
    if ring.undo_max > len {
        ring.undo_max = len;
    }
}

fn new_redo(game: &mut Game, ring: &mut UndoRing) {
    new_undo(game, ring);
    let len = ring.slots.len();
    ring.index = (ring.index + len - 1) % len;
    ring.undo_max -= 1;
}

pub fn undo(game: &mut Game, ring: &mut UndoRing) {
    if ring.undo_max == 0 {
        return;
    }
    if ring.redo_max == 0 {
        new_redo(game, ring);
    }
    let len = ring.slots.len();
    ring.undo_max -= 1;
    ring.redo_max += 1;
    ring.index = (ring.index + len - 1) % len;
    if let Some(state) = ring.slots[ring.index].as_ref() {
        load_state(game, state);
    }
}

pub fn redo(game: &mut Game, ring: &mut UndoRing) {
    if ring.redo_max == 0 {
        return;
    }
    let len = ring.slots.len();
    ring.undo_max += 1;
    ring.redo_max -= 1;
    ring.index = (ring.index + 1) % len;
    if let Some(state) = ring.slots[ring.index].as_ref() {
        load_state(game, state);
    }
}

fn b64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { TABLE[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[n as usize & 63] as char } else { '=' });
    }
    out
}

fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let mut vals = Vec::with_capacity(s.len());
    for c in s.chars() {
        let v = match c {
            'A'..='Z' => c as u32 - 'A' as u32,
            'a'..='z' => c as u32 - 'a' as u32 + 26,
            '0'..='9' => c as u32 - '0' as u32 + 52,
            '+' => 62,
            '/' => 63,
            '=' | '\r' | '\n' | ' ' | '\t' => continue,
            _ => return None,
        };
        vals.push(v as u8);
    }
    let mut out = Vec::with_capacity(vals.len() * 3 / 4);
    for chunk in vals.chunks(4) {
        if chunk.len() < 2 {
            break;
        }
        let n = ((chunk[0] as u32) << 18)
            | ((chunk[1] as u32) << 12)
            | ((chunk.get(2).copied().unwrap_or(0) as u32) << 6)
            | (chunk.get(3).copied().unwrap_or(0) as u32);
        out.push((n >> 16) as u8);
        if chunk.len() > 2 {
            out.push((n >> 8) as u8);
        }
        if chunk.len() > 3 {
            out.push(n as u8);
        }
    }
    Some(out)
}

fn nibbles_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..(i + 2).min(hex.len())], 16).unwrap_or(0))
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

fn queue_encode(game: &Game) -> String {
    let mut hex = format!("{:04X}", game.bag_seed & 0xFFFF);
    hex.push_str(&format!("{:X}", game.piece_h & 0xF));
    for &p in &game.bag {
        hex.push_str(&format!("{:X}", p & 0xF));
    }
    if hex.len() % 2 != 0 {
        hex.push('E');
    }
    b64_encode(&nibbles_to_bytes(&hex))
}

fn queue_decode(data: &str) -> Option<(u32, i32, Vec<i32>)> {
    let bytes = b64_decode(data)?;
    let s = bytes_to_hex(&bytes);
    if s.len() < 5 {
        return None;
    }
    let seed = u32::from_str_radix(&s[0..4], 16).ok()?;
    let mut hold = i32::from_str_radix(&s[4..5], 16).ok()?;
    let queue: Vec<i32> = s[5..]
        .chars()
        .filter_map(|c| u32::from_str_radix(&c.to_string(), 16).ok())
        .map(|v| if v == 15 { -1 } else if v > 7 { 7 } else { v as i32 })
        .collect();
    if hold == 15 {
        hold = -1;
    } else if hold > 7 {
        hold = 7;
    }
    Some((seed, hold, queue))
}

fn board_encode(game: &Game) -> String {
    let w = game.grid_w();
    let h = game.grid_h();
    let mut hex = format!("{w:02X}{h:02X}");
    for j in 0..h {
        for col in &game.grid {
            hex.push_str(&format!("{:X}", col[j] & 0xF));
        }
    }
    if hex.len() % 2 != 0 {
        hex.push('0');
    }
    b64_encode(&crate::lznt::compress_store(&nibbles_to_bytes(&hex)))
}

fn board_decode(game: &Game, data: &str) -> Option<Vec<Vec<u8>>> {
    let bytes = b64_decode(data)?;
    let raw = crate::lznt::decompress(&bytes)?;
    let s = bytes_to_hex(&raw);
    if s.len() < 4 {
        return None;
    }
    let w = usize::from_str_radix(&s[0..2], 16).ok()?;
    let h = usize::from_str_radix(&s[2..4], 16).ok()?;
    if w != game.grid_w() || h != game.grid_h() {
        return None;
    }
    if s.len() < 4 + w * h {
        return None;
    }
    let mut board = vec![vec![0u8; h]; w];
    let mut k = 4;
    for j in 0..h {
        for col in &mut board {
            let v = u32::from_str_radix(&s[k..k + 1], 16).ok()?;
            col[j] = v.min(8) as u8;
            k += 1;
        }
    }
    Some(board)
}

pub fn state_encode(game: &Game) -> String {
    format!("[{}[{}", queue_encode(game), board_encode(game))
}

pub fn state_decode(game: &mut Game, data: &str) -> bool {
    let parts: Vec<&str> = data.splitn(3, '[').collect();
    let comment = parts.first().copied().unwrap_or("").trim().to_string();
    let queue_data = parts.get(1).copied().unwrap_or("").trim_end().to_string();
    let board_data = parts.get(2).copied().unwrap_or("").trim_end().to_string();

    let queue = match queue_decode(&queue_data) {
        Some(q) => q,
        None => return false,
    };
    let board = match board_decode(game, &board_data) {
        Some(b) => b,
        None => return false,
    };

    let comment = comment.replace('\r', "").replace('\n', "");
    let segments: Vec<&str> = comment.split('|').collect();
    let (title, comment) = if segments.len() > 1 {
        (
            segments[0].chars().take(19).collect::<String>(),
            segments[1].chars().take(33).collect::<String>(),
        )
    } else if !segments[0].is_empty() {
        let text = segments[0];
        (
            String::new(),
            format!(
                "{}\n{}",
                text.chars().take(33).collect::<String>(),
                text.chars().skip(33).take(33).collect::<String>()
            ),
        )
    } else {
        (String::new(), String::new())
    };

    game.stats_reset();
    game.piece_reset();
    game.bag_seed = queue.0;
    game.piece_h = queue.1;
    game.bag = queue.2;
    game.grid = board;
    game.changed = true;
    game.pending_comment = Some((title, comment, 2000));

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, GRID_H, PIECE_NAMES};

    fn test_game() -> Game {
        Game::new(10, 20, 0, vec![1], String::new(), false, true, false)
    }

    #[test]
    fn b64_roundtrip() {
        let data: Vec<u8> = (0..=255u8).cycle().take(1000).collect();
        assert_eq!(b64_decode(&b64_encode(&data)).unwrap(), data);
    }

    #[test]
    fn queue_roundtrip() {
        let mut game = test_game();
        game.bag_fill();
        game.bag_seed = 0xBEEF;
        game.piece_h = 3;
        let expected_bag = game.bag.clone();
        let encoded = queue_encode(&game);
        let (seed, hold, queue) = queue_decode(&encoded).unwrap();
        assert_eq!(seed, 0xBEEF);
        assert_eq!(hold, 3);
        assert_eq!(queue, expected_bag);
    }

    #[test]
    fn board_roundtrip() {
        let mut game = test_game();
        game.grid[3][7] = 5;
        game.grid[9][23] = 8;
        let encoded = board_encode(&game);
        let board = board_decode(&game, &encoded).unwrap();
        assert_eq!(board, game.grid);
    }

    #[test]
    fn state_roundtrip() {
        let mut game = test_game();
        game.clear_board();
        game.grid[2][10] = 4;
        let text = state_encode(&game);
        let mut other = test_game();
        assert!(state_decode(&mut other, &text));
        assert_eq!(other.grid[2][10], 4);
        assert_eq!(other.bag_seed, game.bag_seed);
    }

    #[test]
    fn undo_redo_roundtrip() {
        let mut game = test_game();
        let mut ring = UndoRing::new(100);
        game.clear_board();
        game.grid[0][10] = 1;
        new_undo(&mut game, &mut ring);
        game.grid[0][10] = 2;
        new_undo(&mut game, &mut ring);
        game.grid[0][10] = 3;
        undo(&mut game, &mut ring);
        assert_eq!(game.grid[0][10], 2);
        undo(&mut game, &mut ring);
        assert_eq!(game.grid[0][10], 1);
        redo(&mut game, &mut ring);
        assert_eq!(game.grid[0][10], 2);
        redo(&mut game, &mut ring);
        assert_eq!(game.grid[0][10], 3);
        undo(&mut game, &mut ring);
        undo(&mut game, &mut ring);
        redo(&mut game, &mut ring);
        redo(&mut game, &mut ring);
        assert_eq!(game.grid[0][10], 3);
    }

    #[test]
    fn grid_dims() {
        let game = test_game();
        assert_eq!(game.grid_w(), 10);
        assert_eq!(game.grid_h(), 20 + GRID_H);
        assert_eq!(PIECE_NAMES[6], "T");
    }
}
