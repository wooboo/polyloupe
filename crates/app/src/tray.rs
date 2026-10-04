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

    let builder = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Polyloupe — lupa tłumacząca");
    // macOS tints template icons to match the menu bar; elsewhere the icon
    // needs its own background to stay visible on light and dark taskbars.
    #[cfg(target_os = "macos")]
    let builder =
        builder.with_icon_templated(icon(include_bytes!("../assets/tray-icon-template.png")));
    #[cfg(not(target_os = "macos"))]
    let builder = builder.with_icon(icon(include_bytes!("../assets/tray-icon.png")));
    let tray = builder
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

/// Decode one of the icons rendered from `assets/icon.svg` (the template one
/// is black on transparent, for macOS to tint).
fn icon(png: &[u8]) -> Icon {
    let image = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .expect("bundled icon is a valid PNG")
        .into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).expect("valid icon")
}
