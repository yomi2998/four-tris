// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 github.com/yomi2998 (Rust port of fourtris)

#[cfg(all(unix, feature = "x11-capture"))]
pub fn capture_monitor_under_cursor() -> Option<(crate::render::Bitmap, i32, i32)> {
    use x11rb::connection::Connection;
    use x11rb::protocol::randr;
    use x11rb::protocol::xproto::{self, ImageFormat};

    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let pointer = xproto::query_pointer(&conn, root).ok()?.reply().ok()?;
    let (mx, my) = (pointer.root_x as i32, pointer.root_y as i32);

    let monitors = randr::get_monitors(&conn, root, true).ok()?.reply().ok()?;
    let target = monitors.monitors.iter().find(|m| {
        mx >= m.x as i32
            && mx < m.x as i32 + m.width as i32
            && my >= m.y as i32
            && my < m.y as i32 + m.height as i32
    })?;
    let (x, y, w, h) = (target.x, target.y, target.width, target.height);

    let image = xproto::get_image(&conn, ImageFormat::Z_PIXMAP, root, x, y, w, h, u32::MAX)
        .ok()?
        .reply()
        .ok()?;

    let lsb = conn.setup().image_byte_order == x11rb::protocol::xproto::ImageOrder::LSB_FIRST;
    let stride = image.data.len() / (h as usize).max(1);
    let mut bitmap = crate::render::Bitmap::new(w as usize, h as usize);
    for row in 0..h as usize {
        for col in 0..w as usize {
            let base = row * stride + col * 4;
            if base + 3 >= image.data.len() {
                continue;
            }
            let (r, g, b) = if lsb {
                (image.data[base + 2], image.data[base + 1], image.data[base])
            } else {
                (image.data[base + 1], image.data[base + 2], image.data[base + 3])
            };
            bitmap.px[row * w as usize + col] = crate::render::pack_rgb(r, g, b);
        }
    }
    Some((bitmap, x as i32, y as i32))
}

#[cfg(target_os = "windows")]
pub fn capture_monitor_under_cursor() -> Option<(crate::render::Bitmap, i32, i32)> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
        GetDIBits, GetMonitorInfoW, MonitorFromPoint, ReleaseDC, SelectObject, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, MONITORINFO, MONITOR_DEFAULTTONEAREST, SRCCOPY,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    unsafe {
        let mut pt = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut pt) == 0 {
            return None;
        }
        let monitor = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        if monitor.is_null() {
            return None;
        }
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut info) == 0 {
            return None;
        }
        let (x, y) = (info.rcMonitor.left, info.rcMonitor.top);
        let (w, h) = (info.rcMonitor.right - x, info.rcMonitor.bottom - y);
        if w <= 0 || h <= 0 {
            return None;
        }

        let screen_dc = GetDC(std::ptr::null_mut());
        if screen_dc.is_null() {
            return None;
        }
        let mem_dc = CreateCompatibleDC(screen_dc);
        let bitmap = if mem_dc.is_null() {
            std::ptr::null_mut()
        } else {
            CreateCompatibleBitmap(screen_dc, w, h)
        };
        let mut result: Option<(crate::render::Bitmap, i32, i32)> = None;
        if !bitmap.is_null() {
            let old = SelectObject(mem_dc, bitmap);
            if BitBlt(mem_dc, 0, 0, w, h, screen_dc, x, y, SRCCOPY) != 0 {
                let mut bmi: BITMAPINFO = std::mem::zeroed();
                bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                bmi.bmiHeader.biWidth = w;
                bmi.bmiHeader.biHeight = -h;
                bmi.bmiHeader.biPlanes = 1;
                bmi.bmiHeader.biBitCount = 32;
                bmi.bmiHeader.biCompression = BI_RGB;
                let mut buf = vec![0u8; w as usize * h as usize * 4];
                let lines = GetDIBits(
                    screen_dc,
                    bitmap,
                    0,
                    h as u32,
                    buf.as_mut_ptr().cast(),
                    &mut bmi,
                    DIB_RGB_COLORS,
                );
                if lines == h {
                    let mut bmp = crate::render::Bitmap::new(w as usize, h as usize);
                    for (i, px) in bmp.px.iter_mut().enumerate() {
                        let base = i * 4;
                        *px = crate::render::pack_rgb(buf[base + 2], buf[base + 1], buf[base]);
                    }
                    result = Some((bmp, x, y));
                }
            }
            SelectObject(mem_dc, old);
            DeleteObject(bitmap);
        }
        if !mem_dc.is_null() {
            DeleteDC(mem_dc);
        }
        ReleaseDC(std::ptr::null_mut(), screen_dc);
        result
    }
}

#[cfg(not(any(all(unix, feature = "x11-capture"), target_os = "windows")))]
pub fn capture_monitor_under_cursor() -> Option<(crate::render::Bitmap, i32, i32)> {
    None
}

#[allow(dead_code)]
pub enum SlurpOutcome {
    Selected(crate::render::Bitmap),
    Cancelled,
    Unavailable,
}

#[allow(dead_code)]
fn cli_capture(bin: &str, args: &[&str]) -> Option<Option<crate::render::Bitmap>> {
    let path = std::env::temp_dir().join(format!("four-tris-snap-{}.png", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let mut cmd = std::process::Command::new(bin);
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    for arg in args {
        cmd.arg(arg);
    }
    let status = cmd.arg(&path).status().ok()?;
    let result = (|| {
        if !status.success() {
            return None;
        }
        let data = std::fs::read(&path).ok()?;
        let img = image::load_from_memory(&data).ok()?;
        let rgba = img.to_rgba8();
        Some(crate::render::Bitmap::from_rgba8(
            rgba.width() as usize,
            rgba.height() as usize,
            rgba.as_raw(),
        ))
    })();
    let _ = std::fs::remove_file(&path);
    Some(result)
}

#[cfg(target_os = "linux")]
fn grim_capture(args: &[&str]) -> Option<Option<crate::render::Bitmap>> {
    cli_capture("grim", args)
}

#[cfg(target_os = "linux")]
fn slurp_grim_capture() -> SlurpOutcome {
    let out = std::process::Command::new("slurp")
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(out) = out else {
        return SlurpOutcome::Unavailable;
    };
    let geom = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if geom.is_empty() {
        return SlurpOutcome::Cancelled;
    }
    match grim_capture(&["-g", &geom]) {
        Some(Some(img)) => SlurpOutcome::Selected(img),
        Some(None) => SlurpOutcome::Cancelled,
        None => SlurpOutcome::Unavailable,
    }
}

#[cfg(target_os = "linux")]
async fn portal_screenshot() -> Option<crate::render::Bitmap> {
    let request = ashpd::desktop::screenshot::Screenshot::request()
        .interactive(false)
        .modal(false)
        .send()
        .await
        .ok()?;
    let response = request.response().ok()?;
    let uri = response.uri().as_str();
    let path = uri.strip_prefix("file://")?;
    let data = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&data).ok()?;
    let rgba = img.to_rgba8();
    Some(crate::render::Bitmap::from_rgba8(
        rgba.width() as usize,
        rgba.height() as usize,
        rgba.as_raw(),
    ))
}

#[cfg(target_os = "linux")]
fn wayland_fallback() -> Option<crate::render::Bitmap> {
    grim_capture(&[]).flatten().or_else(|| {
        (|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            rt.block_on(async {
                match tokio::time::timeout(std::time::Duration::from_secs(10), portal_screenshot())
                    .await
                {
                    Ok(inner) => inner,
                    Err(_) => None,
                }
            })
        })()
    })
}

pub fn spawn_region_picking() -> Option<std::sync::mpsc::Receiver<SlurpOutcome>> {
    #[cfg(target_os = "linux")]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(slurp_grim_capture());
        });
        return Some(rx);
    }
    #[cfg(target_os = "macos")]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let outcome = match cli_capture("screencapture", &["-i"]) {
                Some(Some(img)) => SlurpOutcome::Selected(img),
                Some(None) => SlurpOutcome::Cancelled,
                None => SlurpOutcome::Unavailable,
            };
            let _ = tx.send(outcome);
        });
        return Some(rx);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

pub fn spawn_fallback_capture() -> Option<std::sync::mpsc::Receiver<Option<crate::render::Bitmap>>> {
    #[cfg(target_os = "linux")]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(wayland_fallback());
        });
        return Some(rx);
    }
    #[cfg(target_os = "macos")]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(cli_capture("screencapture", &["-x"]).flatten());
        });
        return Some(rx);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}
