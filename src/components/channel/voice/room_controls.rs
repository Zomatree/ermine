use std::sync::Arc;

use freya::prelude::*;
use livekit::Room as LivekitRoom;

use crate::{
    Room, SizeExt,
    components::{
        MaterialIcon, StoatButton, StoatButtonLayoutThemePartialExt,
        material::{
            filled::{
                call_end, camera_alt as filled_camera_alt, headphones, headset_off, mic, mic_off,
                screen_share as filled_screen_share,
            },
            outlined::{camera_alt, screen_share},
        },
    },
    consume_material_theme,
};

pub struct RoomControls {
    pub room: Arc<LivekitRoom>,
}

impl PartialEq for RoomControls {
    fn eq(&self, other: &Self) -> bool {
        self.room.name() == other.room.name()
    }
}

impl Component for RoomControls {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let room = consume_context::<Room>();
        let is_camera = use_memo(|| false);

        rect()
            .horizontal()
            .padding(8.)
            .corner_radius(48.)
            .spacing(8.)
            .background(theme.md.surface_container.as_argb_u32())
            .child(
                StoatButton::new()
                    .corner_radius(20.)
                    .on_press({
                        move |_| {
                            spawn(async move {
                                room.toggle_mute().await;
                            });
                        }
                    })
                    .child({
                        let is_muted = room.is_muted();

                        rect()
                            .background(if is_muted {
                                theme.md.secondary_container.as_argb_u32()
                            } else {
                                theme.md.primary.as_argb_u32()
                            })
                            .color(if is_muted {
                                theme.md.on_secondary_container.as_argb_u32()
                            } else {
                                theme.md.on_primary.as_argb_u32()
                            })
                            .width(Size::px(40.))
                            .height(Size::px(40.))
                            .center()
                            .child(
                                MaterialIcon::new(if is_muted { mic_off() } else { mic() })
                                    .size(Size::px(24.)),
                            )
                    }),
            )
            .child(
                StoatButton::new()
                    .corner_radius(20.)
                    .on_press({
                        move |_| {
                            room.toggle_deafen();
                        }
                    })
                    .child({
                        let is_deafened = room.is_deafened();

                        rect()
                            .background(if is_deafened {
                                theme.md.secondary_container.as_argb_u32()
                            } else {
                                theme.md.primary.as_argb_u32()
                            })
                            .color(if is_deafened {
                                theme.md.on_secondary_container.as_argb_u32()
                            } else {
                                theme.md.on_primary.as_argb_u32()
                            })
                            .width(Size::px(40.))
                            .height(Size::px(40.))
                            .center()
                            .child(
                                MaterialIcon::new(if is_deafened {
                                    headset_off()
                                } else {
                                    headphones()
                                })
                                .size(Size::px(24.)),
                            )
                    }),
            )
            .child(
                StoatButton::new()
                    .corner_radius(20.)
                    .on_press({
                        // let room = room.clone();

                        move |_| {
                            room.start_camera();
                        }
                    })
                    .child({
                        let is_camera = is_camera();

                        rect()
                            .background(if !is_camera {
                                theme.md.secondary_container.as_argb_u32()
                            } else {
                                theme.md.primary.as_argb_u32()
                            })
                            .color(if !is_camera {
                                theme.md.on_secondary_container.as_argb_u32()
                            } else {
                                theme.md.on_primary.as_argb_u32()
                            })
                            .width(Size::px(40.))
                            .height(Size::px(40.))
                            .center()
                            .child(
                                MaterialIcon::new(if !is_camera {
                                    camera_alt()
                                } else {
                                    filled_camera_alt()
                                })
                                .size(Size::px(24.)),
                            )
                    }),
            )
            .child(
                StoatButton::new()
                    .corner_radius(20.)
                    .on_press({
                        move |_| {
                            room.toggle_screensharing();
                        }
                    })
                    .child({
                        let is_screenshare = room.is_screensharing();

                        rect()
                            .background(if !is_screenshare {
                                theme.md.secondary_container.as_argb_u32()
                            } else {
                                theme.md.primary.as_argb_u32()
                            })
                            .color(if !is_screenshare {
                                theme.md.on_secondary_container.as_argb_u32()
                            } else {
                                theme.md.on_primary.as_argb_u32()
                            })
                            .width(Size::px(40.))
                            .height(Size::px(40.))
                            .center()
                            .child(
                                MaterialIcon::new(if !is_screenshare {
                                    screen_share()
                                } else {
                                    filled_screen_share()
                                })
                                .size(Size::px(24.)),
                            )
                    }),
            )
            .child(
                StoatButton::new()
                    .corner_radius(20.)
                    .on_press({
                        move |_| {
                            spawn_in_window(async move {
                                room.close().await;
                            });
                        }
                    })
                    .child(
                        rect()
                            .background(theme.md.error.as_argb_u32())
                            .color(theme.md.on_error.as_argb_u32())
                            .width(Size::px(56.))
                            .height(Size::px(40.))
                            .center()
                            .child(
                                MaterialIcon::new(call_end())
                                    .size(Size::px(24.))
                                    .margin((2., 0., 0., 0.)),
                            ),
                    ),
            )
    }
}
