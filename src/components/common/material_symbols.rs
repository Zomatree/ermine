use bytes::Bytes;
use freya::prelude::*;

use crate::{SizeExt, components::svg::svg};

#[derive(PartialEq)]
pub struct MaterialIcon {
    content: Bytes,
    color: Option<Color>,

    layout: LayoutData,
    style: StyleState,
    accessibility: AccessibilityData,
    effect: EffectData,
    event_handlers: EventHandlers,

    key: DiffKey,
}

impl MaterialIcon {
    pub fn new(content: Bytes) -> Self {
        Self {
            content,
            color: None,
            layout: LayoutData::default(),
            style: StyleState::default(),
            accessibility: AccessibilityData::default(),
            effect: EffectData::default(),
            event_handlers: EventHandlers::default(),
            key: DiffKey::None,
        }
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl Component for MaterialIcon {
    fn render(&self) -> impl IntoElement {
        rect()
            .layout(self.layout.clone())
            .corner_radius(self.style.corner_radius)
            .child(
                svg(self.content.clone())
                    .size(Size::Fill)
                    .map(self.color, |this, color| this.color(color))
                    .accessibility(self.accessibility.clone())
                    .effect(self.effect.clone())
                    .event_handlers(self.event_handlers.clone()),
            )
    }
}

impl KeyExt for MaterialIcon {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl LayoutExt for MaterialIcon {
    fn layout_mut(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl ContainerSizeExt for MaterialIcon {}
impl ContainerPositionExt for MaterialIcon {}

impl AccessibilityExt for MaterialIcon {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl EffectExt for MaterialIcon {
    fn effect_mut(&mut self) -> &mut EffectData {
        &mut self.effect
    }
}

impl EventHandlersExt for MaterialIcon {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}

impl StyleExt for MaterialIcon {
    fn style_mut(&mut self) -> &mut StyleState {
        &mut self.style
    }
}

impl DecorationExt for MaterialIcon {}

#[macro_export]
macro_rules! generate_svg {
    ($name:ident, $path:expr) => {
        #[allow(unused)]
        pub fn $name() -> bytes::Bytes {
            bytes::Bytes::from_static(include_bytes!($path))
        }
    };
}

pub mod filled {
    include!(concat!(env!("OUT_DIR"), "/filled.rs"));
}

pub mod outlined {
    include!(concat!(env!("OUT_DIR"), "/outlined.rs"));
}

pub mod round {
    include!(concat!(env!("OUT_DIR"), "/round.rs"));
}
