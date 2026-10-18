use std::sync::Arc;

use freya::prelude::*;
use livekit::Room as LivekitRoom;

use crate::{components::RoomControls, consume_material_theme};

pub struct FloatingVoice {
    pub room: Arc<LivekitRoom>,
}

impl FloatingVoice {
    pub const HEIGHT: f32 = 170.;
    pub const WIDTH: f32 = 300.;
}

impl PartialEq for FloatingVoice {
    fn eq(&self, other: &Self) -> bool {
        self.room.name() == other.room.name()
    }
}

impl Component for FloatingVoice {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        rect()
            .width(Size::px(Self::WIDTH))
            .height(Size::px(Self::HEIGHT))
            .background(theme.md.secondary_container.as_argb_u32())
            .corner_radius(16.)
            .padding(8.)
            .content(Content::Flex)
            .spacing(8.)
            .cross_align(Alignment::Center)
            .child(rect())
            .child(rect().height(Size::flex(1.)).width(Size::Fill))
            .child(rect().child(RoomControls {
                room: self.room.clone(),
                compact: true,
            }))
    }
}
