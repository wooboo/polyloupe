# Platformy: co działa, co jest ryzykiem

Stan na październik 2026. macOS jest zaimplementowany i sprawdzony. Windows i Linux
są przeanalizowane w kodzie zależności (GPUI `gpui-pre 0.3.7`, `xcap 0.9`,
`tray-icon 0.26`, `global-hotkey 0.8`), ale **nie były jeszcze uruchomione** —
sekcja „Do sprawdzenia ręcznie” na końcu to lista testów.

## Kluczowa decyzja: lupa nad „zamrożonym” ekranem

Lupa nie jest przezroczystym oknem nad żywym ekranem. Po wyciągnięciu lupy
robimy zrzut monitora pod kursorem i pokazujemy go w oknie na cały ekran; lupa
porusza się po tym zrzucie. To usuwa trzy najtrudniejsze problemy
międzyplatformowe:

| Problem przy „żywej” lupie | Przy zamrożonym ekranie |
|---|---|
| Lupa fotografuje samą siebie — trzeba wykluczać własne okno z przechwytywania (macOS: filtr ScreenCaptureKit, Windows: `WDA_EXCLUDEFROMCAPTURE`, Linux: brak dobrego sposobu) | Zrzut powstaje, zanim okno się pojawi |
| Okno musi przepuszczać kliknięcia (GPUI tego nie ma — natywne wywołania na każdej platformie) | Okno przejmuje mysz i klawiaturę, to pożądane |
| Kółko myszy trafia do aplikacji pod spodem, więc nie da się nim powiększać bez globalnego przechwytywania zdarzeń | Kółko powiększa, strzałki przesuwają |
| OCR co klatkę albo przy każdym zatrzymaniu | OCR raz, na całym ekranie (~0,3 s), potem tłumaczenie bloków |

Koszt: podczas czytania ekran się nie zmienia (wideo stoi, nie da się
przewinąć strony). Dla osoby, która „wyciąga lupę, czyta, odkłada lupę”, jest to
naturalne. Tryb „żywy” można dodać później na macOS i Windows.

## Macierz funkcji

| Element | macOS | Windows | Linux X11 | Linux Wayland |
|---|---|---|---|---|
| Okno lupy na cały ekran | ✅ GPUI `PopUp` (poziom `NSPopUpWindowLevel`) | 🟡 GPUI `PopUp` (`WS_EX_TOPMOST`), nie testowane | 🟠 GPUI `PopUp` = okno *override-redirect* — **nie dostaje fokusu klawiatury** od menedżera okien; Esc/strzałki mogą nie działać bez `XSetInputFocus`/grab | 🔴 `PopUp` to zwykłe okno, nie można go ustawić w zadanym miejscu. Trzeba użyć pełnego ekranu (`toggle_fullscreen`) albo `WindowKind::LayerShell` (wlroots, KDE — **nie GNOME**) |
| Zrzut ekranu | ✅ `xcap` → `CGWindowListCreateImage` (⚠️ przestarzałe od macOS 15, działa na 26; docelowo `SCScreenshotManager`) | 🟡 `xcap` (GDI/DXGI) | 🟡 `xcap` (XCB) | 🟠 `xcap` przez portal (`org.freedesktop.portal.Screenshot`) lub wlr-screencopy; portal może za każdym razem pytać użytkownika |
| Uprawnienia | ✅ „Nagrywanie ekranu”: sprawdzamy `CGPreflightScreenCaptureAccess`, okno główne prowadzi do Ustawień | nie trzeba | nie trzeba | dialog portalu |
| Pozycja kursora (wybór monitora i miejsca lupy) | ✅ `CGEventGetLocation` | 🟡 `GetCursorPos` — ⚠️ piksele fizyczne vs logiczne GPUI przy skalowaniu DPI | 🟡 `xcb QueryPointer` | 🔴 Wayland nie udostępnia globalnej pozycji kursora → lupa startuje na środku ekranu, potem śledzi mysz w swoim oknie |
| OCR | ✅ Apple Vision | ❌ do zrobienia: `Windows.Media.Ocr` (pakiety językowe pl/da/sv/fi/nb dostępne w systemie) | ❌ do zrobienia: Tesseract (`pol`, `dan`, `swe`, `fin`, `nor`) | jak X11 |
| Tłumaczenie offline | ✅ `fxtranslate` (czysty Rust + jądro SIMD w C++) | 🟡 to samo (x86_64 AVX2) | 🟡 to samo | 🟡 to samo |
| Ikona w trayu / pasku menu | ✅ `tray-icon` (NSStatusItem) | 🟡 `tray-icon` (wymaga pętli zdarzeń Win32 w wątku — GPUI ją ma) | 🟡 `tray-icon` z backendem `ksni` (StatusNotifierItem przez D-Bus, bez GTK). GNOME pokazuje go tylko z rozszerzeniem AppIndicator | jak X11 |
| Globalny skrót | ✅ `global-hotkey` (Carbon, bez dodatkowych uprawnień) | 🟡 `RegisterHotKey` | 🟡 X11 grab | 🔴 brak w `global-hotkey`; trzeba portalu `GlobalShortcuts` (KDE; GNOME od 48–50) |
| Dock / pasek zadań | ✅ kliknięcie ikony w Docku otwiera okno główne (`on_reopen`) | 🟡 do sprawdzenia, czy GPUI przekazuje ponowne uruchomienie | ikona w docku zależy od środowiska | jak X11 |
| Menu górne | ✅ menu aplikacji GPUI | nie dotyczy | nie dotyczy | nie dotyczy |

✅ działa i sprawdzone · 🟡 powinno działać, niesprawdzone · 🟠 znany problem z obejściem · 🔴 wymaga osobnej ścieżki · ❌ nie zaimplementowane

## Największe ryzyka i plan

1. **Wayland (GNOME).** Brak globalnego kursora, brak warstw nakładek (layer-shell),
   zrzut i skrót tylko przez portale z dialogami. Plan: okno lupy na pełnym
   ekranie, zrzut przez portal Screenshot, skrót przez portal GlobalShortcuts.
   Jeśli portal nie jest dostępny, zostaje tray i przycisk w oknie głównym.
2. **Fokus klawiatury na X11.** Okno override-redirect nie dostaje fokusu. Plan:
   po otwarciu jawnie `XSetInputFocus` albo zmiana `WindowKind` na zwykłe okno
   pełnoekranowe. Do sprawdzenia, czy GPUI udostępnia uchwyt okna (`raw-window-handle` — tak).
3. **DPI na Windows.** `xcap` i `GetCursorPos` zwracają piksele fizyczne, a GPUI
   pracuje w logicznych. `capture.rs` liczy skalę z wymiarów zrzutu, ale wybór
   monitora i pozycja lupy mogą być przesunięte na monitorach 125–200%.
4. **macOS: `CGWindowListCreateImage`.** Oznaczone jako przestarzałe; Apple może
   je wyłączyć. Plan: własny backend na `SCScreenshotManager` (macOS 14+).
5. **OCR poza macOS.** Windows.Media.Ocr ma dobre wsparcie języków; na Linuksie
   Tesseract jest wolniejszy i słabszy na zrzutach ekranu o małej czcionce.
   Warto przetestować też `ocrs` (czysty Rust), ale jego alfabet może nie
   obejmować polskich i nordyckich znaków.

## Do sprawdzenia ręcznie (Windows i Linux)

Na każdej platformie:

1. `cargo run --release` — otwiera się okno główne, pobierają się słowniki.
2. `cargo run --release -- --image docs/sample.png` — lupa nad obrazkiem bez
   zrzutu ekranu (sprawdza okno, renderowanie, mysz, kółko, klawiaturę, tłumaczenie;
   OCR tylko na macOS).
3. Przycisk „Pokaż lupę”: okno główne znika, lupa pojawia się nad ekranem
   pod kursorem; po zamknięciu wraca okno główne.
4. Skrót Ctrl+Alt+L z innego programu: lupa pojawia się nad tym programem;
   po zamknięciu fokus wraca do niego.
5. Ikona w trayu: menu „Pokaż lupę / Ustawienia… / Zakończ”.
6. Klawiatura w lupie: Esc, strzałki, +/−, spacja.
7. Dwa monitory, w tym jeden ze skalowaniem 150%: lupa na właściwym monitorze,
   pod kursorem.
