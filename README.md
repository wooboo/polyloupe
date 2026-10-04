<img src="assets/icon.svg" width="64" height="64" alt="">

# Polyloupe

A magnifying glass that translates the text on your screen. Take it out, hover
over some text, and the text under the glass is already translated — in the
same place, in the same colours. Scroll to magnify. Designed for older people:
no selecting, no copying, nothing to remember.

![The loupe over Danish text, showing it in Polish](docs/img/loupe-da.png)

- **Offline.** Translation runs on your computer with the
  [Firefox Translations](https://github.com/mozilla/translations) models; nothing
  on your screen is sent anywhere.
- **Languages:** Polish, English, Danish, Swedish, Finnish, Norwegian (Bokmål), in any pair.
- **Easy to reach:** a big button in the main window, a menu bar / tray icon,
  the Dock or taskbar, and a global shortcut (Ctrl+Alt+L, ⌃⌥L on macOS).
- **Accessible:** aims at WCAG 2.1 AA; the loupe works with the keyboard alone.
  See [docs/accessibility.md](docs/accessibility.md).

> **Status: early prototype.** The full pipeline works on macOS. Windows and
> Linux build in CI but are untested, and text recognition is macOS-only so far.
> The interface is in Polish for now.

## Using the loupe

| Action | Mouse | Keyboard |
|---|---|---|
| Move the loupe | move the mouse | arrow keys (Shift for small steps) |
| Magnify | scroll wheel | `+` / `−` |
| Show the original text | — | Space (toggles) |
| Put the loupe away | click | Esc or Enter |

## Building

Requires a recent stable Rust toolchain.

```sh
cargo run --release
```

On first start the app downloads the translation models (about 25 MB per
language). On macOS it needs the *Screen Recording* permission; the main window
links to the right System Settings page.

On Linux, install the GPUI and screen capture dependencies first (see
[the CI workflow](.github/workflows/ci.yml) for the exact Debian/Ubuntu packages).

To work on the loupe without capturing the screen, open it over an image:

```sh
cargo run --release -- --image docs/sample.png --at 330,340 --zoom 2
```

## How it works

When you take out the loupe, Polyloupe freezes the display under the cursor
(a screenshot) and shows it full screen, with the loupe on top. In the
background it recognizes all text on the screenshot, joins lines into
paragraphs, detects each paragraph's language and translates it — whatever is
under the loupe first. Inside the loupe each translated paragraph is painted
over the original, on a background sampled from around it.

Freezing the screen avoids the hardest cross-platform problems of a live
overlay (the loupe capturing itself, click-through windows, intercepting the
scroll wheel). See [docs/platforms.md](docs/platforms.md) for the reasoning and
the per-platform status.

| Crate | Contents |
|---|---|
| [`crates/app`](crates/app) (`polyloupe`) | The [GPUI Kit](https://gpui-kit.com) app: main window, loupe, tray, shortcut, screen capture |
| [`crates/ocr`](crates/ocr) | Text recognition (Apple Vision) and paragraph grouping |
| [`crates/translate`](crates/translate) | Offline translation ([`fxtranslate`](https://crates.io/crates/fxtranslate)) and language detection ([`lingua`](https://crates.io/crates/lingua)) |

## Platform status

| | macOS | Windows | Linux |
|---|---|---|---|
| Loupe, translation, tray, shortcut | ✅ | 🟡 untested | 🟡 untested (Wayland has limits) |
| Text recognition | ✅ Apple Vision | ❌ planned: Windows.Media.Ocr | ❌ planned: Tesseract |

Details and risks: [docs/platforms.md](docs/platforms.md).
Translation engines and free cloud providers: [docs/translation.md](docs/translation.md).

## Known issues

- Translation models are downloaded from Firefox's CDN, which is provisioned for
  Firefox rather than third-party apps. They need to be mirrored before a release.
- macOS screen capture uses the deprecated `CGWindowListCreateImage`; it should
  move to ScreenCaptureKit.
- Apple Vision sometimes drops diacritics (e.g. Finnish *lääkkeesi* → *laakkeesi*).

## Contributing

Issues and pull requests are welcome — especially testing on Windows and Linux
(see the manual checklist in [docs/platforms.md](docs/platforms.md)) and
feedback from older users.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

The translation models downloaded at runtime are Mozilla's and are distributed
under the [MPL-2.0](https://github.com/mozilla/firefox-translations-models).
