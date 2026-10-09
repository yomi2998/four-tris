// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

const CHUNK: usize = 4096;

pub fn compress_store(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for chunk in input.chunks(CHUNK) {
        let header: u16 = 0x3000 | (chunk.len() - 1) as u16;
        out.extend_from_slice(&header.to_le_bytes());
        out.extend_from_slice(chunk);
    }
    out
}

pub fn decompress(input: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 2 <= input.len() {
        let header = u16::from_le_bytes([input[i], input[i + 1]]);
        if header == 0 {
            break;
        }
        i += 2;
        let size = ((header & 0xFFF) + 1) as usize;
        let end = (i + size).min(input.len());
        let chunk = &input[i.min(input.len())..end];
        i = end;
        if header & 0x8000 != 0 {
            decompress_chunk(chunk, &mut out)?;
        } else {
            out.extend_from_slice(chunk);
        }
    }
    Some(out)
}

fn decompress_chunk(chunk: &[u8], out: &mut Vec<u8>) -> Option<()> {
    let base = out.len();
    let mut c = 0;
    while c < chunk.len() {
        let flags = chunk[c];
        c += 1;
        for bit in 0..8u8 {
            if c >= chunk.len() {
                break;
            }
            if flags & (1 << bit) == 0 {
                out.push(chunk[c]);
                c += 1;
                continue;
            }
            if c + 2 > chunk.len() {
                return None;
            }
            let token = u16::from_le_bytes([chunk[c], chunk[c + 1]]);
            c += 2;

            let mut l_mask: u32 = 0xFFF;
            let mut o_shift: u32 = 12;
            let mut pos = out.len() - base;
            pos = pos.saturating_sub(1);
            while pos >= 0x10 {
                l_mask >>= 1;
                o_shift -= 1;
                pos >>= 1;
            }
            let length = ((token as u32 & l_mask) + 3) as usize;
            let offset = ((token as u32 >> o_shift) + 1) as usize;
            let start = out.len().checked_sub(offset)?;

            if length >= offset {
                let pattern = out[start..].to_vec();
                let reps = 0xFFF / pattern.len() + 1;
                let mut tmp = Vec::with_capacity(pattern.len() * reps);
                for _ in 0..reps {
                    tmp.extend_from_slice(&pattern);
                }
                out.extend_from_slice(&tmp[..length.min(tmp.len())]);
            } else {
                let seg = out[start..start + length].to_vec();
                out.extend_from_slice(&seg);
            }
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_roundtrip() {
        let data: Vec<u8> = (0..10000u32).map(|i| (i * 7 % 251) as u8).collect();
        let packed = compress_store(&data);
        assert_eq!(decompress(&packed).unwrap(), data);
        assert!(packed.len() > data.len());
    }

    #[test]
    fn store_empty() {
        assert_eq!(decompress(&compress_store(&[])).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn decompress_backreference_stream() {
        let mut input = Vec::new();
        input.extend_from_slice(&0x3007_u16.to_le_bytes());
        input.extend_from_slice(&[b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h']);
        input.extend_from_slice(&0xB002_u16.to_le_bytes());
        input.extend_from_slice(&[0x01, 0x01, 0x20]);
        let out = decompress(&input).unwrap();
        assert_eq!(out, b"abcdefghfghf");
    }

    #[test]
    fn decompress_overlapping_run() {
        let mut input = Vec::new();
        input.extend_from_slice(&0x3000_u16.to_le_bytes());
        input.extend_from_slice(&[b'x']);
        input.extend_from_slice(&0xB002_u16.to_le_bytes());
        input.extend_from_slice(&[0x01, 0x00, 0x00]);
        let out = decompress(&input).unwrap();
        assert_eq!(out, b"xxxx");
    }
}
