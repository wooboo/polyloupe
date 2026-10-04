//! Ways to reach the loupe from outside the app: the menu bar / system tray
//! icon and a global keyboard shortcut.

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use gpui_kit::*;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::state;

#[cfg(target_os = "macos")]
pub const HOTKEY_LABEL: &str = "⌃ Control + ⌥ Option + L";
#[cfg(not(target_os = "macos"))]
pub const HOTKEY_LABEL: &str = "Ctrl + Alt + L";

enum Command {
    ToggleLoupe,
    ShowHome,
    Quit,
}

/// Keeps the tray icon and the hotkey registration alive.
struct Integrations {
    _tray: Option<TrayIcon>,
    _hotkeys: Option<GlobalHotKeyManager>,
}

impl Global for Integrations {}

pub fn init(cx: &mut App) {
    let (tx, rx) = async_channel::unbounded::<Command>();

    let show_loupe = MenuItem::new("Pokaż lupę", true, None);
    let show_home = MenuItem::new("Ustawienia…", true, None);
    let quit = MenuItem::new("Zakończ Polyloupe", true, None);
    let menu = Menu::new();
    menu.append_items(&[
        &show_loupe,
        &show_home,
        &PredefinedMenuItem::separator(),
        &quit,
    ])
    .ok();
    let ids = (
        show_loupe.id().clone(),
        show_home.id().clone(),
        quit.id().clone(),
    );
    let menu_tx = tx.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let command = match event.id {
            id if id == ids.0 => Command::ToggleLoupe,
            id if id == ids.1 => Command::ShowHome,
            id if id == ids.2 => Command::Quit,
            _ => return,
        };
        menu_tx.try_send(command).ok();
    }));

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Polyloupe — lupa tłumacząca")
        .with_icon_templated(loupe_icon(64))
        .build()
        .inspect_err(|err| log::error!("creating the tray icon: {err}"))
        .ok();

    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyL);
    let hotkeys = GlobalHotKeyManager::new()
        .and_then(|manager| manager.register(hotkey).map(|()| manager))
        .inspect_err(|err| log::error!("registering the global shortcut: {err}"))
        .ok();
    let hotkey_tx = tx;
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.id == hotkey.id() && event.state == HotKeyState::Pressed {
            hotkey_tx.try_send(Command::ToggleLoupe).ok();
        }
    }));

    cx.set_global(Integrations {
        _tray: tray,
        _hotkeys: hotkeys,
    });

    cx.spawn(async move |cx| {
        while let Ok(command) = rx.recv().await {
            cx.update(|cx| match command {
                Command::ToggleLoupe => state::toggle_loupe(cx),
                Command::ShowHome => state::show_home(cx),
                Command::Quit => cx.quit(),
            });
        }
    })
    .detach();
}

/// A magnifying glass drawn in black on transparent, for use as a template icon.
fn loupe_icon(size: u32) -> Icon {
    let s = size as f32;
    let (cx, cy, radius, ring) = (0.42 * s, 0.42 * s, 0.27 * s, 0.09 * s);
    let handle = (0.62 * s, 0.62 * s, 0.9 * s, 0.9 * s, 0.075 * s);
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let ring_distance = ((px - cx).hypot(py - cy) - radius).abs() - ring / 2.0;
            let handle_distance =
                segment_distance(px, py, handle.0, handle.1, handle.2, handle.3) - handle.4;
            let alpha = (0.5 - ring_distance.min(handle_distance)).clamp(0.0, 1.0);
            rgba[((y * size + x) * 4 + 3) as usize] = (alpha * 255.0) as u8;
        }
    }
    Icon::from_rgba(rgba, size, size).expect("valid icon")
}

fn segment_distance(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (dx, dy) = (bx - ax, by - ay);
    let t = (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    (px - (ax + t * dx)).hypot(py - (ay + t * dy))
}
