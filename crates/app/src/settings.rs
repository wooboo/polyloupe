use std::path::PathBuf;

use polyloupe_translate::Lang;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Language the loupe translates into (a `Lang` code).
    pub target_language: String,
    /// Initial magnification of the loupe.
    pub zoom: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            target_language: Lang::Pl.code().into(),
            zoom: 2.0,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        std::fs::read_to_string(path())
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).ok();
        }
        if let Err(err) = std::fs::write(&path, serde_json::to_string_pretty(self).unwrap()) {
            log::error!("saving settings to {}: {err}", path.display());
        }
    }

    pub fn target(&self) -> Lang {
        Lang::from_code(&self.target_language).unwrap_or(Lang::Pl)
    }
}

fn path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_default()
        .join("polyloupe")
        .join("settings.json")
}

pub fn models_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_default()
        .join("polyloupe")
        .join("models")
}
