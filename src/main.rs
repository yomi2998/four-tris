// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod config;
mod game;
mod gui;
mod ini;
mod lznt;
mod render;
mod snap;
mod state;
mod ui;
mod vk;

fn main() {
    gui::run();
}
