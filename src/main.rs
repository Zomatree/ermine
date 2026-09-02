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
        .with_font("Inter", include_bytes!("./assets/Inter.ttf") as &[u8])
        .with_font(
            "Fira Code",
            include_bytes!("./assets/FiraCode.ttf") as &[u8],
        )
        .with_default_font("Inter")
        .with_plugin(WebViewPlugin::new());

    #[cfg(feature = "performance")]
    let config = config.with_plugin(freya_performance_plugin::PerformanceOverlayPlugin::default());

    launch(config);
}
