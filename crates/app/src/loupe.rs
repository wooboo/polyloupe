//! The loupe: a full-screen overlay showing a frozen screenshot, with a
//! magnifying glass that follows the mouse and shows translated text in place.

use std::sync::Arc;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use polyloupe_ocr::Rect;
use polyloupe_translate::{Lang, Translator as _};

use crate::analysis::{self, Region};
use crate::capture::Screenshot;
use crate::state::AppState;

const MIN_ZOOM: f32 = 1.0;
const MAX_ZOOM: f32 = 6.0;

pub struct LoupeView {
    focus: FocusHandle,
    image: Arc<RenderImage>,
    /// Logical size of the screenshot (= the overlay window).
    screen: Size<Pixels>,
    /// Physical pixels per logical pixel in the screenshot.
    scale: f32,
    cursor: Point<Pixels>,
    zoom: f32,
    target: Lang,
    show_original: bool,
    regions: Vec<Region>,
    status: Status,
    _analysis: Task<()>,
}

enum Status {
    Recognizing,
    Translating,
    Done,
    NothingToTranslate,
    Failed(SharedString),
}

impl LoupeView {
    pub fn new(
        shot: Screenshot,
        cursor: Point<Pixels>,
        prepared: Option<Vec<Region>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);

        let state = AppState::global(cx);
        let target = state.settings.target();
        let zoom = state.settings.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        let translator = state.translator.clone();
        let detector = state.detector.clone();

        let Screenshot { image, scale } = shot;
        let screen = size(
            px(image.width() as f32 / scale),
            px(image.height() as f32 / scale),
        );
        let render_image = Arc::new(to_render_image(&image));

        if let Some(regions) = prepared {
            return Self {
                focus,
                image: render_image,
                screen,
                scale,
                cursor,
                zoom,
                target,
                show_original: false,
                regions,
                status: Status::Done,
                _analysis: Task::ready(()),
            };
        }

        let analysis = cx.spawn(async move |this, cx| {
            let found = cx
                .background_spawn(async move {
                    let started = std::time::Instant::now();
                    let found = analysis::find_regions(&image, target, &detector);
                    log::info!("text recognition took {:?}", started.elapsed());
                    found
                })
                .await;
            let mut regions = match found {
                Ok(regions) => regions,
                Err(err) => {
                    log::error!("text recognition failed: {err:#}");
                    this.update(cx, |this, cx| {
                        this.status =
                            Status::Failed(format!("Nie udało się odczytać tekstu: {err}").into());
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };
            let Ok(cursor) = this.update(cx, |this, cx| {
                this.status = if regions.is_empty() {
                    Status::NothingToTranslate
                } else {
                    Status::Translating
                };
                this.regions = regions.clone();
                cx.notify();
                this.cursor_in_screenshot()
            }) else {
                return;
            };

            // Translate what is under the loupe first.
            let mut order: Vec<usize> = (0..regions.len()).collect();
            order.sort_by(|&a, &b| {
                distance(&regions[a].bounds, cursor)
                    .total_cmp(&distance(&regions[b].bounds, cursor))
            });
            for index in order {
                let region = &mut regions[index];
                let (text, src) = (region.original.clone(), region.language);
                let translator = translator.clone();
                let result = cx
                    .background_spawn(async move { translator.translate(&text, src, target) })
                    .await;
                match result {
                    Ok(translation) => {
                        log::debug!("{src:?}→{target:?}: {translation}");
                        let updated = this.update(cx, |this, cx| {
                            if let Some(region) = this.regions.get_mut(index) {
                                region.translation = Some(translation);
                            }
                            cx.notify();
                        });
                        if updated.is_err() {
                            return; // the loupe was closed
                        }
                    }
                    Err(err) => log::warn!("translating {src:?}→{target:?} failed: {err:#}"),
                }
            }
            this.update(cx, |this, cx| {
                this.status = Status::Done;
                cx.notify();
            })
            .ok();
        });

        Self {
            focus,
            image: render_image,
            screen,
            scale,
            cursor,
            zoom,
            target,
            show_original: false,
            regions: Vec::new(),
            status: Status::Recognizing,
            _analysis: analysis,
        }
    }

    fn cursor_in_screenshot(&self) -> (f32, f32) {
        (
            f32::from(self.cursor.x) * self.scale,
            f32::from(self.cursor.y) * self.scale,
        )
    }

    fn set_zoom(&mut self, zoom: f32, cx: &mut Context<Self>) {
        self.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        cx.notify();
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let zoom = self.zoom;
        AppState::update(cx, |state, _| {
            state.settings.zoom = zoom;
            state.settings.save();
        });
        window.remove_window();
        crate::state::after_loupe_closed(cx);
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "escape" | "enter" => self.close(window, cx),
            "space" => {
                self.show_original = !self.show_original;
                cx.notify();
            }
            "+" | "=" => self.set_zoom(self.zoom + 0.5, cx),
            "-" => self.set_zoom(self.zoom - 0.5, cx),
            // The loupe can be moved without a mouse, too.
            key @ ("left" | "right" | "up" | "down") => {
                let step = px(if event.keystroke.modifiers.shift {
                    8.
                } else {
                    40.
                });
                let (dx, dy) = match key {
                    "left" => (-step, px(0.)),
                    "right" => (step, px(0.)),
                    "up" => (px(0.), -step),
                    _ => (px(0.), step),
                };
                self.cursor = point(
                    (self.cursor.x + dx).clamp(px(0.), self.screen.width),
                    (self.cursor.y + dy).clamp(px(0.), self.screen.height),
                );
                cx.notify();
            }
            _ => {}
        }
    }

    fn lens_size(&self) -> Size<Pixels> {
        let width = (f32::from(self.screen.width) * 0.5).clamp(420.0, 900.0);
        size(px(width), px(width * 0.36))
    }

    /// The region under the centre of the loupe, for the caption.
    fn hovered_region(&self) -> Option<&Region> {
        let (x, y) = self.cursor_in_screenshot();
        self.regions.iter().find(|r| {
            r.bounds.intersects(&Rect {
                x,
                y,
                width: 1.0,
                height: 1.0,
            })
        })
    }

    fn render_lens(&self) -> impl IntoElement {
        let lens = self.lens_size();
        let zoom = self.zoom;
        let origin = point(
            self.cursor.x - lens.width / 2.0,
            self.cursor.y - lens.height / 2.0,
        );
        // Offset of the magnified screenshot inside the lens, so that the point
        // under the cursor stays in the middle of the lens.
        let offset = point(
            lens.width / 2.0 - self.cursor.x * zoom,
            lens.height / 2.0 - self.cursor.y * zoom,
        );
        let visible = Rect {
            x: f32::from(-offset.x) / zoom * self.scale,
            y: f32::from(-offset.y) / zoom * self.scale,
            width: f32::from(lens.width) / zoom * self.scale,
            height: f32::from(lens.height) / zoom * self.scale,
        };
        let to_lens = zoom / self.scale;

        let translations = (!self.show_original).then(|| {
            self.regions
                .iter()
                .filter(|r| r.translation.is_some() && r.bounds.intersects(&visible))
                .map(|r| render_translation(r, to_lens))
                .collect::<Vec<_>>()
        });

        let radius = px(14.);
        div()
            .absolute()
            .left(origin.x)
            .top(origin.y)
            .w(lens.width)
            .h(lens.height)
            .shadow_lg()
            .child(
                div()
                    .size_full()
                    .overflow_hidden()
                    .rounded(radius)
                    .bg(gpui_kit::white())
                    .child(
                        div()
                            .absolute()
                            .left(offset.x)
                            .top(offset.y)
                            .w(self.screen.width * zoom)
                            .h(self.screen.height * zoom)
                            .child(img(self.image.clone()).size_full())
                            .children(translations.into_iter().flatten()),
                    ),
            )
            // Two rings, dark inside light, keep the edge visible on any background.
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded(radius)
                    .border_3()
                    .border_color(gpui_kit::white())
                    .child(
                        div()
                            .size_full()
                            .rounded(radius - px(3.))
                            .border_3()
                            .border_color(rgb(0x1a1a1a)),
                    ),
            )
    }

    fn render_caption(&self) -> impl IntoElement {
        let lens = self.lens_size();
        let mut text = match &self.status {
            Status::Recognizing => "Odczytuję tekst…".to_string(),
            Status::Translating => "Tłumaczę…".to_string(),
            Status::NothingToTranslate => "Nie znalazłem tekstu do przetłumaczenia".to_string(),
            Status::Failed(message) => message.to_string(),
            Status::Done => String::new(),
        };
        if let Some(region) = self.hovered_region() {
            let languages = format!(
                "{} → {}",
                region.language.native_name(),
                self.target.native_name()
            );
            text = if text.is_empty() {
                languages
            } else {
                format!("{languages} · {text}")
            };
        }
        if self.show_original {
            text = "Oryginał (Spacja: tłumaczenie)".into();
        }
        let zoom = format!("{:.1}×", self.zoom).replace(".0×", "×");
        let text = if text.is_empty() {
            zoom
        } else {
            format!("{text} · {zoom}")
        };

        let below = self.cursor.y + lens.height / 2.0 + px(10.);
        let top = if below + px(48.) > self.screen.height {
            self.cursor.y - lens.height / 2.0 - px(56.)
        } else {
            below
        };
        div()
            .absolute()
            .left(self.cursor.x - lens.width / 2.0)
            .top(top)
            .w(lens.width)
            .flex()
            .justify_center()
            .child(pill(text))
    }

    fn render_help(&self) -> impl IntoElement {
        // Stay out of the way: sit at the bottom unless the loupe is down there.
        let at_top = self.cursor.y > self.screen.height * 0.7;
        div()
            .absolute()
            .left_0()
            .right_0()
            .when(at_top, |this| this.top(px(24.)))
            .when(!at_top, |this| this.bottom(px(24.)))
            .flex()
            .justify_center()
            .child(pill(
                "Przesuń lupę nad tekst (myszką lub strzałkami) · Kółko myszy lub +/−: powiększenie · Spacja: oryginał · Kliknięcie lub Esc: zamknij",
            ))
    }
}

impl Render for LoupeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("loupe")
            .track_focus(&self.focus)
            .size_full()
            .relative()
            .overflow_hidden()
            .cursor(CursorStyle::Crosshair)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                this.cursor = event.position;
                cx.notify();
            }))
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                let dy = f32::from(event.delta.pixel_delta(px(20.)).y);
                this.set_zoom(this.zoom * (1.0 + dy / 200.0), cx);
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| this.close(window, cx)),
            )
            .child(
                img(self.image.clone())
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full(),
            )
            .child(self.render_lens())
            .child(self.render_caption())
            .child(self.render_help())
    }
}

/// Paint `region`'s translation over the original text, `to_lens` mapping
/// screenshot pixels to lens pixels.
fn render_translation(region: &Region, to_lens: f32) -> impl IntoElement {
    let translation = region.translation.clone().unwrap_or_default();
    let b = &region.bounds;
    // Pick the font size at which the translation fills as many lines as the
    // original did (an average character is about half an em wide), but never
    // larger than the original text and not so small that it becomes hard to read.
    let natural = region.line_height * 0.78;
    let width = b.width - region.line_height * 0.3;
    let chars = translation.chars().count().max(1) as f32;
    let fitted = width * region.line_count as f32 / (chars * 0.52);
    let font = fitted.min(natural).max(natural * 0.55) * to_lens;
    let [r, g, bl] = region.background;
    let [fr, fg, fb] = region.foreground;
    div()
        .absolute()
        .left(px(b.x * to_lens))
        .top(px(b.y * to_lens))
        .w(px(b.width * to_lens))
        .min_h(px(b.height * to_lens))
        .px(px(region.line_height * to_lens * 0.15))
        .bg(rgb(u32::from_be_bytes([0, r, g, bl])))
        .text_color(rgb(u32::from_be_bytes([0, fr, fg, fb])))
        .text_size(px(font))
        .line_height(px(font * 1.25))
        .child(translation)
}

fn pill(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .px_4()
        .py_2()
        .rounded_full()
        .bg(rgb(0x111111))
        .text_color(gpui_kit::white())
        .text_size(px(18.))
        .border_2()
        .border_color(gpui_kit::white())
        .child(text.into())
}

fn distance(bounds: &Rect, (x, y): (f32, f32)) -> f32 {
    let dx = (bounds.x - x).max(0.0).max(x - bounds.right());
    let dy = (bounds.y - y).max(0.0).max(y - bounds.bottom());
    dx * dx + dy * dy
}

fn to_render_image(image: &image::RgbaImage) -> RenderImage {
    let mut bgra = image.clone();
    for pixel in bgra.pixels_mut() {
        pixel.0.swap(0, 2);
    }
    RenderImage::new(vec![image::Frame::new(bgra)])
}
