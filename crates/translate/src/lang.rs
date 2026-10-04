use lingua::{Language, LanguageDetectorBuilder};

/// Languages polyloupe supports, in implementation-priority order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lang {
    Pl,
    En,
    Da,
    Sv,
    Fi,
    Nb,
}

impl Lang {
    pub const ALL: [Lang; 6] = [Lang::Pl, Lang::En, Lang::Da, Lang::Sv, Lang::Fi, Lang::Nb];

    /// Code used by the Firefox Translations model registry.
    pub fn code(self) -> &'static str {
        match self {
            Lang::Pl => "pl",
            Lang::En => "en",
            Lang::Da => "da",
            Lang::Sv => "sv",
            Lang::Fi => "fi",
            Lang::Nb => "nb",
        }
    }

    /// Name of the language written in that language, as shown in the UI.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Pl => "Polski",
            Lang::En => "English",
            Lang::Da => "Dansk",
            Lang::Sv => "Svenska",
            Lang::Fi => "Suomi",
            Lang::Nb => "Norsk",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code() == code)
    }

    fn from_lingua(language: Language) -> Option<Lang> {
        Some(match language {
            Language::Polish => Lang::Pl,
            Language::English => Lang::En,
            Language::Danish => Lang::Da,
            Language::Swedish => Lang::Sv,
            Language::Finnish => Lang::Fi,
            Language::Bokmal => Lang::Nb,
            #[allow(unreachable_patterns)]
            _ => return None,
        })
    }
}

/// Detects which supported language a piece of text is written in.
pub struct LanguageDetector(lingua::LanguageDetector);

impl LanguageDetector {
    pub fn new() -> Self {
        Self(
            LanguageDetectorBuilder::from_languages(&[
                Language::Polish,
                Language::English,
                Language::Danish,
                Language::Swedish,
                Language::Finnish,
                Language::Bokmal,
            ])
            .with_preloaded_language_models()
            .build(),
        )
    }

    pub fn detect(&self, text: &str) -> Option<Lang> {
        self.0.detect_language_of(text).and_then(Lang::from_lingua)
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}
