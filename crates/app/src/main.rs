mod analysis;
mod capture;
mod home;
mod loupe;
mod platform;
mod settings;
mod state;
mod tray;

use gpui_kit::component::Theme;
use gpui_kit::*;

actions!(polyloupe, [ToggleLoupe, ShowHome, Quit]);

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("polyloupe=info"))
        .init();

    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);
    // Clicking the Dock / taskbar icon brings the main window back.
    app.on_reopen(state::show_home);
    app.run(|cx| {
        gpui_kit::init(cx);
        // Larger default text for readability.
        Theme::global_mut(cx).font_size = px(18.);
        cx.set_quit_mode(QuitMode::Explicit);

        cx.on_action(|_: &ToggleLoupe, cx| state::toggle_loupe(cx));
        cx.on_action(|_: &ShowHome, cx| state::show_home(cx));
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.set_menus(vec![Menu {
            name: "Polyloupe".into(),
            disabled: false,
            items: vec![
                MenuItem::action("Pokaż lupę", ToggleLoupe),
                MenuItem::action("Ustawienia…", ShowHome),
                MenuItem::separator(),
                MenuItem::action("Zakończ Polyloupe", Quit),
            ],
        }]);
        cx.bind_keys([KeyBinding::new("secondary-q", Quit, None)]);

        state::AppState::init(cx);
        tray::init(cx);
        match image_arg() {
            Some(path) => state::open_loupe_on_image(
                &path,
                2.0,
                arg("--at").and_then(|at| {
                    let (x, y) = at.split_once(',')?;
                    Some((x.parse().ok()?, y.parse().ok()?))
                }),
                arg("--zoom").and_then(|zoom| zoom.parse().ok()),
                cx,
            ),
            None => state::show_home(cx),
        }
    });
}

/// `--image <path> [--at x,y] [--zoom z]`: open the loupe over an image
/// instead of the screen (a development aid).
fn image_arg() -> Option<std::path::PathBuf> {
    arg("--image").map(Into::into)
}

fn arg(name: &str) -> Option<String> {
    std::env::args().skip_while(|arg| arg != name).nth(1)
}
