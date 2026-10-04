# Platforms: what works, what is risky

As of October 2026. macOS is implemented and verified. Windows and Linux were
analysed by reading the code of the dependencies (GPUI `gpui-pre 0.3.7`,
`xcap 0.9`, `tray-icon 0.26`, `global-hotkey 0.8`) but have **not been run
yet** — the “Manual checks” section at the end is the test list.

## Key decision: a loupe over a frozen screen

The loupe is not a transparent window over the live screen. When it is taken
out, Polyloupe takes a screenshot of the display under the cursor and shows it
in a full-screen window; the loupe moves over that screenshot. This removes the
three hardest cross-platform problems:

| Problem with a live loupe | With a frozen screen |
|---|---|
| The loupe captures itself — its own window must be excluded from capture (macOS: ScreenCaptureKit filter, Windows: `WDA_EXCLUDEFROMCAPTURE`, Linux: no good way) | The screenshot is taken before the window appears |
| The window must let clicks through (GPUI has no API for it — native calls on every platform) | The window takes mouse and keyboard input, which is what we want |
| The scroll wheel goes to the app underneath, so it cannot zoom without global event interception | The wheel zooms, arrow keys move |
| OCR every frame or every time the mouse stops | OCR once over the whole screen (~0.3 s), then translate blocks |

The cost: the screen does not change while reading (video stops, the page
cannot be scrolled). For someone who “takes out the loupe, reads, puts it away”
this feels natural. A live mode can be added later on macOS and Windows.

## Feature matrix

| Feature | macOS | Windows | Linux X11 | Linux Wayland |
|---|---|---|---|---|
| Full-screen loupe window | ✅ GPUI `PopUp` (`NSPopUpWindowLevel`) | 🟡 GPUI `PopUp` (`WS_EX_TOPMOST`), untested | 🟠 GPUI `PopUp` is an *override-redirect* window — it **does not get keyboard focus** from the window manager; Esc/arrows may not work without `XSetInputFocus`/a grab | 🔴 `PopUp` is a regular toplevel that cannot be positioned. Needs full screen (`toggle_fullscreen`) or `WindowKind::LayerShell` (wlroots, KDE — **not GNOME**) |
| Screenshot | ✅ `xcap` → `CGWindowListCreateImage` (⚠️ deprecated since macOS 15, works on 26; target: `SCScreenshotManager`) | 🟡 `xcap` (GDI/DXGI) | 🟡 `xcap` (XCB) | 🟠 `xcap` via the portal (`org.freedesktop.portal.Screenshot`) or wlr-screencopy; the portal may ask the user every time |
| Permissions | ✅ *Screen Recording*: checked with `CGPreflightScreenCaptureAccess`, the main window links to System Settings | not needed | not needed | portal dialog |
| Cursor position (choosing the display and loupe position) | ✅ `CGEventGetLocation` | 🟡 `GetCursorPos` — ⚠️ physical vs GPUI's logical pixels with DPI scaling | 🟡 `xcb QueryPointer` | 🔴 Wayland exposes no global cursor position → the loupe starts at the screen centre and then follows the mouse inside its window |
| Text recognition | ✅ Apple Vision | ❌ to do: `Windows.Media.Ocr` (pl/da/sv/fi/nb language packs exist) | ❌ to do: Tesseract (`pol`, `dan`, `swe`, `fin`, `nor`) | as X11 |
| Offline translation | ✅ `fxtranslate` (pure Rust + C++ SIMD kernel) | 🟡 same (x86_64 AVX2) | 🟡 same | 🟡 same |
| Tray / menu bar icon | ✅ `tray-icon` (NSStatusItem) | 🟡 `tray-icon` (needs a Win32 event loop on the thread — GPUI runs one) | 🟡 `tray-icon` with the `ksni` backend (StatusNotifierItem over D-Bus, no GTK). GNOME shows it only with the AppIndicator extension | as X11 |
| Global shortcut | ✅ `global-hotkey` (Carbon, no extra permissions) | 🟡 `RegisterHotKey` | 🟡 X11 grab | 🔴 not supported by `global-hotkey`; needs the `GlobalShortcuts` portal (KDE; GNOME 48–50+) |
| Dock / taskbar | ✅ clicking the Dock icon opens the main window (`on_reopen`) | 🟡 check whether GPUI reports re-launch | depends on the desktop | as X11 |
| Top menu | ✅ GPUI application menu | n/a | n/a | n/a |

✅ works, verified · 🟡 should work, untested · 🟠 known problem with a workaround · 🔴 needs a separate path · ❌ not implemented

## Biggest risks and plan

1. **Wayland (GNOME).** No global cursor, no overlay layers (layer-shell),
   screenshots and shortcuts only through portals with dialogs. Plan: a
   full-screen loupe window, screenshots via the Screenshot portal, the shortcut
   via the GlobalShortcuts portal. Without a portal, the tray and the main-window
   button remain.
2. **Keyboard focus on X11.** Override-redirect windows get no focus. Plan:
   call `XSetInputFocus` explicitly after opening, or open a regular full-screen
   window instead. GPUI exposes the window handle (`raw-window-handle`).
3. **DPI on Windows.** `xcap` and `GetCursorPos` report physical pixels, GPUI
   works in logical ones. `capture.rs` derives the scale from the screenshot
   size, but display selection and loupe position may be off on 125–200%
   monitors.
4. **macOS `CGWindowListCreateImage`.** Deprecated; Apple may disable it. Plan:
   our own backend on `SCScreenshotManager` (macOS 14+).
5. **OCR outside macOS.** Windows.Media.Ocr has good language support; on Linux
   Tesseract is slower and weaker on small screen text. Also worth trying
   `ocrs` (pure Rust), though its alphabet may not cover Polish and Nordic
   characters.

## Manual checks (Windows and Linux)

On each platform:

1. `cargo run --release` — the main window opens and the models download.
2. `cargo run --release -- --image docs/sample.png` — the loupe over an image,
   without a screenshot (checks the window, rendering, mouse, wheel, keyboard,
   translation; OCR is macOS-only for now).
3. The “Pokaż lupę” (Show loupe) button: the main window disappears and the
   loupe appears over the screen under the cursor; after closing, the main
   window comes back.
4. Ctrl+Alt+L from another app: the loupe appears over that app; after closing,
   focus returns to it.
5. The tray icon menu: show loupe / settings / quit.
6. Keyboard in the loupe: Esc, arrows, +/−, Space.
7. Two displays, one scaled to 150%: the loupe opens on the right display,
   under the cursor.
