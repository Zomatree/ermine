use std::{sync::Arc, time::Duration};

use freya::{animation::Function, prelude::*};
use livekit::Room as LivekitRoom;

use crate::{SizeExt, components::FloatingVoice};

pub struct FloatingVoiceContainer {
    pub room: Arc<LivekitRoom>,
}

impl PartialEq for FloatingVoiceContainer {
    fn eq(&self, other: &Self) -> bool {
        self.room.name() == other.room.name()
    }
}

impl Component for FloatingVoiceContainer {
    fn render(&self) -> impl IntoElement {
        let position = use_provide_root_context(|| State::create_global(Pos::TopRight));
        let dragging = use_state(|| false);

        rect()
            .cursor(dragging.read().then_some(CursorIcon::Grabbing))
            .content(Content::Flex)
            .child(
                rect()
                    .height(Size::flex(1.))
                    .content(Content::Flex)
                    .horizontal()
                    .child(Area {
                        room: self.room.clone(),
                        pos: Pos::TopLeft,
                        dragging,
                        position,
                    })
                    .child(Area {
                        room: self.room.clone(),
                        pos: Pos::TopRight,
                        dragging,
                        position,
                    }),
            )
            .child(
                rect()
                    .height(Size::flex(1.))
                    .content(Content::Flex)
                    .horizontal()
                    .child(Area {
                        room: self.room.clone(),
                        pos: Pos::BottomLeft,
                        dragging,
                        position,
                    })
                    .child(Area {
                        room: self.room.clone(),
                        pos: Pos::BottomRight,
                        dragging,
                        position,
                    }),
            )
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
enum Pos {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

struct Area {
    pub room: Arc<LivekitRoom>,
    pub pos: Pos,
    pub dragging: State<bool>,
    pub position: State<Pos>,
}

impl PartialEq for Area {
    fn eq(&self, other: &Self) -> bool {
        self.room.name() == other.room.name()
            && self.pos == other.pos
            && self.dragging == other.dragging
            && self.position == other.position
    }
}

impl Component for Area {
    fn render(&self) -> impl IntoElement {
        let pos = self.pos;
        let mut dragging = self.dragging;
        let mut position = self.position;

        DropZone::<()>::new(move |_| {
            dragging.set(false);
            position.set(pos);
        })
        .size(Size::flex(1.))
        .child(
            rect()
                .cross_align(match pos {
                    Pos::TopLeft | Pos::BottomLeft => Alignment::Start,
                    Pos::TopRight | Pos::BottomRight => Alignment::End,
                })
                .main_align(match pos {
                    Pos::TopLeft | Pos::TopRight => Alignment::Start,
                    Pos::BottomLeft | Pos::BottomRight => Alignment::End,
                })
                .expanded()
                .padding((72., 16.))
                .maybe_child((position() == pos).then(|| {
                    DragZone::new(())
                        .maybe_child((!dragging()).then(|| {
                            Portal::new(())
                                .animation_dependency(position())
                                .key(())
                                .width(Size::px(FloatingVoice::WIDTH))
                                .height(Size::px(FloatingVoice::HEIGHT))
                                .duration(Duration::from_secs_f32(0.2))
                                .function(Function::Back)
                                .child(rect().cursor(CursorIcon::Grab).child(FloatingVoice {
                                    room: self.room.clone(),
                                }))
                        }))
                        .drag_element(
                            rect()
                                .on_sized(move |_| dragging.set(true))
                                .cursor(CursorIcon::Grabbing)
                                .child(
                                    Portal::new(())
                                        .key(())
                                        .width(Size::px(FloatingVoice::WIDTH))
                                        .height(Size::px(FloatingVoice::HEIGHT))
                                        .child(FloatingVoice {
                                            room: self.room.clone(),
                                        }),
                                ),
                        )
                })),
        )
    }
}
