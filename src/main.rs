use std::{io::Cursor, rc::Rc};

use freya::{prelude::*, radio::use_init_radio_station, webview::WebViewPlugin};
use tokio::runtime::Builder;

pub mod components;
pub mod config;
pub mod error;
pub mod state;
pub mod stoat;
pub mod theme;
pub mod utils;

// pub use components::*;
pub use config::*;
pub use error::*;
pub use state::*;
pub use stoat::*;
pub use theme::*;
pub use utils::*;

use crate::components::{HttpManager, MaterialThemeProvider, ModalManager, Root};

pub type ThemeSet = Rc<syntect::highlighting::ThemeSet>;
pub type SyntaxSet = Rc<syntect::parsing::SyntaxSet>;

fn app() -> impl IntoElement {
    use_init_radio_station::<AppState, AppChannel>(AppState::new);
    use_clipboard();

    let config = use_hook(|| {
        let state = State::create(read_config());
        provide_context(state);

        state
    });

    use_side_effect(move || {
        let new_value = config.read();

        write_config(&new_value);
    });

    use_hook(|| {
        use syntect::highlighting::ThemeSet;

        let mut themes = ThemeSet::load_defaults();

        themes.themes.insert(
            "CatppuccinMacchiato".to_string(),
            ThemeSet::load_from_reader(&mut Cursor::new(include_bytes!(
                "./assets/themes/CatppuccinMacchiato.tmTheme"
            )))
            .unwrap(),
        );

        themes.themes.insert(
            "OneHalfDark".to_string(),
            ThemeSet::load_from_reader(&mut Cursor::new(include_bytes!(
                "./assets/themes/OneHalfDark.tmTheme"
            )))
            .unwrap(),
        );

        themes.themes.insert(
            "OneHalfLight".to_string(),
            ThemeSet::load_from_reader(&mut Cursor::new(include_bytes!(
                "./assets/themes/OneHalfLight.tmTheme"
            )))
            .unwrap(),
        );

        provide_root_context(Rc::new(themes));
        provide_root_context(Rc::new(two_face::syntax::extra_newlines()));
    });

    MaterialThemeProvider::new()
        .child(HttpManager::new().child(Root {}))
        .child(ModalManager {})
        .child(
            rect()
                .layer(Layer::OverlayLevel(10))
                .child(ContextMenuViewer::new()),
        )
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    };

    pretty_env_logger::init();

    let rt = Builder::new_multi_thread().enable_all().build().unwrap();
    let _rt = rt.enter();

    get_unicode_emojis();

    let config = LaunchConfig::new()
        .with_window(
            WindowConfig::new(app)
                .with_title("Ermine - Stoat")
                .with_app_id("live.zomatree.ermine")
                .with_size(1280., 720.)
                .with_decorations(true),
        )
        .with_font("Inter", include_bytes!("./assets/fonts/Inter.ttf") as &[u8])
        .with_font(
            "Fira Code",
            include_bytes!("./assets/fonts/FiraCode.ttf") as &[u8],
        )
        .with_font(
            "Fira Code Bold",
            include_bytes!("./assets/fonts/FiraCode-Bold.ttf") as &[u8],
        )
        .with_font(
            "Fira Code Medium",
            include_bytes!("./assets/fonts/FiraCode-Medium.ttf") as &[u8],
        )
        .with_default_font("Inter")
        .with_plugin(WebViewPlugin::new());

    #[cfg(feature = "performance")]
    let config = config.with_plugin(freya_performance_plugin::PerformanceOverlayPlugin::default());

    launch(config);
}
