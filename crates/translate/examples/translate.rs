//! cargo run -p polyloupe-translate --release --example translate -- pl "Some text"
use std::time::Instant;

use polyloupe_translate::{Lang, LanguageDetector, OfflineTranslator, Translator};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let trg =
        Lang::from_code(&args.next().unwrap_or("pl".into())).expect("unknown target language");
    let text = args
        .next()
        .unwrap_or("The weather is nice today. Please remember to take your medicine.".into());

    let dir = std::env::temp_dir().join("polyloupe-models");
    let translator = OfflineTranslator::new(&dir);
    let src = LanguageDetector::new()
        .detect(&text)
        .expect("could not detect language");
    println!("{src:?} → {trg:?}");

    let t = Instant::now();
    let first = translator.translate(&text, src, trg)?;
    println!("[{:?} incl. download/load] {first}", t.elapsed());
    let t = Instant::now();
    let second = translator.translate(&text, src, trg)?;
    println!("[{:?} warm] {second}", t.elapsed());
    Ok(())
}
