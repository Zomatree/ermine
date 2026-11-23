use dpi::{PhysicalPosition, PhysicalSize};
use freya::prelude::*;
use freya_winit::plugins::{FreyaPlugin, PluginEvent};

use crate::{Config, use_changed, use_config};

pub struct Plugin {
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
            let size = window.surface_size();

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
