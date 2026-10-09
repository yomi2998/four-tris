// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

pub const GRID_H: usize = 4;

pub const PIECE_NAMES: [&str; 7] = ["I", "J", "S", "O", "Z", "L", "T"];

pub const MIRROR_PIECE: [i32; 7] = [0, 5, 4, 3, 2, 1, 6];
pub const MIRROR_CELL: [u8; 9] = [0, 1, 6, 5, 4, 3, 2, 7, 8];
pub const PC_SIZES: [usize; 7] = [7, 4, 1, 5, 2, 6, 3];

pub const TEXTURE_MAP: [usize; 10] = [9, 4, 5, 3, 2, 0, 1, 6, 7, 8];

type Shape = [[u8; 4]; 4];

const SHAPES: [[Shape; 4]; 8] = [
    [
        [[0, 1, 0, 0], [0, 1, 0, 0], [0, 1, 0, 0], [0, 1, 0, 0]],
        [[0, 0, 0, 0], [1, 1, 1, 1], [0, 0, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 1, 0], [0, 0, 1, 0], [0, 0, 1, 0], [0, 0, 1, 0]],
        [[0, 0, 0, 0], [0, 0, 0, 0], [1, 1, 1, 1], [0, 0, 0, 0]],
    ],
    [
        [[1, 1, 0, 0], [0, 1, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 1, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
        [[0, 1, 0, 0], [0, 1, 0, 0], [0, 1, 1, 0], [0, 0, 0, 0]],
        [[0, 0, 0, 0], [1, 1, 1, 0], [1, 0, 0, 0], [0, 0, 0, 0]],
    ],
    [
        [[0, 1, 0, 0], [1, 1, 0, 0], [1, 0, 0, 0], [0, 0, 0, 0]],
        [[1, 1, 0, 0], [0, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 1, 0], [0, 1, 1, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 0, 0], [1, 1, 0, 0], [0, 1, 1, 0], [0, 0, 0, 0]],
    ],
    [
        [[0, 0, 0, 0], [1, 1, 0, 0], [1, 1, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 0, 0], [1, 1, 0, 0], [1, 1, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 0, 0], [1, 1, 0, 0], [1, 1, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 0, 0], [1, 1, 0, 0], [1, 1, 0, 0], [0, 0, 0, 0]],
    ],
    [
        [[1, 0, 0, 0], [1, 1, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
        [[0, 1, 1, 0], [1, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
        [[0, 1, 0, 0], [0, 1, 1, 0], [0, 0, 1, 0], [0, 0, 0, 0]],
        [[0, 0, 0, 0], [0, 1, 1, 0], [1, 1, 0, 0], [0, 0, 0, 0]],
    ],
    [
        [[0, 1, 0, 0], [0, 1, 0, 0], [1, 1, 0, 0], [0, 0, 0, 0]],
        [[1, 0, 0, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
        [[0, 1, 1, 0], [0, 1, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 0, 0], [1, 1, 1, 0], [0, 0, 1, 0], [0, 0, 0, 0]],
    ],
    [
        [[0, 1, 0, 0], [1, 1, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
        [[0, 1, 0, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
        [[0, 1, 0, 0], [0, 1, 1, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 0, 0], [1, 1, 1, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
    ],
    [
        [[0, 0, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
        [[0, 0, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
    ],
];

const KICKS_I_CCW: [[[i32; 2]; 4]; 4] = [
    [[2, 0], [-1, 0], [2, -1], [-1, 2]],
    [[-1, 0], [2, 0], [-1, -2], [2, 1]],
    [[-2, 0], [1, 0], [-2, 1], [1, -2]],
    [[1, 0], [-2, 0], [1, 2], [-2, 1]],
];
const KICKS_I_CW: [[[i32; 2]; 4]; 4] = [
    [[1, 0], [-2, 0], [1, 2], [-2, -1]],
    [[2, 0], [-1, 0], [2, -1], [-1, 2]],
    [[-1, 0], [2, 0], [-1, -2], [2, 1]],
    [[-2, 0], [1, 0], [-2, 1], [1, -2]],
];
const KICKS_CCW: [[[i32; 2]; 4]; 4] = [
    [[1, 0], [1, 1], [0, -2], [1, -2]],
    [[1, 0], [1, -1], [0, 2], [1, 2]],
    [[-1, 0], [-1, 1], [0, -2], [-1, -2]],
    [[-1, 0], [-1, -1], [0, 2], [-1, 2]],
];
const KICKS_CW: [[[i32; 2]; 4]; 4] = [
    [[-1, 0], [-1, 1], [0, -2], [-1, -2]],
    [[1, 0], [1, -1], [0, 2], [1, 2]],
    [[1, 0], [1, 1], [0, -2], [1, -2]],
    [[-1, 0], [-1, -1], [0, 2], [-1, 2]],
];
const KICKS_180: [[[i32; 2]; 5]; 4] = [
    [[0, 1], [-1, 1], [1, 1], [-1, 0], [1, 0]],
    [[1, 0], [1, -2], [1, -1], [0, -2], [0, -1]],
    [[0, -1], [1, -1], [-1, -1], [1, 0], [-1, 0]],
    [[-1, 0], [-1, 2], [-1, -1], [0, -2], [0, -1]],
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Training,
    Cheese,
    Four,
    Master,
    PC,
}

#[derive(Clone)]
pub struct Rng {
    s: [u32; 4],
}

impl Rng {
    pub fn seed(&mut self, seed: u32) {
        let mut z = seed | 1;
        for i in 0..4 {
            z = z.wrapping_mul(0x9E3779B9).wrapping_add(0x85EBCA6B);
            z ^= z >> 15;
            self.s[i] = z.wrapping_mul(0x2545F491).wrapping_add(0x9E3779B9);
        }
        for _ in 0..8 {
            self.next_u32();
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        if self.s == [0, 0, 0, 0] {
            self.s[0] = 0x9E3779B9;
        }
        let result = self.s[1]
            .wrapping_mul(5)
            .rotate_left(7)
            .wrapping_mul(9);
        let t = self.s[1] << 9;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(11);
        result
    }

    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo + 1) as u32;
        lo + (self.next_u32() % span) as i32
    }
}

#[derive(Clone)]
pub struct Game {
    pub gx: usize,
    pub gy: usize,
    pub grid: Vec<Vec<u8>>,
    pub hlight: Vec<Vec<u8>>,

    pub piece_x: i32,
    pub piece_y: i32,
    pub piece_a: i32,
    pub piece_h: i32,
    pub swapped: bool,

    pub bag: Vec<i32>,
    pub bag_seed: u32,
    pub bag_type: u8,
    rng: Rng,

    pub mode: Mode,
    pub gravity: u32,
    pub pc_leftover: usize,

    pub damage: u32,
    pub lines: u32,
    pub moves: u32,
    pub lost: bool,
    pub btb: bool,
    pub perfect: bool,
    pub clear_combo: u32,
    pub b2b_text: String,
    pub attack_text: String,

    pub t_spin: bool,
    pub s_mini: bool,
    pub l_kick: usize,
    pub highlight_mode: bool,
    pub hlight_on: bool,

    pub garbage_alternates: bool,
    pub hole_size: i32,
    pub hole_pos: i32,
    pub garbage_amounts: Vec<u32>,
    pub static_bag: String,
    pub static_bag_loaded: bool,
    pub hlight_clear: bool,
    pub infinite_hold: bool,

    pub changed: bool,
    pub sounds: Vec<&'static str>,
    pub pending_comment: Option<(String, String, u32)>,
    pub active_comment: Option<(f64, u32, String, String, bool)>,
}

impl Game {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gx: usize,
        gy: usize,
        bag_type: u8,
        garbage_amounts: Vec<u32>,
        static_bag: String,
        static_bag_loaded: bool,
        hlight_clear: bool,
        infinite_hold: bool,
    ) -> Game {
        let grid = vec![vec![0u8; gy + GRID_H]; gx];
        let hlight = vec![vec![0u8; gy + GRID_H]; gx];
        Game {
            gx,
            gy,
            grid,
            hlight,
            piece_x: 0,
            piece_y: 0,
            piece_a: 0,
            piece_h: -1,
            swapped: false,
            bag: Vec::new(),
            bag_seed: 0,
            bag_type,
            rng: Rng { s: [0; 4] },
            mode: Mode::Training,
            gravity: 0,
            pc_leftover: 7,
            damage: 0,
            lines: 0,
            moves: 0,
            lost: false,
            btb: false,
            perfect: false,
            clear_combo: 0,
            b2b_text: String::new(),
            attack_text: String::new(),
            t_spin: false,
            s_mini: false,
            l_kick: 0,
            highlight_mode: false,
            hlight_on: false,
            garbage_alternates: true,
            hole_size: 0,
            hole_pos: 0,
            garbage_amounts,
            static_bag,
            static_bag_loaded,
            hlight_clear,
            infinite_hold,
            changed: true,
            sounds: Vec::new(),
            pending_comment: None,
            active_comment: None,
        }
    }

    pub fn play_sound(&mut self, name: &'static str) {
        self.sounds.push(name);
    }

    pub fn drain_sounds(&mut self) -> Vec<&'static str> {
        std::mem::take(&mut self.sounds)
    }

    pub fn piece_reset(&mut self) {
        self.piece_x = (self.gx as i32) / 2 - 2;
        self.piece_y = GRID_H as i32 - 2;
        self.piece_a = 0;
        self.t_spin = false;
        self.changed = true;
    }

    pub fn stats_reset(&mut self) {
        self.damage = 0;
        self.lines = 0;
        self.moves = 0;
        self.btb = false;
        self.perfect = false;
        self.swapped = false;
        self.lost = false;
        self.clear_combo = 0;
        self.b2b_text.clear();
        self.attack_text.clear();
    }

    pub fn grid_reset(&mut self) {
        for col in &mut self.grid {
            for cell in col {
                *cell = 0;
            }
        }
    }

    pub fn grid_w(&self) -> usize {
        self.grid.len()
    }

    pub fn grid_h(&self) -> usize {
        self.grid.first().map_or(0, |c| c.len())
    }

    pub fn block_is_full(&self, x: i32, y: i32) -> bool {
        self.block_out_of_bounds(x, y) || self.grid[x as usize][y as usize] != 0
    }

    pub fn block_in_bounds(&self, x: i32, y: i32) -> bool {
        !self.block_out_of_bounds(x, y)
    }

    pub fn block_out_of_bounds(&self, x: i32, y: i32) -> bool {
        x < 0 || x >= self.grid_w() as i32 || y < 0 || y >= self.grid_h() as i32
    }

    pub fn block_is_neighbour(&self, x: i32, y: i32, block: u8) -> bool {
        self.block_in_bounds(x, y) && self.grid[x as usize][y as usize] == block
    }

    pub fn clear_line(&mut self, grid: &mut Vec<Vec<u8>>, line: usize) {
        let w = grid.len();
        let h = grid.first().map_or(0, |c| c.len());
        if line >= h {
            return;
        }
        for col in grid.iter_mut() {
            for j in (0..line).rev() {
                col[j + 1] = col[j];
            }
            col[0] = 0;
        }
        let _ = w;
    }

    pub fn push_line(&mut self, grid: &mut Vec<Vec<u8>>, line: usize) {
        let h = grid.first().map_or(0, |c| c.len());
        if line >= h {
            return;
        }
        for col in grid.iter_mut() {
            for j in 0..line {
                col[j] = col[j + 1];
            }
            col[line] = 0;
        }
    }

    pub fn grid_spawn_garbage(&mut self) {
        let garbage_amount = self.grid_get_garbage_level() as i32;
        let mut hole_change = garbage_amount + self.hole_size;
        let mut hole_last_pos;

        let limit = (self.gy as i32) / 2 - 1;
        let mut i = garbage_amount;
        while i <= limit {
            if i == hole_change {
                hole_last_pos = self.hole_pos;
                loop {
                    self.hole_pos = self.rng.range(0, self.grid_w() as i32 - 1);
                    if self.hole_pos != hole_last_pos || !self.garbage_alternates {
                        break;
                    }
                }
                let pick = self.rng.range(0, self.garbage_amounts.len() as i32 - 1);
                hole_change += self.garbage_amounts[pick as usize] as i32;
            }
            self.grid_add_garbage_line(self.hole_pos);
            i += 1;
        }
        if limit < garbage_amount {
            i = garbage_amount;
        }
        self.hole_size = hole_change - i;
        self.changed = true;
    }

    pub fn grid_add_garbage_line(&mut self, hole_pos: i32) {
        let w = self.grid_w();
        let h = self.grid_h();
        for col in &mut self.grid {
            for j in 1..h {
                col[j - 1] = col[j];
            }
        }
        for (i, col) in self.grid.iter_mut().enumerate() {
            col[h - 1] = if i as i32 == hole_pos { 0 } else { 8 };
        }
        let _ = w;
    }

    pub fn grid_get_garbage_level(&self) -> usize {
        let h = self.grid_h();
        for j in 0..h {
            for col in &self.grid {
                if col[j] == 8 {
                    return h - j;
                }
            }
        }
        0
    }

    pub fn grid_spawn_4w(&mut self) {
        let h = (self.grid_w() as i32) / 2 - 2;
        let d = self.grid_h() - 1;

        for i in 0..self.grid_w() {
            for j in 0..self.grid_h() {
                if (i as i32) < h || (i as i32) > h + 3 {
                    self.grid[i][j] = 8;
                }
            }
        }

        match self.rng.range(0, 5) {
            0 => {
                self.grid[(h) as usize][d] = 8;
                self.grid[(h + 1) as usize][d] = 8;
                self.grid[(h + 2) as usize][d] = 8;
            }
            1 => {
                self.grid[(h + 1) as usize][d] = 8;
                self.grid[(h + 2) as usize][d] = 8;
                self.grid[(h + 3) as usize][d] = 8;
            }
            2 => {
                self.grid[h as usize][d] = 8;
                self.grid[h as usize][d - 1] = 8;
                self.grid[(h + 1) as usize][d - 1] = 8;
            }
            3 => {
                self.grid[(h + 3) as usize][d] = 8;
                self.grid[(h + 3) as usize][d - 1] = 8;
                self.grid[(h + 2) as usize][d - 1] = 8;
            }
            4 => {
                self.grid[h as usize][d] = 8;
                self.grid[h as usize][d - 1] = 8;
                self.grid[(h + 1) as usize][d] = 8;
            }
            _ => {
                self.grid[(h + 3) as usize][d] = 8;
                self.grid[(h + 3) as usize][d - 1] = 8;
                self.grid[(h + 2) as usize][d] = 8;
            }
        }
        self.changed = true;
    }

    pub fn has_full_lines(&self) -> bool {
        let h = self.grid_h();
        (0..h).any(|j| self.grid.iter().all(|col| col[j] != 0))
    }

    pub fn grid_clear_full_lines(&mut self) {
        let h = self.grid_h();
        for j in 0..h {
            let full = self.grid.iter().all(|col| col[j] != 0);
            if full {
                let mut grid = std::mem::take(&mut self.grid);
                self.clear_line(&mut grid, j);
                self.grid = grid;
                if self.hlight_clear {
                    let mut hlight = std::mem::take(&mut self.hlight);
                    self.clear_line(&mut hlight, j);
                    self.hlight = hlight;
                }
            }
        }
        self.changed = true;
    }

    pub fn grid_shift(&mut self, direction: i32) {
        let mut d = direction;
        let last = self.grid_h() - 1;
        while d < 0 {
            let mut grid = std::mem::take(&mut self.grid);
            self.clear_line(&mut grid, last);
            self.grid = grid;
            d += 1;
        }
        while d > 0 {
            let mut grid = std::mem::take(&mut self.grid);
            self.push_line(&mut grid, last);
            self.grid = grid;
            d -= 1;
        }
        self.changed = true;
    }

    pub fn grid_roll(&mut self, direction: i32) {
        let mem = self.grid.clone();
        let w = self.grid_w();
        let h = self.grid_h();
        let mut d = direction;
        while d < 0 {
            for i in 0..w {
                for j in 0..h {
                    self.grid[i][j] = mem[(i + w - 1) % w][j];
                }
            }
            d += 1;
        }
        while d > 0 {
            for i in 0..w {
                for j in 0..h {
                    self.grid[i][j] = mem[(i + 1) % w][j];
                }
            }
            d -= 1;
        }
        self.changed = true;
    }

    pub fn grid_mirror(&mut self) {
        let mut mirror = vec![vec![0u8; self.grid_h()]; self.grid_w()];
        for i in 0..self.grid_w() {
            for j in 0..self.grid_h() {
                let v = self.grid[i][j] as usize;
                mirror[self.grid_w() - 1 - i][j] = MIRROR_CELL.get(v).copied().unwrap_or(v as u8);
            }
        }
        self.grid = mirror;
        self.changed = true;
    }

    pub fn highlight_reset(&mut self) {
        for col in &mut self.hlight {
            for cell in col {
                *cell = 0;
            }
        }
        self.changed = true;
    }

    pub fn clear_board(&mut self) {
        self.bag_reset();
        self.piece_h = -1;
        self.swapped = false;
        self.grid_reset();
        self.stats_reset();
        self.piece_reset();

        match self.mode {
            Mode::Training | Mode::Master => {}
            Mode::Cheese => self.grid_spawn_garbage(),
            Mode::Four => self.grid_spawn_4w(),
            Mode::PC => self.pc_set_bag(self.pc_leftover),
        }
    }

    pub fn lose_game(&mut self) {
        self.fill_color(8);
        self.lost = true;
        self.play_sound("lose");
    }

    pub fn fill_color(&mut self, c: u8) {
        for col in &mut self.grid {
            for cell in col {
                if *cell != 0 {
                    *cell = c;
                }
            }
        }
        self.changed = true;
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.gravity = match mode {
            Mode::Master => 1000,
            _ => 0,
        };
        self.clear_board();
    }

    pub fn current_piece(&self) -> i32 {
        *self.bag.first().unwrap_or(&7)
    }

    pub fn check_tspin(&self) -> bool {
        if self.current_piece() != 6 {
            return false;
        }
        let x = self.piece_x;
        let y = self.piece_y;
        let mut block = 0;
        if self.block_is_full(x, y) {
            block += 1;
        }
        if self.block_is_full(x + 2, y) {
            block += 1;
        }
        if self.block_is_full(x, y + 2) {
            block += 1;
        }
        if self.block_is_full(x + 2, y + 2) {
            block += 1;
        }
        block >= 3
    }

    pub fn check_mini(&self) -> bool {
        if self.t_spin && self.l_kick != 3 {
            let x = self.piece_x;
            let y = self.piece_y;
            match self.piece_a {
                0 => {
                    return !(self.block_is_full(x + 2, y) && self.block_is_full(x, y));
                }
                1 => {
                    return !(self.block_is_full(x, y) && self.block_is_full(x, y + 2));
                }
                2 => {
                    return !(self.block_is_full(x, y + 2) && self.block_is_full(x + 2, y + 2));
                }
                3 => {
                    return !(self.block_is_full(x + 2, y + 2) && self.block_is_full(x + 2, y));
                }
                _ => {}
            }
        }
        false
    }

    pub fn check_lines(&mut self) {
        let mut full_clear = true;
        let mut line_clear = 0;

        let h = self.grid_h();
        for j in 0..h {
            let mut full = true;
            let mut empty = true;
            for col in &self.grid {
                if col[j] == 0 {
                    full = false;
                } else {
                    empty = false;
                }
            }

            if full {
                line_clear += 1;
                let mut grid = std::mem::take(&mut self.grid);
                self.clear_line(&mut grid, j);
                self.grid = grid;
                if self.hlight_clear {
                    let mut hlight = std::mem::take(&mut self.hlight);
                    self.clear_line(&mut hlight, j);
                    self.hlight = hlight;
                }
            } else if !empty {
                full_clear = false;
            }
        }

        self.perfect = full_clear;
        self.lines += line_clear as u32;
        if self.perfect {
            self.damage += 10;
        }
        match line_clear {
            0 => {
                self.clear_combo = 0;
            }
            1..=3 => {
                self.clear_combo += 1;
                self.damage += if self.t_spin {
                    (line_clear as u32) * 2
                } else {
                    line_clear as u32 - 1
                };
                if self.s_mini {
                    self.damage = self.damage.saturating_sub(2);
                }
                if self.t_spin {
                    if self.btb {
                        self.play_sound("btb");
                    } else {
                        self.play_sound("tspin");
                    }
                } else {
                    self.play_sound("clear");
                }
            }
            4 => {
                self.clear_combo += 1;
                self.damage += 4;
                if self.btb {
                    self.play_sound("btb");
                } else {
                    self.play_sound("tetris");
                }
            }
            _ => {}
        }
        self.changed = true;

        if self.clear_combo > 2 {
            self.damage += 1;
        }
        if self.clear_combo > 4 {
            self.damage += 1;
        }
        if self.clear_combo > 6 {
            self.damage += 1;
        }
        if self.clear_combo > 8 {
            self.damage += 1;
        }
        if self.clear_combo > 11 {
            self.damage += 1;
        }

        self.b2b_text.clear();
        self.attack_text.clear();
        if line_clear == 4 || (line_clear > 0 && self.t_spin) {
            if self.btb {
                self.damage += 1;
                self.b2b_text = "B2B".to_string();
            } else {
                self.btb = true;
            }

            self.attack_text = match line_clear {
                0 => "T-SPIN      ",
                1 => {
                    if self.s_mini {
                        "T-SPIN MINI "
                    } else {
                        "T-SPINSINGLE"
                    }
                }
                2 => "T-SPINDOUBLE",
                3 => "T-SPINTRIPLE",
                _ => " FOUR  TRIS ",
            }
            .to_string();
        } else if line_clear != 0 {
            self.btb = false;
        }

        match self.mode {
            Mode::Cheese => {
                if line_clear == 0 {
                    self.grid_spawn_garbage();
                }
            }
            Mode::Four => {
                if line_clear == 0 {
                    self.lose_game();
                }
                self.add_wide(line_clear as usize);
            }
            _ => {}
        }
    }

    pub fn add_wide(&mut self, amount: usize) {
        let hole = (self.grid_w() as i32) / 2 - 2;
        for j in 0..=amount.min(self.grid_h() - 1) {
            for (i, col) in self.grid.iter_mut().enumerate() {
                if (i as i32) < hole || (i as i32) > hole + 3 {
                    col[j] = 8;
                }
            }
        }
        self.changed = true;
    }

    pub fn piece_move(&mut self, angle: i32, x: i32, y: i32) -> bool {
        let rotation = angle;
        let angle = (self.piece_a + angle).rem_euclid(4);
        let mut x = self.piece_x + x;
        let mut y = self.piece_y + y;
        let piece = self.bag_get_piece();

        if self.piece_fits(piece, angle, x, y) {
            self.piece_a = angle;
            self.piece_x = x;
            self.piece_y = y;

            self.t_spin = rotation != 0 && self.check_tspin();
            self.s_mini = rotation != 0 && self.check_mini();
            if rotation != 0 {
                if self.t_spin {
                    self.play_sound("kick");
                } else {
                    self.play_sound("rotate");
                }
            }

            self.changed = true;
            true
        } else if rotation != 0 {
            if self.piece_kick(piece, angle, &mut x, &mut y, rotation) {
                self.piece_a = angle;
                self.piece_x = x;
                self.piece_y = y;

                self.t_spin = self.check_tspin();
                self.s_mini = self.check_mini();
                if self.t_spin {
                    self.play_sound("kick");
                } else {
                    self.play_sound("rotate");
                }

                self.changed = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn piece_kick(&mut self, piece: i32, angle: i32, x: &mut i32, y: &mut i32, rotation: i32) -> bool {
        let offsets: Vec<[i32; 2]> = match (piece, rotation) {
            (0, 1) => KICKS_I_CCW[angle as usize].to_vec(),
            (0, 3) => KICKS_I_CW[angle as usize].to_vec(),
            (0, _) => return false,
            (_, 1) => KICKS_CCW[angle as usize].to_vec(),
            (_, 2) => KICKS_180[angle as usize].to_vec(),
            (_, 3) => KICKS_CW[angle as usize].to_vec(),
            _ => return false,
        };

        for (i, off) in offsets.iter().enumerate() {
            if self.piece_fits(piece, angle, *x + off[0], *y + off[1]) {
                *x += off[0];
                *y += off[1];
                self.l_kick = i;
                return true;
            }
        }
        false
    }

    pub fn piece_fits(&self, piece: i32, angle: i32, x: i32, y: i32) -> bool {
        let shape = piece_get_shape(piece, angle);
        for i in 0..4 {
            for j in 0..4 {
                if shape[i][j] != 0 && self.block_is_full(x + i as i32, y + j as i32) {
                    return false;
                }
            }
        }
        true
    }

    pub fn piece_freeze(&mut self, piece: i32, angle: i32, x: i32, y: i32) {
        let shape = piece_get_shape(piece, angle);
        for i in 0..4 {
            for j in 0..4 {
                if shape[i][j] != 0 && self.block_in_bounds(x + i as i32, y + j as i32) {
                    self.grid[(x + i as i32) as usize][(y + j as i32) as usize] = (piece + 1) as u8;
                }
            }
        }
    }

    pub fn piece_next(&mut self) {
        self.bag_next();
        self.piece_reset();

        self.swapped = false;
        self.changed = true;

        let piece = self.bag_get_piece();
        if !self.piece_fits(piece, self.piece_a, self.piece_x, self.piece_y) {
            self.lose_game();
        }
    }

    pub fn piece_hold(&mut self) {
        if self.swapped || self.lost {
            return;
        }

        self.piece_reset();
        if self.piece_h == -1 {
            self.piece_h = self.bag_get_piece();
            self.piece_next();
        } else {
            std::mem::swap(&mut self.bag[0], &mut self.piece_h);
        }

        if !self.infinite_hold() {
            self.swapped = true;
        }
        self.changed = true;

        self.play_sound("hold");

        let piece = self.bag_get_piece();
        if !self.piece_fits(piece, self.piece_a, self.piece_x, self.piece_y) {
            self.lose_game();
        }
    }

    pub fn infinite_hold(&self) -> bool {
        self.infinite_hold
    }

    pub fn bag_get_piece(&mut self) -> i32 {
        self.bag_fill();
        *self.bag.first().unwrap_or(&7)
    }

    pub fn bag_fill(&mut self) {
        while self.bag.len() < 7 {
            let mut fill: Vec<i32> = (0..7).collect();

            self.rng.seed(self.bag_seed);
            match self.bag_type {
                0 => {
                    for i in 0..fill.len() {
                        let j = self.rng.range(i as i32, fill.len() as i32 - 1) as usize;
                        fill.swap(i, j);
                    }
                }
                1 => {
                    fill.extend_from_slice(&(0..7).collect::<Vec<i32>>());
                    for i in 0..fill.len() {
                        let j = self.rng.range(i as i32, fill.len() as i32 - 1) as usize;
                        fill.swap(i, j);
                    }
                }
                _ => {
                    for v in fill.iter_mut() {
                        *v = self.rng.range(0, 6);
                    }
                }
            }
            self.bag_reseed();

            self.bag.extend_from_slice(&fill);
        }
    }

    pub fn bag_next(&mut self) {
        self.bag_fill();

        if self.bag.get(1) == Some(&-1) {
            if self.piece_h == -1 {
                self.bag.drain(0..2);
            } else {
                std::mem::swap(&mut self.piece_h, &mut self.bag[0]);
                self.piece_h = -1;
            }
        } else {
            self.bag.remove(0);
        }
    }

    pub fn bag_get_separator(&self) -> Vec<usize> {
        let period = (self.bag_type as usize + 1) * 7;
        let mut bag_count = self.bag.len() % period;
        let mut result = vec![0usize];
        while bag_count <= self.bag.len() {
            result.push(bag_count);
            bag_count += period;
        }
        result
    }

    pub fn bag_shuffle(&mut self) {
        let sep = self.bag_get_separator();
        for k in 1..sep.len() {
            for i in sep[k - 1]..sep[k] {
                let j = self.rng.range(i as i32, sep[k] as i32 - 1) as usize;
                self.bag.swap(i, j);
            }
        }
        self.changed = true;
    }

    pub fn bag_mirror(&mut self) {
        for v in self.bag.iter_mut() {
            if (0..7).contains(v) {
                *v = MIRROR_PIECE[*v as usize];
            }
        }
        self.changed = true;
    }

    pub fn bag_reset(&mut self) {
        self.bag.clear();
        if self.static_bag_loaded && !self.static_bag.is_empty() {
            let list = self.static_bag.clone();
            self.bag_load_from_string(&list);
        }
        self.bag_fill();
    }

    pub fn bag_reseed(&mut self) {
        self.bag_seed = self.rng.range(0, 65535) as u32;
    }

    pub fn bag_load_from_string(&mut self, s: &str) {
        let q: Vec<i32> = s
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| piece_get_id(c))
            .collect();
        self.bag = q;
        self.changed = true;
        self.piece_reset();
    }

    pub fn pc_set_leftover(&mut self, leftover: usize) {
        if self.mode != Mode::PC {
            return;
        }
        self.pc_leftover = PC_SIZES[leftover - 1];
        self.clear_board();
    }

    pub fn pc_set_bag(&mut self, leftover: usize) {
        let mut fill: Vec<i32>;
        loop {
            fill = (0..7).collect();
            for i in 0..fill.len() {
                let j = self.rng.range(i as i32, fill.len() as i32 - 1) as usize;
                fill.swap(i, j);
            }
            fill.truncate(leftover);
            if !self.pc_reroll_bag(&fill) {
                break;
            }
        }

        self.bag = fill;
        self.changed = true;

        self.piece_h = -1;
        self.swapped = false;
        self.bag_fill();
    }

    fn pc_reroll_bag(&self, bag: &[i32]) -> bool {
        if bag.len() != 4 {
            return false;
        }
        let mut p = [false; 7];
        for &v in bag {
            if (0..7).contains(&v) {
                p[v as usize] = true;
            }
        }
        p[1] && p[5] && p[6]
    }

    pub fn hold_reset(&mut self) {
        self.swapped = false;
        self.piece_h = -1;
        self.changed = true;
    }

    pub fn hold_mirror(&mut self) {
        if (0..7).contains(&self.piece_h) {
            self.piece_h = MIRROR_PIECE[self.piece_h as usize];
        }
    }

    pub fn hold_shuffle(&mut self) {
        if self.piece_h != -1 {
            let sep = self.bag_get_separator();
            let j = self.rng.range(sep[0] as i32, sep[1] as i32 - 1);
            if let Some(slot) = self.bag.get_mut(j as usize) {
                std::mem::swap(&mut self.piece_h, slot);
            }
        }
        self.changed = true;
    }

    pub fn recolor(&mut self, coords: &[(usize, usize)], c: u8) {
        self.changed = true;
        for &(x, y) in coords {
            self.grid[x][y] = c;
        }
    }

    pub fn auto_color(&mut self, x: i32, y: i32, stroke: usize, stroke_coord: &[(i32, i32)]) {
        if stroke < 4 {
            let mut mem = vec![vec![false; self.grid_h()]; self.grid_w()];
            let mut queue = std::collections::VecDeque::new();
            queue.push_back((x, y));
            let mut coord: Vec<(i32, i32)> = Vec::new();

            while !queue.is_empty() && coord.len() < 5 {
                let pair = queue.pop_front().unwrap();
                if self.block_in_bounds(pair.0, pair.1)
                    && self.grid[pair.0 as usize][pair.1 as usize] == 8
                    && !mem[pair.0 as usize][pair.1 as usize]
                {
                    coord.push(pair);
                    mem[pair.0 as usize][pair.1 as usize] = true;
                    queue.push_back((pair.0 - 1, pair.1));
                    queue.push_back((pair.0 + 1, pair.1));
                    queue.push_back((pair.0, pair.1 - 1));
                    queue.push_back((pair.0, pair.1 + 1));
                }
            }

            if coord.len() == 4 {
                let piece = piece_from_shape(&shape_from_coords(&coord));
                let coords: Vec<(usize, usize)> = coord
                    .iter()
                    .map(|&(x, y)| (x as usize, y as usize))
                    .collect();
                self.recolor(&coords, (piece + 1) as u8);
            }
        } else if stroke == 4 {
            let piece = piece_from_shape(&shape_from_coords(stroke_coord));
            let coords: Vec<(usize, usize)> = stroke_coord
                .iter()
                .map(|&(x, y)| (x as usize, y as usize))
                .collect();
            self.recolor(&coords, (piece + 1) as u8);
        } else if stroke == 5 {
            let coords: Vec<(usize, usize)> = stroke_coord
                .iter()
                .map(|&(x, y)| (x as usize, y as usize))
                .collect();
            self.recolor(&coords, 8);
        }
    }
}

pub fn piece_get_shape(piece: i32, angle: i32) -> Shape {
    let angle = angle.clamp(0, 3) as usize;
    let piece = if (0..=7).contains(&piece) { piece as usize } else { 7 };
    SHAPES[piece][angle]
}

pub fn piece_get_name(piece: i32) -> &'static str {
    if piece < 0 {
        return "-";
    }
    if piece > 6 {
        return "M";
    }
    PIECE_NAMES[piece as usize]
}

pub fn piece_get_id(c: char) -> i32 {
    match c.to_ascii_uppercase() {
        '-' => -1,
        'I' => 0,
        'J' => 1,
        'S' => 2,
        'O' => 3,
        'Z' => 4,
        'L' => 5,
        'T' => 6,
        _ => 7,
    }
}

pub fn piece_from_shape(shape: &Shape) -> i32 {
    let mut count = 0;
    for i in 0..4 {
        for j in 0..4 {
            if shape[i][j] != 0 {
                count += 1;
            }
        }
    }
    if count != 4 {
        return 7;
    }

    if shape[0][0] != 0 && shape[1][0] != 0 && shape[2][0] != 0 && shape[3][0] != 0 {
        return 0;
    }
    if shape[0][0] != 0 && shape[0][1] != 0 && shape[0][2] != 0 && shape[0][3] != 0 {
        return 0;
    }

    if shape[0][0] != 0 && shape[0][1] != 0 && shape[1][1] != 0 && shape[2][1] != 0 {
        return 1;
    }
    if shape[0][0] != 0 && shape[1][0] != 0 && shape[2][0] != 0 && shape[2][1] != 0 {
        return 1;
    }
    if shape[0][2] != 0 && shape[1][0] != 0 && shape[1][1] != 0 && shape[1][2] != 0 {
        return 1;
    }
    if shape[0][0] != 0 && shape[0][1] != 0 && shape[0][2] != 0 && shape[1][0] != 0 {
        return 1;
    }

    if shape[0][1] != 0 && shape[1][0] != 0 && shape[1][1] != 0 && shape[2][0] != 0 {
        return 2;
    }
    if shape[0][0] != 0 && shape[0][1] != 0 && shape[1][1] != 0 && shape[1][2] != 0 {
        return 2;
    }

    if shape[0][0] != 0 && shape[0][1] != 0 && shape[1][0] != 0 && shape[1][1] != 0 {
        return 3;
    }

    if shape[0][0] != 0 && shape[1][0] != 0 && shape[1][1] != 0 && shape[2][1] != 0 {
        return 4;
    }
    if shape[0][1] != 0 && shape[0][2] != 0 && shape[1][0] != 0 && shape[1][1] != 0 {
        return 4;
    }

    if shape[0][1] != 0 && shape[1][1] != 0 && shape[2][0] != 0 && shape[2][1] != 0 {
        return 5;
    }
    if shape[0][0] != 0 && shape[0][1] != 0 && shape[1][0] != 0 && shape[2][0] != 0 {
        return 5;
    }
    if shape[0][0] != 0 && shape[0][1] != 0 && shape[0][2] != 0 && shape[1][2] != 0 {
        return 5;
    }
    if shape[0][0] != 0 && shape[1][0] != 0 && shape[1][1] != 0 && shape[1][2] != 0 {
        return 5;
    }

    if shape[0][1] != 0 && shape[1][0] != 0 && shape[1][1] != 0 && shape[2][1] != 0 {
        return 6;
    }
    if shape[0][0] != 0 && shape[1][0] != 0 && shape[1][1] != 0 && shape[2][0] != 0 {
        return 6;
    }
    if shape[0][0] != 0 && shape[0][1] != 0 && shape[0][2] != 0 && shape[1][1] != 0 {
        return 6;
    }
    if shape[0][1] != 0 && shape[1][0] != 0 && shape[1][1] != 0 && shape[1][2] != 0 {
        return 6;
    }

    7
}

pub fn shape_from_coords(coord: &[(i32, i32)]) -> Shape {
    let mut shape = [[0u8; 4]; 4];
    let mut x = i32::MAX;
    let mut y = i32::MAX;
    for &(cx, cy) in coord {
        x = x.min(cx);
        y = y.min(cy);
    }
    for &(cx, cy) in coord {
        let nx = cx - x;
        let ny = cy - y;
        if (0..4).contains(&nx) && (0..4).contains(&ny) {
            shape[nx as usize][ny as usize] = 1;
        }
    }
    shape
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        Game::new(10, 20, 0, vec![1], String::new(), false, true, false)
    }

    #[test]
    fn shapes_match_original() {
        let i = piece_get_shape(0, 0);
        assert_eq!(i[1][1], 1);
        assert_eq!(i[4 - 1][1], 1);
        let t = piece_get_shape(6, 0);
        assert_eq!(t[0][1], 1);
        assert_eq!(t[1][0], 1);
        assert_eq!(t[1][1], 1);
        assert_eq!(t[2][1], 1);
        let mono = piece_get_shape(-1, 0);
        assert_eq!(mono[1][1], 1);
    }

    #[test]
    fn piece_fits_and_move() {
        let mut g = game();
        g.bag_reset();
        g.piece_reset();
        let piece = g.bag_get_piece();
        assert!(g.piece_fits(piece, 0, g.piece_x, g.piece_y));
        assert!(g.piece_move(0, 1, 0));
        assert_eq!(g.piece_x, (g.gx as i32) / 2 - 2 + 1);
    }

    #[test]
    fn seven_bag_distribution() {
        let mut g = game();
        g.bag_reset();
        let mut counts = [0i32; 7];
        for _ in 0..70 {
            let p = g.bag_get_piece();
            counts[p as usize] += 1;
            g.bag_next();
            g.piece_reset();
        }
        assert!(counts.iter().all(|&c| c == 10), "{counts:?}");
    }

    #[test]
    fn hard_drop_locks_and_scores() {
        let mut g = game();
        g.clear_board();
        let before_moves = g.moves;
        while g.piece_move(0, 0, 1) {}
        let piece = g.bag_get_piece();
        g.piece_freeze(piece, g.piece_a, g.piece_x, g.piece_y);
        g.moves += 1;
        g.check_lines();
        g.piece_next();
        assert_eq!(g.moves, before_moves + 1);
        assert!(!g.lost);
    }

    #[test]
    fn tetris_scoring_and_b2b() {
        let mut g = game();
        let h = g.grid_h();
        let w = g.gx;
        for col in 0..w {
            for j in 0..4 {
                g.grid[col][h - 1 - j] = 8;
            }
        }
        g.grid[0][h - 5] = 8;
        g.t_spin = false;
        g.btb = false;
        g.check_lines();
        assert_eq!(g.lines, 4);
        assert_eq!(g.damage, 4);
        assert!(g.btb);
        assert_eq!(g.attack_text, " FOUR  TRIS ");
    }

    #[test]
    fn tspin_detection() {
        let mut g = game();
        g.bag = vec![6, 0, 1, 2, 3, 4, 5];
        g.piece_reset();
        let px = g.piece_x;
        let py = g.piece_y;
        for &(dx, dy) in &[(0, 0), (2, 0), (0, 2)] {
            g.grid[(px + dx) as usize][(py + dy) as usize] = 8;
        }
        assert!(g.piece_move(3, 0, 0));
        assert!(g.t_spin);
    }

    #[test]
    fn garbage_spawn() {
        let mut g = game();
        g.mode = Mode::Cheese;
        g.grid_spawn_garbage();
        let level = g.grid_get_garbage_level();
        let h = g.grid_h();
        assert!(level >= 9, "level {level}");
        assert!(g.grid.iter().any(|col| col[h - 1] == 8));
    }

    #[test]
    fn separator_periods() {
        let mut g = game();
        g.bag_type = 0;
        g.bag = (0..7).collect();
        assert_eq!(g.bag_get_separator(), vec![0, 0, 7]);
        g.bag_type = 1;
        g.bag = (0..14).map(|i| i % 7).collect();
        assert_eq!(g.bag_get_separator(), vec![0, 0, 14]);
    }

    #[test]
    fn mirror_lookup() {
        assert_eq!(MIRROR_PIECE[1], 5);
        assert_eq!(MIRROR_CELL[2], 6);
        assert_eq!(MIRROR_CELL[3], 5);
    }
}
