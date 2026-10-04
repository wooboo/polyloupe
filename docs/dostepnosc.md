# Dostępność i obsługa (WCAG 2.1 AA)

Grupa docelowa: starsze osoby. Zasada: czynności mają przypominać prawdziwą
lupę. Nie trzeba niczego zaznaczać ani zapamiętywać skrótów.

## Jak się wyciąga lupę

- duży przycisk **„Pokaż lupę”** w oknie głównym (wysokość 72 px),
- ikona lupy na **pasku menu** (macOS) lub w **trayu** (Windows, Linux) → „Pokaż lupę”,
- kliknięcie ikony w **Docku / na pasku zadań** otwiera okno główne,
- **skrót** Ctrl+Alt+L (⌃⌥L na macOS) działa w każdym programie i ponownie naciśnięty chowa lupę.

## Lupa

| Czynność | Mysz | Klawiatura |
|---|---|---|
| Przesuń lupę | ruch myszy | strzałki (z Shift — drobniej) |
| Powiększ / pomniejsz | kółko myszy | `+` / `−` |
| Pokaż oryginał | — | spacja (przełącza) |
| Odłóż lupę | kliknięcie | Esc lub Enter |

Pod lupą stale widać podpis: „Dansk → Polski · 2×” oraz stan („Odczytuję tekst…”).
Na dole ekranu jest pasek z instrukcją; przenosi się na górę, kiedy lupa jest nisko.

## Lista kontrolna WCAG 2.1 AA

| Kryterium | Stan |
|---|---|
| 1.4.3 Kontrast tekstu (4,5:1) | ✅ Okno główne: tekst pomocniczy 4,74:1 (zmierzone), reszta czarny na białym. Tłumaczenie w lupie: zostawiamy oryginalny kolor tekstu tylko przy kontraście ≥ 4,5:1 z tłem, inaczej czarny lub biały. Dla dowolnego tła jeden z nich daje ≥ 4,58:1 (test `foreground_always_meets_wcag_aa`) |
| 1.4.11 Kontrast elementów (3:1) | ✅ Ramka lupy: biały pierścień + ciemny pierścień, widoczna na każdym tle. Paski z podpisami: biały na #111 z białą obwódką |
| 1.4.4 Powiększanie tekstu | 🟡 Domyślny rozmiar czcionki podniesiony do 18 px; brak ustawienia rozmiaru czcionki w aplikacji |
| 2.1.1 Klawiatura | ✅ Lupa w pełni obsługiwana klawiaturą; okno główne — komponenty GPUI Kit (Tab, spacja, Enter) |
| 2.4.7 Widoczny fokus | 🟡 Zapewniany przez GPUI Kit — do sprawdzenia na żywo |
| 3.1.1 Język strony | 🟡 Interfejs tylko po polsku; brak deklaracji języka dla czytników ekranu |
| 3.3.2 Etykiety | ✅ Etykiety tekstowe przy wszystkich kontrolkach |
| 4.1.2 Nazwa, rola, wartość | 🟡 GPUI Kit ma AccessKit (role, nazwy, stany). Nakładka lupy to obraz — czytnik ekranu jej nie przeczyta (rozważyć odczyt tłumaczenia na głos) |

## Pomysły na dalej

- Odczytywanie tłumaczenia pod lupą na głos (TTS systemu).
- Ustawienie rozmiaru lupy i czcionki w oknie głównym.
- Język interfejsu zgodny z językiem docelowym.
- Tryb wysokiego kontrastu: tłumaczenie zawsze czarne na żółtym.
