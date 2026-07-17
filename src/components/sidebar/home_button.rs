use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel, Selection, SizeExt,
    components::{
        StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip,
        material::{MaterialIcon, filled::home},
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct HomeButton {}

impl Component for HomeButton {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::Selection);
        let theme = consume_material_theme();

        StoatTooltip::new(
            label()
                .max_lines(1)
                .text("You have 0 pending friend requests.")
                .font_size(11.),
        )
        .position(AttachedPosition::Right)
        .child(
            rect()
                .width(Size::px(56.))
                .height(Size::px(56.))
                .child(
                    rect()
                        .position(Position::new_absolute().left(-8.).top(12.))
                        .width(Size::px(12.))
                        .height(Size::px(32.))
                        .corner_radius(4.)
                        .background(theme.md.on_surface.as_argb_u32())
                        .opacity(if radio.read().selection == Selection::Home {
                            1.
                        } else {
                            0.
                        }),
                )
                .child(
                    rect().expanded().center().child(
                        StoatButton::new().corner_radius(42.).child(
                            rect()
                                .width(Size::px(42.))
                                .height(Size::px(42.))
                                .background(theme.md.surface_container_low.as_argb_u32())
                                .center()
                                .on_press(move |_| {
                                    radio.write().selection = Selection::Home;
                                    radio
                                        .write_channel(AppChannel::SelectedChannel)
                                        .selected_channel = None;
                                })
                                .child(
                                    MaterialIcon::new(home())
                                        .size(Size::px(24.))
                                        .color(theme.md.on_surface.as_argb_u32()),
                                ),
                        ),
                    ),
                ),
        )

        // rect()
        //     .corner_radius(42.)
        //     .overflow(Overflow::Clip)
        //     .width(Size::px(42.))
        //     .height(Size::px(42.))
        //     .center()
        //     .child(
        //         svg(house())
        //             .width(Size::px(24.))
        //             .height(Size::px(24.))
        //             .color(0xffe3e1e9),
        //     )
        //     .background(0xff1b1b21)
        //     .on_pointer_enter(|_| {
        //         Cursor::set(CursorIcon::Pointer);
        //     })
        //     .on_pointer_leave(|_| {
        //         Cursor::set(CursorIcon::Default);
        //     })
        //     .on_press(move |_| {
        //         radio.write().selection = Selection::Home;
        //         radio
        //             .write_channel(AppChannel::SelectedChannel)
        //             .selected_channel = None;
        //     })
    }
}
