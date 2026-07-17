use bytes::Bytes;
use freya::{
    elements::image::ImageData,
    prelude::{name::EventName, *},
};
use freya_core::integration::EventHandlerType;
use rustc_hash::FxHashMap;

#[derive(PartialEq)]
pub struct MaterialIcon {
    content: Bytes,
    color: Option<Color>,

    layout: LayoutData,
    image_data: ImageData,
    accessibility: AccessibilityData,
    effect: EffectData,
    event_handlers: FxHashMap<EventName, EventHandlerType>,

    key: DiffKey,
}

impl MaterialIcon {
    pub fn new(content: Bytes) -> Self {
        Self {
            content,
            color: None,
            layout: LayoutData::default(),
            image_data: ImageData::default(),
            accessibility: AccessibilityData::default(),
            effect: EffectData::default(),
            event_handlers: FxHashMap::default(),
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
        let mut inherited_color = use_state::<Option<Color>>(|| self.color.clone());

        rect()
            .child(
                SvgViewer::new(self.content.clone())
                    .accessibility(self.accessibility.clone())
                    .layout(self.layout.clone())
                    .image_data(self.image_data.clone())
                    .effect(self.effect.clone())
                    .with_event_handlers(self.event_handlers.clone())
                    .map(inherited_color(), |this, color| {
                        this.color(color).fill(color)
                    }),
            )
            .on_styled(move |event: Event<StyledEventData>| {
                if inherited_color.peek().is_none() {
                    let color = event.text_style.color.as_color().unwrap_or(Color::BLACK);
                    inherited_color.set_if_modified(Some(color));
                }
            })
    }
}

impl KeyExt for MaterialIcon {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl LayoutExt for MaterialIcon {
    fn get_layout(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl ContainerSizeExt for MaterialIcon {}
impl ContainerPositionExt for MaterialIcon {}

impl ImageExt for MaterialIcon {
    fn get_image_data(&mut self) -> &mut ImageData {
        &mut self.image_data
    }
}

impl AccessibilityExt for MaterialIcon {
    fn get_accessibility_data(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl EffectExt for MaterialIcon {
    fn get_effect(&mut self) -> &mut EffectData {
        &mut self.effect
    }
}

impl EventHandlersExt for MaterialIcon {
    fn get_event_handlers(&mut self) -> &mut FxHashMap<EventName, EventHandlerType> {
        &mut self.event_handlers
    }
}

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
