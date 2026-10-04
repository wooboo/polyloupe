//! The main window: a big button to take out the loupe, and the few settings.

use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::radio::RadioGroup;
use gpui_kit::component::{ActiveTheme as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use polyloupe_translate::Lang;

use crate::state::{self, AppState, ModelStatus};
use crate::{platform, tray};

pub struct HomeView {
    capture_allowed: bool,
    _subscriptions: Vec<Subscription>,
    _poll_permission: Task<()>,
}

impl HomeView {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe_global::<AppState>(|_, cx| cx.notify());
        // The permission is granted in System Settings; notice when it changes.
        let poll = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(2)).await;
                let allowed = platform::screen_capture_allowed();
                let changed = this.update(cx, |this, cx| {
                    if this.capture_allowed != allowed {
                        this.capture_allowed = allowed;
                        cx.notify();
                    }
                });
                if changed.is_err() {
                    break;
                }
            }
        });
        Self {
            capture_allowed: platform::screen_capture_allowed(),
            _subscriptions: vec![observe],
            _poll_permission: poll,
        }
    }

    fn render_permission(&self, cx: &App) -> Option<impl IntoElement> {
        (!self.capture_allowed).then(|| {
            notice(cx)
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Polyloupe potrzebuje pozwolenia, aby widzieć ekran."),
                )
                .child(
                    "W Ustawieniach systemowych włącz Polyloupe w sekcji „Nagrywanie ekranu”, \
                     a potem uruchom program ponownie.",
                )
                .child(
                    Button::new("open-settings")
                        .large()
                        .label("Otwórz ustawienia")
                        .on_click(|_, _, _| platform::open_screen_capture_settings()),
                )
        })
    }

    fn render_models(&self, cx: &App) -> impl IntoElement {
        let (text, retry) = match &AppState::global(cx).models {
            ModelStatus::Ready => (
                "Słowniki są gotowe. Tłumaczenie działa bez internetu.".to_string(),
                false,
            ),
            ModelStatus::Downloading { done, total } => (
                format!("Pobieram słowniki ({done} z {total})… To potrzebne tylko raz."),
                false,
            ),
            ModelStatus::Failed(_) => (
                "Nie udało się pobrać słowników. Sprawdź połączenie z internetem.".to_string(),
                true,
            ),
        };
        div()
            .flex()
            .flex_col()
            .gap_2()
            .text_color(cx.theme().muted_foreground)
            .child(text)
            .when(retry, |this| {
                this.child(
                    Button::new("retry-models")
                        .large()
                        .label("Spróbuj ponownie")
                        .on_click(|_, _, cx| state::prepare_models(cx)),
                )
            })
    }
}

impl Render for HomeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let target = AppState::global(cx).settings.target();
        let selected = Lang::ALL.iter().position(|&l| l == target);

        div()
            .id("home")
            .size_full()
            .overflow_y_scroll()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .p_8()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(32.))
                            .font_weight(FontWeight::BOLD)
                            .child("Polyloupe"),
                    )
                    .child("Lupa, która tłumaczy tekst na ekranie."),
            )
            .children(self.render_permission(cx))
            .child(
                Button::new("show-loupe")
                    .primary()
                    .large()
                    .h(px(72.))
                    .w_full()
                    .text_size(px(24.))
                    .label("Pokaż lupę")
                    .on_click(|_, _, cx| state::toggle_loupe(cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child("Lupę można też wyciągnąć w każdym programie skrótem:")
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(tray::HOTKEY_LABEL),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child("albo z ikony lupy na pasku menu."),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Tłumacz na język:"),
                    )
                    .child(
                        RadioGroup::vertical("target-language")
                            .children(Lang::ALL.map(Lang::native_name))
                            .selected_index(selected)
                            .on_change(|index, _, cx| {
                                if let Some(&lang) = Lang::ALL.get(*index) {
                                    state::set_target_language(lang, cx);
                                }
                            }),
                    ),
            )
            .child(self.render_models(cx))
    }
}

fn notice(cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .p_4()
        .rounded(cx.theme().radius)
        .border_2()
        .border_color(cx.theme().warning)
        .bg(cx.theme().warning.opacity(0.12))
}
