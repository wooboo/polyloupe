# Tłumaczenie: offline i chmura

Języki: polski, angielski, duński, szwedzki, fiński, norweski (bokmål), w tej
kolejności. Dowolne pary.

## Wybór: modele Firefox Translations przez `fxtranslate`

- **Modele:** Mozilla [Firefox Translations](https://github.com/mozilla/translations),
  te same, których używa Firefox. Dla naszych języków istnieją wszystkie potrzebne kierunki
  (sprawdzone w rejestrze Remote Settings):
  `pl↔en`, `da↔en`, `sv↔en`, `fi↔en`, `nb↔en` (także `nn→en`).
  Rozmiar: 17–31 MB na kierunek.
- **Pary bez angielskiego** idą przez angielski (`da → en → pl`).
- **Silnik:** [`fxtranslate`](https://crates.io/crates/fxtranslate) — przepisany na
  Rusta silnik Bergamot/Marian (autor: Greg Tatum z zespołu tłumaczeń Mozilli),
  sprawdzany względem oryginału w C++. Nie trzeba budować Mariana ani CMake.
  Jądro mnożenia macierzy (gemmology) jest w C++ i budowane przez `cc`;
  jest też wariant `portable` w czystym Ruście.
- **Pomiary na tym Macu (M-series, release):**

  | Para | Pierwsze użycie (pobranie + ładowanie) | Kolejne |
  |---|---|---|
  | en → pl | 1,4 s | 8 ms |
  | da → pl (przez en) | 1,0 s | 12 ms |
  | sv → pl | 0,8 s | 11 ms |
  | fi → pl | 1,2 s | 15 ms |
  | nb → pl | 1,7 s | 11 ms |
  | pl → da | 2,0 s | 13 ms |

  Jakość: „Husk at tage din medicin før morgenmaden. Lægen ringer i morgen.” →
  „Pamiętaj, aby wziąć leki przed śniadaniem. Lekarz zadzwoni jutro.”
- **Pamięć:** ok. 150 MB na załadowany model (wg autora); ładujemy leniwie,
  przez `mmap`.
- **Licencje:** `fxtranslate` — MPL-2.0; modele — sprawdzić przed publikacją
  (repozytorium mozilla/translations jest na MPL-2.0).

⚠️ **Hosting modeli.** Pobieramy je z CDN Firefoksa. Autor `fxtranslate` wprost
zastrzega, że ta infrastruktura jest dla Firefoksa, a nie dla innych aplikacji.
Przed wydaniem trzeba
**skopiować potrzebne pliki na własny serwer** (np. GitHub Releases) i ładować je
stamtąd (`Engine::load` / własny `Fetch`).

### Odrzucone alternatywy offline

| Opcja | Dlaczego nie (na razie) |
|---|---|
| Apple Translation (macOS 15+) | Tylko API w Swifcie (most Swift↔Rust); brak fińskiego; tylko macOS |
| NLLB-200 + CTranslate2 (`ct2rs`) | Bezpośrednie tłumaczenie między dowolnymi parami, ale licencja **CC-BY-NC** (zakaz zastosowań komercyjnych) i ok. 600 MB |
| OPUS-MT (Helsinki-NLP) + CTranslate2 | Dobre modele nordyckie, ale wymaga konwersji i biblioteki C++ CTranslate2 |
| LibreTranslate / Argos | Serwer w Pythonie; obsługuje wszystkie nasze języki, ale to osobna usługa do uruchomienia |
| MADLAD-400 | 3B+ parametrów — za ciężkie na starszy komputer |

## Dostawcy chmurowi (bez płacenia)

Do rozważenia jako **opcjonalny** dodatek, np. „lepsze tłumaczenie, gdy jest
internet” albo zapasowe rozwiązanie. Domyślnie wszystko zostaje offline, bo
lupa będzie najeżdżać na bankowość, maile i dokumenty urzędowe.

| Dostawca | Darmowy limit | Karta / konto | Nasze języki | Uwagi |
|---|---|---|---|---|
| **Azure AI Translator F0** | 2 mln znaków / mies. | konto Azure | wszystkie | Najhojniejszy stały limit. Najlepszy kandydat, jeśli chmura w ogóle |
| **Google Cloud Translation** | 500 tys. znaków / mies. | **wymaga konta rozliczeniowego** (karta) | wszystkie | Przy przekroczeniu nalicza opłaty — ryzyko kosztów |
| **DeepL API Free** | 500 tys. znaków / mies. | — | wszystkie | **Nowi użytkownicy nie mogą się już zapisać**; nowy plan „Developer” daje jednorazowo 1 mln znaków |
| **MyMemory** | 5 tys. znaków / dzień (50 tys. z e-mailem) | brak | wszystkie | Maks. 500 bajtów na zapytanie; jakość nierówna (pamięć tłumaczeń + MT) |
| **Gemini API (free tier)** | ok. 1000–1500 zapytań / dzień | konto Google | wszystkie | **Dane z darmowego planu służą do ulepszania produktów Google** — nie do treści z ekranu starszej osoby |
| **LibreTranslate (własny serwer)** | bez limitu | brak | wszystkie | Darmowy, ale wymaga postawienia serwera; publiczna instancja jest płatna |

**Rekomendacja:** zostać przy offline jako jedynym domyślnym trybie. Jeśli kiedyś
chmura, to Azure F0 z kluczem podanym przez użytkownika i wyraźną zgodą
„wysyłaj tekst z ekranu do Microsoftu”. Interfejs `Translator` w
`crates/translate` jest już przygotowany na kolejne implementacje.

Źródła: [DeepL API plans](https://support.deepl.com/hc/en-us/articles/360021200939-DeepL-API-plans),
[Azure Translator pricing](https://azure.microsoft.com/en-us/pricing/details/cognitive-services/translator/),
[Google Cloud Translation](https://cloud.google.com/translate),
[MyMemory limits](https://mymemory.translated.net/doc/usagelimits.php),
[Gemini free tier](https://www.memetik.ai/guides/gemini-api-free-tier-limits),
[LibreTranslate](https://github.com/LibreTranslate/LibreTranslate).
