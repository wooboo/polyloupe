# Translation: offline and cloud

Languages: Polish, English, Danish, Swedish, Finnish, Norwegian (Bokmål), in
that order of priority. Any pair.

## Choice: Firefox Translations models via `fxtranslate`

- **Models:** Mozilla's [Firefox Translations](https://github.com/mozilla/translations),
  the same ones Firefox uses. All directions we need exist (checked in the
  Remote Settings registry): `pl↔en`, `da↔en`, `sv↔en`, `fi↔en`, `nb↔en`
  (and `nn→en`). Size: 17–31 MB per direction.
- **Pairs without English** pivot through English (`da → en → pl`).
- **Engine:** [`fxtranslate`](https://crates.io/crates/fxtranslate) — a Rust
  rewrite of the Bergamot/Marian engine by Greg Tatum of Mozilla's translations
  team, validated against the C++ original. No Marian or CMake build. The
  matrix-multiplication kernel (gemmology) is C++ built with `cc`; a pure-Rust
  `portable` variant exists too.
- **Measured on an Apple Silicon Mac (release build):**

  | Pair | First use (download + load) | Afterwards |
  |---|---|---|
  | en → pl | 1.4 s | 8 ms |
  | da → pl (via en) | 1.0 s | 12 ms |
  | sv → pl | 0.8 s | 11 ms |
  | fi → pl | 1.2 s | 15 ms |
  | nb → pl | 1.7 s | 11 ms |
  | pl → da | 2.0 s | 13 ms |

  Quality: “Husk at tage din medicin før morgenmaden. Lægen ringer i morgen.” →
  “Pamiętaj, aby wziąć leki przed śniadaniem. Lekarz zadzwoni jutro.”
- **Memory:** about 150 MB per loaded model (per the author); models load
  lazily, memory-mapped.
- **Licenses:** `fxtranslate` and the models are MPL-2.0.

⚠️ **Model hosting.** The models are downloaded from Firefox's CDN. The
`fxtranslate` author states explicitly that this hosting is provisioned for
Firefox, not for third-party apps. Before a release, **mirror the needed files
on our own hosting** (e.g. GitHub Releases) and load them from there
(`Engine::load` / a custom `Fetch`).

### Offline alternatives considered

| Option | Why not (for now) |
|---|---|
| Apple Translation (macOS 15+) | Swift-only API (needs a Swift↔Rust bridge); no Finnish; macOS only |
| NLLB-200 + CTranslate2 (`ct2rs`) | Direct any-to-any translation, but **CC-BY-NC** (no commercial use) and ~600 MB |
| OPUS-MT (Helsinki-NLP) + CTranslate2 | Good Nordic models, but needs conversion and the CTranslate2 C++ library |
| LibreTranslate / Argos | A Python server; supports all our languages, but is a separate service to run |
| MADLAD-400 | 3B+ parameters — too heavy for an older computer |

## Cloud providers (without paying)

Worth considering only as an **optional** extra, e.g. “better translation when
online”, or as a fallback. Everything stays offline by default, because the
loupe will hover over online banking, email and official letters.

| Provider | Free allowance | Card / account | Our languages | Notes |
|---|---|---|---|---|
| **Azure AI Translator F0** | 2M characters / month | Azure account | all | Most generous permanent allowance. Best candidate if we ever add a cloud option |
| **Google Cloud Translation** | 500k characters / month | **billing account required** (card) | all | Charges above the allowance — risk of costs |
| **DeepL API Free** | 500k characters / month | — | all | **No longer open to new sign-ups**; the new “Developer” plan gives a one-off 1M characters |
| **MyMemory** | 5k characters / day (50k with an email) | none | all | Max 500 bytes per request; uneven quality (translation memory + MT) |
| **Gemini API (free tier)** | ~1,000–1,500 requests / day | Google account | all | **Free-tier content is used to improve Google products** — not for an older person's screen contents |
| **LibreTranslate (self-hosted)** | unlimited | none | all | Free, but needs a server; the public instance is paid |

**Recommendation:** keep offline as the only default. If a cloud option is ever
added, use Azure F0 with a user-supplied key and explicit consent (“send text
from my screen to Microsoft”). The `Translator` trait in `crates/translate` is
ready for more implementations.

Sources: [DeepL API plans](https://support.deepl.com/hc/en-us/articles/360021200939-DeepL-API-plans),
[Azure Translator pricing](https://azure.microsoft.com/en-us/pricing/details/cognitive-services/translator/),
[Google Cloud Translation](https://cloud.google.com/translate),
[MyMemory limits](https://mymemory.translated.net/doc/usagelimits.php),
[Gemini free tier](https://www.memetik.ai/guides/gemini-api-free-tier-limits),
[LibreTranslate](https://github.com/LibreTranslate/LibreTranslate).
