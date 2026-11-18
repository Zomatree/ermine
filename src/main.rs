use std::{io::Cursor, rc::Rc};

use dpi::{PhysicalPosition, PhysicalSize};
use freya::{
    prelude::*, radio::use_init_radio_station, winit::platform::macos::WindowAttributesExtMacOS,
};
use freya_winit::plugins::{FreyaPlugin, PluginEvent};
use tokio::runtime::Builder;

pub mod components;
pub mod config;
pub mod error;
pub mod state;
pub mod stoat;
pub mod theme;
pub mod utils;
pub mod voice;

pub use config::*;
pub use error::*;
pub use state::*;
pub use stoat::*;
pub use theme::*;
pub use utils::*;
pub use voice::*;

use crate::components::{MaterialThemeProvider, ModalManager, Window};

pub type ThemeSet = Rc<syntect::highlighting::ThemeSet>;
pub type SyntaxSet = Rc<syntect::parsing::SyntaxSet>;

fn app() -> impl IntoElement {
    use_init_radio_station::<AppState, AppChannel>(AppState::new);
    use_clipboard();

    let config = use_config();

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
        .child(Window {})
        .child(ModalManager {})
        .child(
            rect()
                .layer(Layer::OverlayLevel(10))
                .child(ContextMenuViewer::new()),
        )
}

struct Plugin {
    pub size: State<PhysicalSize<u32>>,
    pub position: State<Option<PhysicalPosition<i32>>>,
}

impl Plugin {
    pub fn new(config: State<Config>) -> Self {
        Self {
            size: State::create_global(config.read().size),
            position: State::create_global(config.read().position),
        }
    }
}

impl FreyaPlugin for Plugin {
    fn plugin_id(&self) -> &'static str {
        "win-size-plugin"
    }

    fn on_event(
        &mut self,
        event: &mut freya_winit::plugins::PluginEvent,
        _handle: freya_winit::plugins::PluginHandle,
    ) {
        if let PluginEvent::AfterRender { window, .. } = event {
            let Ok(pos) = window.outer_position() else {
                return;
            };
            let size = window.inner_size();

            let has_changed = { Some(pos) != *self.position.read() || size != *self.size.read() };

            if has_changed {
                self.position.set(Some(pos));
                self.size.set(size);
            }
        }
    }

    fn root_component(&self, root: Element) -> Element {
        #[derive(PartialEq)]
        struct Inner {
            pub size: State<PhysicalSize<u32>>,
            pub position: State<Option<PhysicalPosition<i32>>>,
            pub inner: Element,
        }

        impl Component for Inner {
            fn render(&self) -> impl IntoElement {
                let mut config = use_config();

                let size = self.size;
                let position = self.position;

                use_changed((size(), position()), move |(size, position)| {
                    if let Some(mut config) = config.try_write() {
                        config.size = *size;
                        config.position = *position;
                    }
                });

                self.inner.clone()
            }
        }

        Inner {
            size: self.size,
            position: self.position,
            inner: root,
        }
        .into_element()
    }
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

    let config_state = State::create_global(read_config());
    let position = { config_state.read().position };

    let launch_config = LaunchConfig::new()
        .with_window(
            WindowConfig::new(app)
                .with_title("Ermine - Stoat")
                .with_app_id("live.zomatree.ermine")
                .with_size(
                    config_state.read().size.width as f64,
                    config_state.read().size.height as f64,
                )
                .with_decorations(true)
                .with_window_attributes(move |mut attrs, _| {
                    if let Some(pos) = position {
                        attrs = attrs.with_position(pos)
                    };

                    #[cfg(target_os = "macos")]
                    let attrs = attrs
                        .with_fullsize_content_view(true)
                        .with_titlebar_transparent(true)
                        .with_title_hidden(true);

                    attrs
                }),
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
        .with_global_context(config_state)
        .with_plugin(Plugin::new(config_state))
        .with_default_font("Inter");

    #[cfg(feature = "performance")]
    let launch_config = launch_config.with_plugin(freya_metrics_plugin::MetricsPlugin::default());

    launch(launch_config);
}
