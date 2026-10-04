//! Application-wide state and the commands that open and close windows.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::*;
use polyloupe_translate::{Lang, LanguageDetector, OfflineTranslator, Translator as _};

use crate::home::HomeView;
use crate::loupe::LoupeView;
use crate::settings::{self, Settings};
use crate::{capture, platform};

pub struct AppState {
    pub settings: Settings,
    pub translator: Arc<OfflineTranslator>,
    pub detector: Arc<LanguageDetector>,
    pub models: ModelStatus,
    home: Option<AnyWindowHandle>,
    loupe: Option<AnyWindowHandle>,
    after_loupe: AfterLoupe,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModelStatus {
    Ready,
    Downloading { done: usize, total: usize },
    Failed(String),
}

/// Where focus goes back to once the loupe is put away.
#[derive(Clone, Copy, Default, PartialEq)]
enum AfterLoupe {
    /// Back to the home window, which the loupe was opened from.
    Home,
    /// Back to whichever app was in front (the loupe came from the hotkey or tray).
    #[default]
    PreviousApp,
    /// Our app was already in front; nothing to do.
    Stay,
}

impl Global for AppState {}

impl AppState {
    pub fn init(cx: &mut App) {
        let settings = Settings::load();
        cx.set_global(AppState {
            settings,
            translator: Arc::new(OfflineTranslator::new(settings::models_dir())),
            detector: Arc::new(LanguageDetector::new()),
            models: ModelStatus::Ready,
            home: None,
            loupe: None,
            after_loupe: AfterLoupe::default(),
        });
        prepare_models(cx);
    }

    pub fn global(cx: &App) -> &AppState {
        cx.global::<AppState>()
    }

    pub fn update<R>(cx: &mut App, f: impl FnOnce(&mut AppState, &mut App) -> R) -> R {
        cx.update_global(f)
    }
}

/// Make sure the offline dictionaries for the target language are on disk.
pub fn prepare_models(cx: &mut App) {
    let state = AppState::global(cx);
    let target = state.settings.target();
    let translator = state.translator.clone();
    if translator.is_ready_for(target) {
        AppState::update(cx, |state, _| state.models = ModelStatus::Ready);
        return;
    }
    AppState::update(cx, |state, _| {
        state.models = ModelStatus::Downloading { done: 0, total: 1 }
    });

    let (progress_tx, progress_rx) = async_channel::unbounded();
    let download = cx.background_spawn(async move {
        translator.prefetch(target, |done, total| {
            progress_tx.try_send((done, total)).ok();
        })
    });
    cx.spawn(async move |cx| {
        while let Ok((done, total)) = progress_rx.recv().await {
            cx.update(|cx| {
                AppState::update(cx, |state, _| {
                    state.models = ModelStatus::Downloading { done, total }
                })
            });
        }
        let status = match download.await {
            Ok(()) => ModelStatus::Ready,
            Err(err) => {
                log::error!("downloading models: {err:#}");
                ModelStatus::Failed(err.to_string())
            }
        };
        cx.update(|cx| AppState::update(cx, |state, _| state.models = status));
    })
    .detach();
}

pub fn set_target_language(lang: Lang, cx: &mut App) {
    AppState::update(cx, |state, _| {
        state.settings.target_language = lang.code().into();
        state.settings.save();
    });
    prepare_models(cx);
}

pub fn show_home(cx: &mut App) {
    if let Some(home) = AppState::global(cx).home
        && home
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    {
        cx.activate(true);
        return;
    }
    let bounds = Bounds::centered(None, size(px(560.), px(720.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("Polyloupe".into()),
            ..Default::default()
        }),
        window_min_size: Some(size(px(420.), px(480.))),
        ..Default::default()
    };
    match gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| HomeView::new(window, cx))
    }) {
        Ok((handle, _)) => {
            AppState::update(cx, |state, _| state.home = Some(handle));
            cx.activate(true);
        }
        Err(err) => log::error!("opening the main window: {err:#}"),
    }
}

/// Open the loupe, or put it away if it is already out.
pub fn toggle_loupe(cx: &mut App) {
    if let Some(loupe) = AppState::update(cx, |state, _| state.loupe.take())
        && loupe
            .update(cx, |_, window, _| window.remove_window())
            .is_ok()
    {
        after_loupe_closed(cx);
        return;
    }

    if !platform::screen_capture_allowed() {
        platform::request_screen_capture();
        show_home(cx);
        return;
    }

    // The home window would cover what the user wants to read: hide it first
    // and give the window server a moment before taking the screenshot.
    let home = AppState::global(cx).home.filter(|home| {
        home.update(cx, |_, window, _| window.is_window_active())
            .unwrap_or(false)
    });
    let after = match (home, cx.active_window()) {
        (Some(_), _) => AfterLoupe::Home,
        (None, Some(_)) => AfterLoupe::Stay,
        (None, None) => AfterLoupe::PreviousApp,
    };
    AppState::update(cx, |state, _| state.after_loupe = after);
    if let Some(home) = home {
        home.update(cx, |_, window, _| window.remove_window()).ok();
        cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_millis(350))
                .await;
            cx.update(open_loupe);
        })
        .detach();
    } else {
        open_loupe(cx);
    }
}

fn open_loupe(cx: &mut App) {
    let displays = cx.displays();
    let primary = cx.primary_display();
    let (x, y) = platform::cursor_position().unwrap_or_else(|| {
        let center = primary
            .as_ref()
            .map(|d| d.bounds().center())
            .unwrap_or_default();
        (center.x.into(), center.y.into())
    });
    let at = point(px(x), px(y));
    let Some(display) = displays
        .into_iter()
        .find(|d| d.bounds().contains(&at))
        .or(primary)
    else {
        log::error!("no display to show the loupe on");
        return;
    };
    let bounds = display.bounds();
    let shot = match capture::capture_display_at(x, y) {
        Ok(shot) => shot,
        Err(err) => {
            log::error!("{err:#}");
            return;
        }
    };
    let cursor = point(at.x - bounds.origin.x, at.y - bounds.origin.y);

    open_loupe_window(shot, cursor, None, bounds, Some(display.id()), cx);
}

/// Development aid: open the loupe over an image file instead of the screen,
/// in an ordinary window. `scale` is the image's physical-per-logical pixel ratio.
///
/// The text is recognized and translated before the window opens, so even the
/// first frame (e.g. a window screenshot) shows the translations.
pub fn open_loupe_on_image(
    path: &std::path::Path,
    scale: f32,
    at: Option<(f32, f32)>,
    zoom: Option<f32>,
    cx: &mut App,
) {
    let image = match image::open(path) {
        Ok(image) => image.to_rgba8(),
        Err(err) => {
            log::error!("opening {}: {err}", path.display());
            return;
        }
    };
    let logical = size(
        px(image.width() as f32 / scale),
        px(image.height() as f32 / scale),
    );
    let bounds = Bounds::centered(None, logical, cx);
    let cursor = at.map_or(
        point(logical.width / 2.0, logical.height / 2.0),
        |(x, y)| point(px(x), px(y)),
    );
    let state = AppState::update(cx, |state, _| {
        state.after_loupe = AfterLoupe::Stay;
        if let Some(zoom) = zoom {
            state.settings.zoom = zoom;
        }
        (
            state.settings.target(),
            state.detector.clone(),
            state.translator.clone(),
        )
    });
    let (target, detector, translator) = state;
    let regions = match crate::analysis::find_regions(&image, target, &detector) {
        Ok(mut regions) => {
            for region in &mut regions {
                region.translation = translator
                    .translate(&region.original, region.language, target)
                    .ok();
            }
            regions
        }
        Err(err) => {
            log::error!("{err:#}");
            Vec::new()
        }
    };
    open_loupe_window(
        capture::Screenshot { image, scale },
        cursor,
        Some(regions),
        bounds,
        None,
        cx,
    );
}

fn open_loupe_window(
    shot: capture::Screenshot,
    cursor: Point<Pixels>,
    prepared: Option<Vec<crate::analysis::Region>>,
    bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    cx: &mut App,
) {
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        kind: if display_id.is_some() {
            WindowKind::PopUp
        } else {
            WindowKind::Normal
        },
        focus: true,
        show: true,
        is_movable: false,
        is_resizable: false,
        is_minimizable: false,
        display_id,
        ..Default::default()
    };
    match cx.open_window(options, |window, cx| {
        cx.new(|cx| LoupeView::new(shot, cursor, prepared, window, cx))
    }) {
        Ok(handle) => {
            AppState::update(cx, |state, _| state.loupe = Some(handle.into()));
            cx.activate(true);
        }
        Err(err) => log::error!("opening the loupe: {err:#}"),
    }
}

/// Called by the loupe after it removed its window.
pub fn after_loupe_closed(cx: &mut App) {
    let after = AppState::update(cx, |state, _| {
        state.loupe = None;
        std::mem::take(&mut state.after_loupe)
    });
    match after {
        AfterLoupe::Home => show_home(cx),
        AfterLoupe::PreviousApp => cx.hide(),
        AfterLoupe::Stay => {}
    }
}
