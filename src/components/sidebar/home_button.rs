use freya::{animation::{AnimNum, use_animation_transition}, prelude::*, radio::{use_radio, use_radio_station}};
use stoat_models::v0;

use crate::{
    AppChannel, Selection, SizeExt, components::{
        StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip,
        material::{MaterialIcon, filled::home},
    }, consume_material_theme, fast_func, set_selected_channel, set_selection,
};

#[derive(PartialEq)]
pub struct HomeButton {}

impl Component for HomeButton {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let station = use_radio_station();
        let radio = use_radio(AppChannel::Selection);
        let selection = radio.slice_current(|state| &state.selection);
        let user = radio.slice(AppChannel::Users, |state| {
            state.users.get(state.user_id.as_ref().unwrap()).unwrap()
        });

        let mut hovering = use_state(|| false);

        let friend_request_count = use_memo(move || {
            user.read()
                .relations
                .iter()
                .filter(|rel| rel.status == v0::RelationshipStatus::Incoming)
                .count()
        });

        let indicator_height = if &*selection.read() == &Selection::Home {
            32.
        } else if hovering() {
            16.
        } else {
            0.
        };

        let indicator_anim = use_animation_transition(indicator_height, move |before, after| {
            AnimNum::new(before, after).time(100).function(fast_func())
        });

        StoatTooltip::new(
            label()
                .max_lines(1)
                .text(format!(
                    "You have {} pending friend requests.",
                    friend_request_count()
                ))
                .font_size(11.),
        )
        .position(AttachedPosition::Right)
        .child(
            rect()
                .width(Size::px(56.))
                .height(Size::px(56.))
                .on_pointer_over(move |_| {
                    hovering.set(true);
                })
                .on_pointer_out(move |_| hovering.set_if_modified(false))
                .child(
                    rect()
                        .center()
                        .position(Position::new_absolute().left(0.))
                        .height(Size::px(56.))
                        .layer(Layer::Relative(1))
                        .child(
                            rect()
                                .width(Size::px(4.))
                                .height(Size::px(indicator_anim.read().value()))
                                .corner_radius(CornerRadius::new(0., 4., 4., 0.))
                                .background(theme.md.on_surface.as_argb_u32()),
                        ),
                )
                .child(
                    rect().expanded().center().child(
                        rect()
                            .width(Size::px(42.))
                            .height(Size::px(42.))
                            .child(
                                StoatButton::new().corner_radius(42.).child(
                                    rect()
                                        .width(Size::px(42.))
                                        .height(Size::px(42.))
                                        .background(theme.md.surface_container_low.as_argb_u32())
                                        .center()
                                        .on_press(move |_| {
                                            set_selection(Selection::Home, station);
                                            set_selected_channel(None, station);
                                        })
                                        .child(
                                            MaterialIcon::new(home())
                                                .size(Size::px(32.))
                                                .color(theme.md.on_surface.as_argb_u32()),
                                        ),
                                ),
                            )
                            .maybe_child({
                                let count = friend_request_count();
                                (count != 0).then(|| {
                                    rect()
                                        .position(Position::new_absolute().right(0.).top(0.))
                                        .layer(Layer::Relative(10))
                                        .width(Size::px(13.))
                                        .height(Size::px(13.))
                                        .corner_radius(13.)
                                        .center()
                                        .background(theme.md.error.as_argb_u32())
                                        .color(theme.md.on_error.as_argb_u32())
                                        .font_size(8.)
                                        .font_weight(FontWeight::SEMI_BOLD)
                                        .child(if count <= 9 {
                                            count.to_string()
                                        } else {
                                            "+".to_string()
                                        })
                                })
                            }),
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
