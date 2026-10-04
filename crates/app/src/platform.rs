//! Small OS integrations that GPUI does not provide.

/// Global mouse position in logical pixels (top-left origin of the main display).
pub fn cursor_position() -> Option<(f32, f32)> {
    imp::cursor_position()
}

/// Whether the app may read other apps' windows from screenshots.
pub fn screen_capture_allowed() -> bool {
    imp::screen_capture_allowed()
}

/// Ask the OS for screen capture permission (shows the system prompt once).
pub fn request_screen_capture() {
    imp::request_screen_capture()
}

/// Open the system settings page where screen capture permission is granted.
pub fn open_screen_capture_settings() {
    imp::open_screen_capture_settings()
}

#[cfg(target_os = "macos")]
mod imp {
    use objc2_core_graphics::{
        CGEvent, CGPreflightScreenCaptureAccess, CGRequestScreenCaptureAccess,
    };

    pub fn cursor_position() -> Option<(f32, f32)> {
        let event = CGEvent::new(None)?;
        let point = CGEvent::location(Some(&event));
        Some((point.x as f32, point.y as f32))
    }

    pub fn screen_capture_allowed() -> bool {
        CGPreflightScreenCaptureAccess()
    }

    pub fn request_screen_capture() {
        CGRequestScreenCaptureAccess();
    }

    pub fn open_screen_capture_settings() {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .spawn()
            .ok();
    }
}

#[cfg(windows)]
mod imp {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    // TODO: GetCursorPos reports physical pixels on per-monitor-DPI-aware
    // processes; map to GPUI's logical coordinates (see docs/platforms.md).
    pub fn cursor_position() -> Option<(f32, f32)> {
        let mut point = POINT { x: 0, y: 0 };
        (unsafe { GetCursorPos(&mut point) } != 0).then(|| (point.x as f32, point.y as f32))
    }

    pub fn screen_capture_allowed() -> bool {
        true
    }

    pub fn request_screen_capture() {}

    pub fn open_screen_capture_settings() {}
}

#[cfg(target_os = "linux")]
mod imp {
    /// Works on X11 (and XWayland for X11 windows only). Wayland does not
    /// expose a global cursor position; callers fall back to the screen centre.
    pub fn cursor_position() -> Option<(f32, f32)> {
        let (conn, screen) = xcb::Connection::connect(None).ok()?;
        let root = conn.get_setup().roots().nth(screen as usize)?.root();
        let cookie = conn.send_request(&xcb::x::QueryPointer { window: root });
        let reply = conn.wait_for_reply(cookie).ok()?;
        Some((reply.root_x() as f32, reply.root_y() as f32))
    }

    /// On Wayland the screenshot portal asks the user itself.
    pub fn screen_capture_allowed() -> bool {
        true
    }

    pub fn request_screen_capture() {}

    pub fn open_screen_capture_settings() {}
}
