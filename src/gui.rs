// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{ModifiersState, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

use crate::app::{vk_from_keycode, App, SnapState};

enum SnapPhase {
    Hiding(Instant),
    Picking(std::sync::mpsc::Receiver<crate::snap::SlurpOutcome>),
    Capturing(std::sync::mpsc::Receiver<Option<crate::render::Bitmap>>),
    Selecting,
}

struct GuiState {
    app: Option<App>,
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    modifiers: ModifiersState,
    snap_phase: Option<SnapPhase>,
    is_x11: bool,
}

impl GuiState {
    fn app_animating(app: &App) -> bool {
        app.keys.iter().any(|k| k.pressed)
            || crate::app::gravity_step(app.game.gravity).is_finite()
            || app.comment_animating()
            || app.transition.is_some()
            || app.screen == crate::app::Screen::Settings
    }

    fn draw_and_present(&mut self) {
        let Some(app) = self.app.as_mut() else { return };
        let Some(window) = &self.window else { return };
        let Some(surface) = &mut self.surface else { return };

        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        if app.canvas.w != size.width as usize || app.canvas.h != size.height as usize {
            app.canvas.resize(size.width as usize, size.height as usize);
            app.layout_update(size.width as f64, size.height as f64, app.hidpi);
            app.game.changed = true;
        }
        app.redraw_pending = false;
        app.draw_frame();
        if surface
            .resize(
                NonZeroU32::new(size.width).unwrap(),
                NonZeroU32::new(size.height).unwrap(),
            )
            .is_err()
        {
            return;
        }
        let Ok(mut buffer) = surface.buffer_mut() else { return };
        let len = buffer.len().min(app.canvas.px.len());
        buffer[..len].copy_from_slice(&app.canvas.px[..len]);
        let _ = buffer.present();
    }

    fn sync_window(&mut self) {
        let Some(app) = self.app.as_ref() else { return };
        let Some(window) = &self.window else { return };
        let logical_w = app.wsize[0] * app.user_scale;
        let logical_h = app.wsize[1] * app.user_scale;
        let _ = window.request_inner_size(LogicalSize::new(logical_w, logical_h));
    }

    fn begin_snap_capture(&mut self) {
        let Some(app) = self.app.as_mut() else { return };
        app.snap_request = false;
        if self.is_x11 || cfg!(windows) {
            let Some(window) = &self.window else { return };
            window.set_visible(false);
            self.snap_phase = Some(SnapPhase::Hiding(Instant::now()));
            return;
        }
        match crate::snap::spawn_region_picking() {
            Some(rx) => self.snap_phase = Some(SnapPhase::Picking(rx)),
            None => self.abort_snap_capture(),
        }
    }

    fn start_snap_capture(&mut self) {
        if let Some((image, mx, my)) = crate::snap::capture_monitor_under_cursor() {
            self.finish_snap_capture(image, Some((mx, my)));
            return;
        }
        match crate::snap::spawn_fallback_capture() {
            Some(rx) => self.snap_phase = Some(SnapPhase::Capturing(rx)),
            None => self.abort_snap_capture(),
        }
    }

    fn poll_snap_capture(&mut self) {
        let picked = match &self.snap_phase {
            Some(SnapPhase::Picking(rx)) => match rx.try_recv() {
                Ok(outcome) => Some(outcome),
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    Some(crate::snap::SlurpOutcome::Unavailable)
                }
            },
            _ => None,
        };
        if let Some(outcome) = picked {
            match outcome {
                crate::snap::SlurpOutcome::Selected(image) => {
                    self.snap_phase = None;
                    if let Some(app) = self.app.as_mut() {
                        app.fill_board_from_bitmap(image);
                    }
                }
                crate::snap::SlurpOutcome::Cancelled => {
                    self.snap_phase = None;
                }
                crate::snap::SlurpOutcome::Unavailable => match crate::snap::spawn_fallback_capture() {
                    Some(rx) => self.snap_phase = Some(SnapPhase::Capturing(rx)),
                    None => self.abort_snap_capture(),
                },
            }
            return;
        }
        let Some(SnapPhase::Capturing(rx)) = &self.snap_phase else { return };
        let received = match rx.try_recv() {
            Ok(image) => Some(image),
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(None),
        };
        let Some(received) = received else { return };

        match received {
            Some(image) => self.finish_snap_capture(image, None),
            None => self.abort_snap_capture(),
        }
    }

    fn finish_snap_capture(&mut self, image: crate::render::Bitmap, origin: Option<(i32, i32)>) {
        let Some(app) = self.app.as_mut() else { return };
        let Some(window) = &self.window else { return };
        let monitor = match origin {
            Some((mx, my)) => window.current_monitor().or_else(|| {
                window.available_monitors().find(|m| {
                    let pos = m.position();
                    (mx..mx + image.w as i32).contains(&pos.x)
                        && (my..my + image.h as i32).contains(&pos.y)
                })
            }),
            None => window.current_monitor(),
        };
        if let Some(monitor) = monitor {
            window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
        }
        app.snap = Some(SnapState { image, drag_start: None, drag_cur: (0.0, 0.0) });
        window.set_visible(true);
        self.snap_phase = Some(SnapPhase::Selecting);
    }

    fn abort_snap_capture(&mut self) {
        let Some(app) = self.app.as_mut() else { return };
        let Some(window) = &self.window else { return };
        window.set_visible(true);
        self.snap_phase = None;
        app.game.pending_comment = Some((
            "SNAPSHOT".to_string(),
            "No screen capture method is available on this platform.".to_string(),
            2500,
        ));
    }

    fn end_snap(&mut self, apply: bool) {
        let Some(app) = self.app.as_mut() else { return };
        let Some(window) = &self.window else { return };
        if apply {
            if let Some(snap) = app.snap.take() {
                if let Some((sx, sy)) = snap.drag_start {
                    let (cx, cy) = snap.drag_cur;
                    let x0 = sx.min(cx).round().max(0.0) as usize;
                    let y0 = sy.min(cy).round().max(0.0) as usize;
                    let x1 = (sx.max(cx).round() as usize).min(snap.image.w);
                    let y1 = (sy.max(cy).round() as usize).min(snap.image.h);
                    if x1 > x0 && y1 > y0 {
                        let w = x1 - x0;
                        let h = y1 - y0;
                        let mut cropped = crate::render::Bitmap::new(w, h);
                        for y in 0..h {
                            for x in 0..w {
                                cropped.px[y * w + x] = snap.image.px[(y0 + y) * snap.image.w + (x0 + x)];
                            }
                        }
                        app.fill_board_from_bitmap(cropped);
                    }
                }
            }
        } else {
            app.snap = None;
        }
        window.set_fullscreen(None);
        window.set_visible(true);
        self.snap_phase = None;
        app.game.changed = true;
    }
}

impl ApplicationHandler for GuiState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut app = App::new();
        let attrs = Window::default_attributes()
            .with_title("four-tris")
            .with_inner_size(LogicalSize::new(
                app.wsize[0] * app.cfg.scale,
                app.wsize[1] * app.cfg.scale,
            ))
            .with_min_inner_size(LogicalSize::new(200.0, 300.0));
        let window = Rc::new(event_loop.create_window(attrs).expect("create window"));
        if let Some(bmp) = app.app_icon.take() {
            let mut rgba = Vec::with_capacity(bmp.w * bmp.h * 4);
            for &px in &bmp.px {
                rgba.push(((px >> 16) & 0xFF) as u8);
                rgba.push(((px >> 8) & 0xFF) as u8);
                rgba.push((px & 0xFF) as u8);
                rgba.push(((px >> 24) & 0xFF) as u8);
            }
            if let Ok(icon) = winit::window::Icon::from_rgba(rgba, bmp.w as u32, bmp.h as u32) {
                window.set_window_icon(Some(icon));
            }
        }
        app.hidpi = window.scale_factor();
        app.user_scale = app.cfg.scale;
        app.canvas.scale = app.total_scale();
        let size = window.inner_size();
        app.canvas.resize(size.width as usize, size.height as usize);
        app.layout_update(size.width as f64, size.height as f64, app.hidpi);
        app.game.changed = true;
        app.redraw_pending = true;

        let context = Context::new(window.clone()).expect("softbuffer context");
        let surface = Surface::new(&context, window.clone()).expect("softbuffer surface");
        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        self.app = Some(app);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let mut do_snap_capture = false;
        let mut finish_hide = false;
        let mut sync_request = false;
        #[allow(unused_assignments)]
        let mut wake = None;
        {
            let Some(app) = self.app.as_mut() else { return };
            if app.snap_request {
                app.snap_request = false;
                do_snap_capture = true;
            }
            if let Some(SnapPhase::Hiding(start)) = &self.snap_phase {
                if start.elapsed() >= Duration::from_millis(200) {
                    finish_hide = true;
                }
            }

            let snapping = app.snap.is_some();
            if !snapping {
                app.step();
            }

            if let Some(window) = &self.window {
                let size = window.inner_size();
                if size.width > 0
                    && size.height > 0
                    && (app.canvas.w != size.width as usize || app.canvas.h != size.height as usize)
                {
                    app.canvas.resize(size.width as usize, size.height as usize);
                    app.layout_update(size.width as f64, size.height as f64, app.hidpi);
                    app.game.changed = true;
                    app.redraw_pending = true;
                }
            }

            if app.window_sync_request {
                app.window_sync_request = false;
                sync_request = true;
            }

            if app.redraw_pending || (!self.is_x11 && Self::app_animating(app)) {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            let now = app.now_ms();
            let deadline_ms = if self.snap_phase.is_some() || app.snap.is_some() {
                now + 16.0
            } else {
                let mut deadline_ms = app.t_input;
                let gravity = crate::app::gravity_step(app.game.gravity);
                if gravity.is_finite() {
                    deadline_ms = deadline_ms.min(app.t_gravity);
                }
                if let Some((start, _)) = app.transition {
                    deadline_ms = deadline_ms.min(start + 160.0);
                }
                if let Some((start, duration, _, _, _)) = app.game.active_comment {
                    if app.comment_animating() {
                        deadline_ms = deadline_ms.min((start + duration as f64 + 1.0).max(now));
                    }
                }
                if app.transition.is_some() || app.comment_animating() {
                    deadline_ms = deadline_ms.min(now + 16.0);
                }
                deadline_ms.max(now)
            };
            wake = Some(app.epoch + Duration::from_secs_f64(deadline_ms / 1000.0));
        }

        if do_snap_capture {
            self.begin_snap_capture();
        }

        if finish_hide {
            self.start_snap_capture();
        }
        self.poll_snap_capture();
        if sync_request {
            self.sync_window();
        }

        if let Some(wake) = wake {
            event_loop.set_control_flow(ControlFlow::WaitUntil(wake));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        self.handle_window_event(event_loop, event);
    }
}

impl GuiState {
    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        if self.app.is_none() || self.window.is_none() {
            return;
        }
        let Some(app) = self.app.as_mut() else { return };
        let window = self.window.clone().unwrap();

        let mut drop_end_snap = false;
        let mut cancel_snap = false;
        match event {
            WindowEvent::CloseRequested => {
                if app.screen == crate::app::Screen::Settings {
                    app.close_settings();
                } else {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(_size) => {
                app.redraw_pending = true;
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                app.hidpi = scale_factor;
                app.canvas.scale = app.total_scale();
                let size = window.inner_size();
                app.layout_update(size.width as f64, size.height as f64, app.hidpi);
                app.game.changed = true;
                app.redraw_pending = true;
            }
            WindowEvent::Focused(focused) => {
                app.focused = focused;
                if !focused {
                    for key in app.keys.iter_mut() {
                        key.pressed = false;
                    }
                    app.das_dir = 0;
                    app.paint_session = None;
                    app.drag_slider = None;
                }
            }
            WindowEvent::RedrawRequested => {
                self.draw_and_present();
            }
            WindowEvent::Occluded(false) => {
                app.redraw_pending = true;
            }
            WindowEvent::CursorMoved { position, .. } => {
                app.mouse_phys = (position.x, position.y);
                if app.snap.is_some() {
                    if let Some(snap) = app.snap.as_mut() {
                        if snap.drag_start.is_some() {
                            snap.drag_cur = app.mouse_phys;
                        }
                    }
                    app.redraw_pending = true;
                }
                let total = app.total_scale();
                if total > 0.0 {
                    app.mouse = [
                        (position.x - app.view_off.0) / total,
                        (position.y - app.view_off.1) / total,
                    ];
                }
                app.update_hover();
                app.on_mouse_move();
            }
            WindowEvent::CursorLeft { .. } => {
                app.mouse = [-1.0, -1.0];
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if app.snap.is_some() {
                    match (button, state) {
                        (MouseButton::Left, ElementState::Pressed) => {
                            if let Some(snap) = app.snap.as_mut() {
                                snap.drag_start = Some(app.mouse_phys);
                                snap.drag_cur = app.mouse_phys;
                            }
                        }
                        (MouseButton::Left, ElementState::Released) => {
                            drop_end_snap = app
                                .snap
                                .as_ref()
                                .map_or(false, |s| s.drag_start.is_some());
                        }
                        _ => {}
                    }
                    app.redraw_pending = true;
                } else {
                    match (button, state) {
                        (MouseButton::Left, ElementState::Pressed) => app.on_mouse_down(true),
                        (MouseButton::Right, ElementState::Pressed) => app.on_mouse_down(false),
                        (MouseButton::Middle, ElementState::Pressed) => app.middle_click(),
                        (MouseButton::Left, ElementState::Released) => app.on_mouse_up(true),
                        (MouseButton::Right, ElementState::Released) => app.on_mouse_up(false),
                        _ => {}
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let down = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y < 0.0,
                    MouseScrollDelta::PixelDelta(pos) => pos.y < 0.0,
                };
                let ctrl = self.modifiers.control_key();
                let alt = self.modifiers.alt_key();
                app.on_wheel(down, ctrl, alt);
            }
            WindowEvent::KeyboardInput { event: KeyEvent { state, physical_key, text, repeat, .. }, .. } => {
                if app.snap.is_some() {
                    if state == ElementState::Pressed {
                        cancel_snap = true;
                    }
                } else {
                let vk = match physical_key {
                    PhysicalKey::Code(code) => vk_from_keycode(code),
                    PhysicalKey::Unidentified(_) => 0,
                };
                let ctrl = self.modifiers.control_key();
                let shift = self.modifiers.shift_key();
                let alt = self.modifiers.alt_key();
                match state {
                    ElementState::Pressed => {
                        if !repeat {
                            app.handle_key_down(vk, ctrl, shift, alt);
                            if app.input_box.is_some() {
                                if let Some(t) = &text {
                                    let chars: String = t.chars().collect();
                                    if !chars.is_empty() {
                                        app.input_box_text(&chars);
                                    }
                                }
                            }
                        }
                    }
                    ElementState::Released => {
                        app.handle_key_up(vk);
                    }
                }
                }
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
                app.ctrl_down = mods.state().control_key();
            }
            WindowEvent::DroppedFile(path) => {
                if let Some(bmp) = crate::app::load_bitmap(&path) {
                    app.fill_board_from_bitmap(bmp);
                }
            }
            _ => {}
        }

        if drop_end_snap {
            self.end_snap(true);
        }
        if cancel_snap {
            self.end_snap(false);
        }
    }
}

pub fn run() {
    let mut builder = winit::event_loop::EventLoop::builder();
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    {
        use winit::platform::wayland::EventLoopBuilderExtWayland as _;
        use winit::platform::x11::EventLoopBuilderExtX11 as _;
        match std::env::var("FOUR_TRIS_BACKEND").as_deref() {
            Ok("x11") => {
                builder.with_x11();
            }
            Ok("wayland") => {
                builder.with_wayland();
            }
            _ => {}
        }
    }
    let event_loop = builder.build().expect("event loop");
    let is_x11 = {
        #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
        {
            use winit::platform::x11::EventLoopExtX11 as _;
            event_loop.is_x11()
        }
        #[cfg(not(all(unix, not(target_os = "macos"), not(target_os = "android"))))]
        {
            false
        }
    };
    let mut state = GuiState {
        app: None,
        window: None,
        context: None,
        surface: None,
        modifiers: ModifiersState::empty(),
        snap_phase: None,
        is_x11,
    };
    event_loop.run_app(&mut state).expect("event loop run");
}
