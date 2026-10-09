// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

use std::fmt::Write as _;

#[derive(Clone)]
pub struct Ini {
    pub path: std::path::PathBuf,
    pub lines: Vec<String>,
}

impl Ini {
    pub fn load(path: &std::path::Path) -> Ini {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        Ini { path: path.to_path_buf(), lines: text.lines().map(str::to_string).collect() }
    }

    fn section_range(&self, section: &str) -> Option<(usize, usize)> {
        let mut start = None;
        let mut end = self.lines.len();
        for (i, line) in self.lines.iter().enumerate() {
            let t = line.trim();
            if t.starts_with('[') && t.ends_with(']') {
                if start.is_some() {
                    end = i;
                    break;
                }
                if t[1..t.len() - 1].eq_ignore_ascii_case(section) {
                    start = Some(i);
                }
            }
        }
        start.map(|s| (s, end))
    }

    pub fn read(&self, section: &str, key: &str) -> Option<String> {
        let (s, e) = self.section_range(section)?;
        for line in &self.lines[s + 1..e] {
            let t = line.trim();
            if t.starts_with('#') || t.starts_with(';') || t.is_empty() {
                continue;
            }
            if let Some(eq) = t.find('=') {
                if t[..eq].trim().eq_ignore_ascii_case(key) {
                    return Some(t[eq + 1..].trim().to_string());
                }
            }
        }
        None
    }

    pub fn read_str(&self, section: &str, key: &str, default: &str) -> String {
        self.read(section, key).unwrap_or_else(|| default.to_string())
    }

    pub fn read_num<T: std::str::FromStr>(&self, section: &str, key: &str, default: T) -> T {
        self.read(section, key)
            .and_then(|v| v.trim().parse::<T>().ok())
            .unwrap_or(default)
    }

    pub fn read_bool(&self, section: &str, key: &str, default: bool) -> bool {
        match self.read(section, key) {
            Some(v) => v.trim().eq_ignore_ascii_case("true"),
            None => default,
        }
    }

    pub fn write(&mut self, section: &str, key: &str, value: &str) {
        if let Some((s, e)) = self.section_range(section) {
            for i in s + 1..e {
                let t = self.lines[i].trim().to_string();
                if t.starts_with('#') || t.starts_with(';') || t.is_empty() {
                    continue;
                }
                if let Some(eq) = t.find('=') {
                    if t[..eq].trim().eq_ignore_ascii_case(key) {
                        self.lines[i] = format!("{key}={value}");
                        return;
                    }
                }
            }
            self.lines.insert(s + 1, format!("{key}={value}"));
        } else {
            self.lines.push(format!("[{section}]"));
            self.lines.push(format!("{key}={value}"));
        }
        self.flush();
    }

    fn flush(&self) {
        let mut out = self.lines.join("\n");
        out.push('\n');
        let _ = std::fs::write(&self.path, out);
    }
}

pub fn bool_str(v: bool) -> &'static str {
    if v { "True" } else { "False" }
}

pub fn format_num(v: f64) -> String {
    if (v - v.round()).abs() < f64::EPSILON {
        format!("{}", v.round() as i64)
    } else {
        let mut s = String::new();
        let _ = write!(s, "{v}");
        s
    }
}
