// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

use std::collections::HashMap;

pub const ICON_UNDO: (i32, i32, i32, i32) = (0, 0, 16, 12);
pub const ICON_REDO: (i32, i32, i32, i32) = (0, 13, 16, 12);
pub const ICON_ERASER: (i32, i32, i32, i32) = (0, 26, 13, 13);
pub const ICON_SHUFFLE: (i32, i32, i32, i32) = (0, 40, 18, 18);
pub const ICON_DELETE: (i32, i32, i32, i32) = (0, 59, 14, 14);
pub const ICON_SNAP: (i32, i32, i32, i32) = (20, 0, 58, 35);
pub const ICON_MIRROR: (i32, i32, i32, i32) = (20, 36, 58, 35);

pub fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
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

#[derive(Clone)]
pub struct Bitmap {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u32>,
}

impl Bitmap {
    pub fn new(w: usize, h: usize) -> Bitmap {
        Bitmap { w, h, px: vec![0; w * h] }
    }

    pub fn from_rgba8(w: usize, h: usize, data: &[u8]) -> Bitmap {
        let mut px = Vec::with_capacity(w * h);
        for p in data.chunks_exact(4) {
            px.push(((p[3] as u32) << 24) | ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | p[2] as u32);
        }
        Bitmap { w, h, px }
    }

    pub fn get(&self, x: usize, y: usize) -> u32 {
        self.px[y * self.w + x]
    }
}

#[derive(Clone)]
struct CachedGlyph {
    coverage: Vec<u8>,
    xmin: i32,
    ymin: i32,
    width: usize,
    height: usize,
    advance: u32,
}

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u32>,
    pub scale: f64,
    pub scroll: (f64, f64),
    pub off: (i32, i32),
    font: fontdue::Font,
    glyphs: HashMap<(char, u32), CachedGlyph>,
    clip: (i32, i32, i32, i32),
}

impl Canvas {
    pub fn new(scale: f64) -> Canvas {
        let font_data = include_bytes!("../assets/fonts/LiberationMono-Regular.ttf");
        let font = fontdue::Font::from_bytes(font_data.as_slice(), fontdue::FontSettings::default())
            .expect("bundled font");
        Canvas {
            w: 1,
            h: 1,
            px: vec![0],
            scale,
            scroll: (0.0, 0.0),
            off: (0, 0),
            font,
            glyphs: HashMap::new(),
            clip: (0, 0, i32::MAX, i32::MAX),
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.px = vec![0; w * h];
    }

    pub fn clear(&mut self, color: u32) {
        self.px.fill(color & 0xFFFFFF);
    }

    pub fn set_scroll(&mut self, x: f64, y: f64) {
        self.scroll = (x, y);
    }

    pub fn set_off(&mut self, off: (i32, i32)) {
        self.off = off;
    }

    pub fn snapshot(&self) -> Bitmap {
        Bitmap { w: self.w, h: self.h, px: self.px.clone() }
    }

    pub fn restore_from(&mut self, bmp: &Bitmap) {
        if bmp.w == self.w && bmp.h == self.h {
            self.px.copy_from_slice(&bmp.px);
        }
    }

    pub fn blit_bitmap(&mut self, bmp: &Bitmap, dx: i32, dy: i32) {
        for y in 0..bmp.h {
            let ty = dy + y as i32;
            if ty < 0 || ty >= self.h as i32 {
                continue;
            }
            for x in 0..bmp.w {
                let tx = dx + x as i32;
                if tx < 0 || tx >= self.w as i32 {
                    continue;
                }
                self.px[(ty as usize) * self.w + (tx as usize)] = bmp.px[y * bmp.w + x] & 0xFFFFFF;
            }
        }
    }

    pub fn blit_bitmap_scaled(&mut self, bmp: &Bitmap, dx: f64, dy: f64, dw: f64, dh: f64) {
        if bmp.w == 0 || bmp.h == 0 || dw < 1.0 || dh < 1.0 {
            return;
        }
        let xa = dx.max(0.0) as i32;
        let ya = dy.max(0.0) as i32;
        let xb = ((dx + dw) as i32).min(self.w as i32);
        let yb = ((dy + dh) as i32).min(self.h as i32);
        for ty in ya..yb {
            for tx in xa..xb {
                let u = ((tx as f64 - dx) / dw * bmp.w as f64) as usize;
                let v = ((ty as f64 - dy) / dh * bmp.h as f64) as usize;
                let u = u.min(bmp.w - 1);
                let v = v.min(bmp.h - 1);
                self.px[(ty as usize) * self.w + (tx as usize)] = bmp.px[v * bmp.w + u] & 0xFFFFFF;
            }
        }
    }

    pub fn set_clip(&mut self, clip: (i32, i32, i32, i32)) {
        self.clip = clip;
    }

    fn x0(&self, v: f64) -> i32 {
        ((v - self.scroll.0) * self.scale).round() as i32 + self.off.0
    }

    fn x1(&self, v: f64) -> i32 {
        ((v - self.scroll.0) * self.scale).round() as i32 + self.off.0
    }

    fn y0(&self, v: f64) -> i32 {
        ((v - self.scroll.1) * self.scale).round() as i32 + self.off.1
    }

    fn y1(&self, v: f64) -> i32 {
        ((v - self.scroll.1) * self.scale).round() as i32 + self.off.1
    }

    fn text_px(&self, size: f64) -> u32 {
        ((size * 0.82 * self.scale).round() as u32).max(1)
    }

    pub fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: u32) {
        self.fill_rect_i(self.x0(x), self.y0(y), self.x1(x + w) - self.x0(x), self.y1(y + h) - self.y0(y), color);
    }

    fn fill_rect_i(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        let (cx, cy, cw, ch) = self.clip;
        let xa = x.max(cx);
        let ya = y.max(cy);
        let xb = (x + w).min(cx + cw);
        let yb = (y + h).min(cy + ch);
        if xa >= xb || ya >= yb {
            return;
        }
        for py in ya..yb {
            if py < 0 || py >= self.h as i32 {
                continue;
            }
            let row = &mut self.px[(py as usize) * self.w..][..self.w];
            for px in xa..xb {
                if px < 0 || px >= self.w as i32 {
                    continue;
                }
                row[px as usize] = color & 0xFFFFFF;
            }
        }
    }

    pub fn fill_rect_alpha(&mut self, x: f64, y: f64, w: f64, h: f64, color: u32, alpha: u8) {
        let xa = self.x0(x).max(0);
        let ya = self.y0(y).max(0);
        let xb = self.x1(x + w).min(self.w as i32);
        let yb = self.y1(y + h).min(self.h as i32);
        let (cx, cy, cw, ch) = self.clip;
        let xa = xa.max(cx);
        let ya = ya.max(cy);
        let xb = xb.min(cx + cw);
        let yb = yb.min(cy + ch);
        if xa >= xb || ya >= yb {
            return;
        }
        let a = alpha as u32;
        let sr = unpack_r(color) as u32;
        let sg = unpack_g(color) as u32;
        let sb = unpack_b(color) as u32;
        for py in ya..yb {
            let row = &mut self.px[(py as usize) * self.w..][..self.w];
            for px in xa..xb {
                let dst = row[px as usize];
                let dr = unpack_r(dst) as u32;
                let dg = unpack_g(dst) as u32;
                let db = unpack_b(dst) as u32;
                let r = (sr * a + dr * (255 - a)) / 255;
                let g = (sg * a + dg * (255 - a)) / 255;
                let b = (sb * a + db * (255 - a)) / 255;
                row[px as usize] = pack_rgb(r as u8, g as u8, b as u8);
            }
        }
    }

    pub fn frame_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: u32) {
        self.fill_rect_i(self.x0(x), self.y0(y), self.x1(x + w) - self.x0(x), 1, color);
        self.fill_rect_i(self.x0(x), self.y1(y + h) - 1, self.x1(x + w) - self.x0(x), 1, color);
        self.fill_rect_i(self.x0(x), self.y0(y), 1, self.y1(y + h) - self.y0(y), color);
        self.fill_rect_i(self.x1(x + w) - 1, self.y0(y), 1, self.y1(y + h) - self.y0(y), color);
    }

    pub fn xor_blit(&mut self, dst_x: f64, dst_y: f64, src: &Bitmap, sx: i32, sy: i32, sw: i32, sh: i32) {
        let px0 = self.x0(dst_x);
        let py0 = self.y0(dst_y);
        let pw = self.x1(dst_x + sw as f64) - px0;
        let ph = self.y1(dst_y + sh as f64) - py0;
        if pw <= 0 || ph <= 0 {
            return;
        }
        for dy in 0..ph {
            for dx in 0..pw {
                let sxp = (dx as f64 / pw as f64 * sw as f64) as i32;
                let syp = (dy as f64 / ph as f64 * sh as f64) as i32;
                let sxp = sxp.clamp(0, sw - 1);
                let syp = syp.clamp(0, sh - 1);
                if sx + sxp >= src.w as i32 || sy + syp >= src.h as i32 {
                    continue;
                }
                let xp = px0 + dx;
                let yp = py0 + dy;
                if xp < 0 || yp < 0 || xp >= self.w as i32 || yp >= self.h as i32 {
                    continue;
                }
                let (cx, cy, cw, ch) = self.clip;
                if xp < cx || yp < cy || xp >= cx + cw || yp >= cy + ch {
                    continue;
                }
                let s = src.get((sx + sxp) as usize, (sy + syp) as usize) & 0xFFFFFF;
                let d = self.px[(yp as usize) * self.w + (xp as usize)] & 0xFFFFFF;
                self.px[(yp as usize) * self.w + (xp as usize)] = (s ^ d) & 0xFFFFFF;
            }
        }
    }

    pub fn texture_block(
        &mut self,
        dst_x: f64,
        dst_y: f64,
        dst_w: f64,
        dst_h: f64,
        tex: &Bitmap,
        sx: usize,
        sy: usize,
        sw: usize,
        sh: usize,
    ) {
        let xa = self.x0(dst_x);
        let ya = self.y0(dst_y);
        let xb = self.x1(dst_x + dst_w);
        let yb = self.y1(dst_y + dst_h);
        for yp in ya..yb {
            if yp < 0 || yp >= self.h as i32 {
                continue;
            }
            for xp in xa..xb {
                if xp < 0 || xp >= self.w as i32 {
                    continue;
                }
                let (cx, cy, cw, ch) = self.clip;
                if xp < cx || yp < cy || xp >= cx + cw || yp >= cy + ch {
                    continue;
                }
                let u = sx as f64 + (xp - xa) as f64 / (xb - xa) as f64 * sw as f64;
                let v = sy as f64 + (yp - ya) as f64 / (yb - ya) as f64 * sh as f64;
                let u = (u as usize).min(sx + sw - 1);
                let v = (v as usize).min(sy + sh - 1);
                let s = tex.get(u, v);
                let a = (s >> 24) as u32;
                if a == 0 {
                    continue;
                }
                let d = self.px[(yp as usize) * self.w + (xp as usize)];
                if a >= 255 {
                    self.px[(yp as usize) * self.w + (xp as usize)] = s & 0xFFFFFF;
                } else {
                    let dr = unpack_r(d) as u32;
                    let dg = unpack_g(d) as u32;
                    let db = unpack_b(d) as u32;
                    let r = (unpack_r(s) as u32 * a + dr * (255 - a)) / 255;
                    let g = (unpack_g(s) as u32 * a + dg * (255 - a)) / 255;
                    let b = (unpack_b(s) as u32 * a + db * (255 - a)) / 255;
                    self.px[(yp as usize) * self.w + (xp as usize)] = pack_rgb(r as u8, g as u8, b as u8);
                }
            }
        }
    }

    fn glyph(&mut self, ch: char, px: u32) -> CachedGlyph {
        if let Some(cached) = self.glyphs.get(&(ch, px)) {
            return CachedGlyph {
                coverage: cached.coverage.clone(),
                xmin: cached.xmin,
                ymin: cached.ymin,
                width: cached.width,
                height: cached.height,
                advance: cached.advance,
            };
        }
        let (metrics, coverage) = self.font.rasterize(ch, px as f32);
        let cached = CachedGlyph {
            coverage,
            xmin: metrics.xmin,
            ymin: metrics.ymin,
            width: metrics.width,
            height: metrics.height,
            advance: metrics.advance_width.round() as u32,
        };
        self.glyphs.insert((ch, px), cached.clone());
        cached
    }

    pub fn text_width(&mut self, text: &str, size: f64) -> f64 {
        let px = self.text_px(size);
        let mut width = 0u32;
        for ch in text.chars() {
            width += self.glyph(ch, px).advance;
        }
        width as f64 / self.scale
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        rect: (f64, f64, f64, f64),
        align: Align,
        size: f64,
        color: u32,
        bg: Option<u32>,
    ) {
        let px = self.text_px(size);
        let (rx, ry, rw, rh) = rect;
        let mut x = match align {
            Align::Left => rx,
            Align::Center => rx + rw / 2.0 - self.text_width(text, size) / 2.0,
        };
        let saved_clip = self.clip;
        let ty0 = self.y0(ry).max(0);
        let ty1 = self.y1(ry + rh.max(1.0)).min(self.h as i32);
        if ty1 <= ty0 {
            return;
        }
        self.clip = (0, ty0, self.w as i32, ty1 - ty0);
        let baseline = self.y0(ry) as f64 + 0.80 * size * self.scale;

        for ch in text.chars() {
            let g = self.glyph(ch, px);
            if let Some(bg_color) = bg {
                let ax = self.x0(x);
                self.fill_rect_i(
                    ax,
                    self.y0(ry),
                    self.x1(x + g.advance as f64 / self.scale) - ax,
                    self.y1(ry + size) - self.y0(ry),
                    bg_color & 0xFFFFFF,
                );
            }
            if g.width > 0 && g.height > 0 {
                let gx = self.x0(x) + (g.xmin as f64 * self.scale).round() as i32;
                let gy = (baseline as i32) - g.height as i32 - (g.ymin as i32);
                for gyi in 0..g.height as i32 {
                    for gxi in 0..g.width as i32 {
                        let cov = g.coverage[gyi as usize * g.width as usize + gxi as usize] as u32;
                        if cov == 0 {
                            continue;
                        }
                        let xp = gx + gxi;
                        let yp = gy + gyi;
                        if xp < 0 || yp < 0 || xp >= self.w as i32 || yp >= self.h as i32 {
                            continue;
                        }
                        let (cx, cy, cw, ch) = self.clip;
                        if xp < cx || yp < cy || xp >= cx + cw || yp >= cy + ch {
                            continue;
                        }
                        let d = self.px[(yp as usize) * self.w + (xp as usize)];
                        if cov >= 255 {
                            self.px[(yp as usize) * self.w + (xp as usize)] = color & 0xFFFFFF;
                        } else {
                            let a = cov;
                            let dr = unpack_r(d) as u32;
                            let dg = unpack_g(d) as u32;
                            let db = unpack_b(d) as u32;
                            let r = (unpack_r(color) as u32 * a + dr * (255 - a)) / 255;
                            let g2 = (unpack_g(color) as u32 * a + dg * (255 - a)) / 255;
                            let b = (unpack_b(color) as u32 * a + db * (255 - a)) / 255;
                            self.px[(yp as usize) * self.w + (xp as usize)] =
                                pack_rgb(r as u8, g2 as u8, b as u8);
                        }
                    }
                }
            }
            x += g.advance as f64 / self.scale;
        }
        self.clip = saved_clip;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Align {
    Left,
    Center,
}
