use freya::{prelude::*, webview::WebViewPlugin};
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

use crate::components::{
    HttpManager, MaterialThemeProvider, Root, StoatButtonColorsThemePreference,
    StoatButtonLayoutThemePreference,
};

pub const BASE: &str = "https://api.stoat.chat";

fn app() -> impl IntoElement {
    let config = use_hook(|| {
        let state = State::create(read_config());
        provide_context(state);

        state
    });

    use_side_effect(move || {
        let new_value = config.read();

        write_config(&new_value);
    });

    use_init_theme(|| {
        let mut theme = dark_theme();
        // theme.colors.text_primary = 0xffe3e1e9.into();

        theme.set(
            "stoat_button",
            StoatButtonColorsThemePreference {
                background: Preference::Specific(Color::TRANSPARENT),
                hover_background: Preference::Specific(Color::TRANSPARENT),
                border_fill: Preference::Specific(Color::TRANSPARENT),
                focus_border_fill: Preference::Specific(Color::TRANSPARENT),
                color: Preference::Specific(Color::TRANSPARENT),
            },
        );

        theme.set(
            "stoat_button_layout",
            StoatButtonLayoutThemePreference {
                margin: Preference::Specific(Gaps::new_all(0.)),
                corner_radius: Preference::Specific(CornerRadius::new_all(0.)),
                width: Preference::Specific(Size::Inner),
                height: Preference::Specific(Size::Inner),
                padding: Preference::Specific(Gaps::new_all(0.)),
            },
        );

        theme
    });

    MaterialThemeProvider::new()
        .child(HttpManager::new().child(Root {}))
        .child(
            rect()
                .layer(Layer::RelativeOverlay(10))
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

    launch(
        LaunchConfig::new()
            .with_window(
                WindowConfig::new(app)
                    .with_title("Ermine - Stoat")
                    .with_app_id("live.zomatree.ermine")
                    .with_size(1280., 720.)
                    .with_decorations(true),
            )
            .with_plugin(WebViewPlugin::new()),
    );
}
