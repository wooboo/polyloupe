use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use anyhow::{Context as _, anyhow};
use fxtranslate::cache::{Cache, ModelFiles, ensure_model};
use fxtranslate::engine::Engine;
use fxtranslate::fetch::NetworkFetch;
use fxtranslate::remote::{Record, fetch_records};
use parking_lot::Mutex;

use crate::{Lang, Translator};

/// Offline translator backed by Firefox Translations models.
///
/// Models are downloaded on first use (or ahead of time with [`prefetch`]) and
/// kept in a local cache. `fxtranslate` engines are not `Send`, so they live on
/// a dedicated worker thread; this handle talks to it over a channel.
///
/// [`prefetch`]: OfflineTranslator::prefetch
pub struct OfflineTranslator {
    models_dir: PathBuf,
    jobs: Mutex<mpsc::Sender<Job>>,
}

struct Job {
    text: String,
    src: Lang,
    trg: Lang,
    reply: mpsc::Sender<anyhow::Result<String>>,
}

impl OfflineTranslator {
    pub fn new(models_dir: impl Into<PathBuf>) -> Self {
        let models_dir = models_dir.into();
        let (jobs, rx) = mpsc::channel::<Job>();
        let worker_dir = models_dir.clone();
        thread::Builder::new()
            .name("polyloupe-translate".into())
            .spawn(move || {
                let mut worker = Worker {
                    models: Models::new(&worker_dir),
                    engines: HashMap::new(),
                };
                for job in rx {
                    let result = worker.translate(&job.text, job.src, job.trg);
                    job.reply.send(result).ok();
                }
            })
            .expect("spawning translation worker");
        Self {
            models_dir,
            jobs: Mutex::new(jobs),
        }
    }

    /// Download every model needed to translate from any supported language
    /// into `trg`, so later translations work without a network. Blocking.
    pub fn prefetch(&self, trg: Lang, mut on_progress: impl FnMut(usize, usize)) -> anyhow::Result<()> {
        let models = Models::new(&self.models_dir);
        let pairs = model_pairs_into(trg);
        for (i, (src, trg)) in pairs.iter().enumerate() {
            on_progress(i, pairs.len());
            models.files(*src, *trg)?;
        }
        on_progress(pairs.len(), pairs.len());
        Ok(())
    }

    /// Whether every model needed for translating into `trg` is already on disk.
    pub fn is_ready_for(&self, trg: Lang) -> bool {
        let models = Models::new(&self.models_dir);
        model_pairs_into(trg)
            .into_iter()
            .all(|(s, t)| models.cache.cached_model(s.code(), t.code()).is_some())
    }
}

impl Translator for OfflineTranslator {
    fn translate(&self, text: &str, src: Lang, trg: Lang) -> anyhow::Result<String> {
        if src == trg {
            return Ok(text.to_owned());
        }
        let (reply, rx) = mpsc::channel();
        self.jobs
            .lock()
            .send(Job {
                text: text.to_owned(),
                src,
                trg,
                reply,
            })
            .map_err(|_| anyhow!("translation worker stopped"))?;
        rx.recv().map_err(|_| anyhow!("translation worker stopped"))?
    }
}

struct Worker {
    models: Models,
    engines: HashMap<(Lang, Lang), Engine>,
}

impl Worker {
    fn translate(&mut self, text: &str, src: Lang, trg: Lang) -> anyhow::Result<String> {
        Ok(match (src, trg) {
            _ if src == trg => text.to_owned(),
            (Lang::En, _) | (_, Lang::En) => self.engine(src, trg)?.translate_long(text),
            _ => {
                let english = self.engine(src, Lang::En)?.translate_long(text);
                self.engine(Lang::En, trg)?.translate_long(&english)
            }
        })
    }

    fn engine(&mut self, src: Lang, trg: Lang) -> anyhow::Result<&Engine> {
        if !self.engines.contains_key(&(src, trg)) {
            let files = self.models.files(src, trg)?;
            let engine = Engine::load_mmapped(&files.model, &files.src_vocab, &files.trg_vocab)
                .map_err(|e| anyhow!(e))?;
            self.engines.insert((src, trg), engine);
        }
        Ok(&self.engines[&(src, trg)])
    }
}

/// Locates model files on disk, downloading them on a cache miss.
struct Models {
    cache: Cache,
    records: Option<Vec<Record>>,
}

impl Models {
    fn new(dir: &Path) -> Self {
        Self {
            cache: Cache::with_root(dir).with_progress(false),
            records: None,
        }
    }

    fn files(&self, src: Lang, trg: Lang) -> anyhow::Result<ModelFiles> {
        if let Some(files) = self.cache.cached_model(src.code(), trg.code()) {
            return Ok(files);
        }
        let fetch = NetworkFetch::new();
        let records = match &self.records {
            Some(records) => records,
            None => &fetch_records(&fetch).map_err(|e| anyhow!(e))?,
        };
        ensure_model(&fetch, &self.cache, records, src.code(), trg.code())
            .map_err(|e| anyhow!(e))
            .with_context(|| format!("downloading model {}→{}", src.code(), trg.code()))
    }
}

/// The models (each one-directional, to or from English) needed to translate
/// from every supported language into `trg`.
fn model_pairs_into(trg: Lang) -> Vec<(Lang, Lang)> {
    let mut pairs: Vec<_> = Lang::ALL
        .into_iter()
        .filter(|&l| l != Lang::En && l != trg)
        .map(|l| (l, Lang::En))
        .collect();
    if trg != Lang::En {
        pairs.push((Lang::En, trg));
    }
    pairs
}
