//! Translation for polyloupe.
//!
//! The default backend is fully offline: Firefox Translations models run by
//! [`fxtranslate`]. Every model goes to or from English, so other pairs pivot
//! through English (e.g. `da → en → pl`).

mod lang;
mod offline;

pub use lang::{Lang, LanguageDetector};
pub use offline::OfflineTranslator;

/// Something that turns text in one language into another.
pub trait Translator: Send + Sync {
    fn translate(&self, text: &str, src: Lang, trg: Lang) -> anyhow::Result<String>;
}
