# Accessibility and interaction (WCAG 2.1 AA)

Audience: older people. Principle: using it should feel like using a real
magnifying glass. Nothing to select, no shortcuts that must be remembered.

## Taking out the loupe

- a large **“Pokaż lupę”** (Show loupe) button in the main window (72 px tall),
- the loupe icon in the **menu bar** (macOS) or **tray** (Windows, Linux) → Show loupe,
- clicking the app icon in the **Dock / taskbar** opens the main window,
- the **shortcut** Ctrl+Alt+L (⌃⌥L on macOS) works in any app; pressing it again puts the loupe away.

## The loupe

| Action | Mouse | Keyboard |
|---|---|---|
| Move the loupe | move the mouse | arrow keys (Shift for small steps) |
| Magnify / reduce | scroll wheel | `+` / `−` |
| Show the original | — | Space (toggles) |
| Put the loupe away | click | Esc or Enter |

A caption under the loupe always shows e.g. “Dansk → Polski · 2×” and the
current state (“Reading text…”). A bar at the bottom of the screen explains the
controls; it moves to the top when the loupe is low on the screen.

## WCAG 2.1 AA checklist

| Criterion | Status |
|---|---|
| 1.4.3 Contrast (4.5:1) | ✅ Main window: secondary text measured at 4.74:1, the rest is black on white. Translations in the loupe keep the original text colour only at ≥ 4.5:1 against the background, otherwise black or white. For any background one of those reaches ≥ 4.58:1 (test `foreground_always_meets_wcag_aa`) |
| 1.4.11 Non-text contrast (3:1) | ✅ Loupe frame: a white ring plus a dark ring, visible on any background. Caption pills: white on #111 with a white outline |
| 1.4.4 Resize text | 🟡 Default font size raised to 18 px; no in-app text size setting yet |
| 2.1.1 Keyboard | ✅ The loupe is fully keyboard operable; the main window uses GPUI Kit components (Tab, Space, Enter) |
| 2.4.7 Focus visible | 🟡 Provided by GPUI Kit — to be checked live |
| 3.1.1 Language of page | 🟡 Polish-only interface; no language declared for screen readers |
| 3.3.2 Labels | ✅ Text labels on all controls |
| 4.1.2 Name, role, value | 🟡 GPUI Kit exposes AccessKit roles, names and states. The loupe overlay is an image — screen readers cannot read it (consider reading the translation aloud) |

## Ideas

- Read the translation under the loupe aloud (system TTS).
- Loupe size and font size settings in the main window.
- Interface language following the target language.
- High-contrast mode: translations always black on yellow.
