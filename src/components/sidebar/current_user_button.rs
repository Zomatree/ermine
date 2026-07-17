use std::marker::PhantomData;

use euclid::{Point2D, Size2D};
use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Avatar, StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip},
    http,
    theme::Theme,
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct CurrentUserButton {}

impl Component for CurrentUserButton {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let theme = consume_material_theme();

        let current_user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());
        let current_user = radio.slice(AppChannel::Users, move |state| {
            state.users.get(&*current_user_id.read()).unwrap()
        });
        let mut area = use_state(Area::default);
        let mut show_popup = use_state(|| false);

        let user = current_user.read();

        rect()
            .horizontal()
            .child(
                StoatTooltip::new(
                    rect()
                        .spacing(8.)
                        .font_size(11.)
                        .child(label().max_lines(1).text(user.username.clone()))
                        .child(
                            match user.status.as_ref().and_then(|s| s.presence.as_ref()) {
                                Some(v0::Presence::Online) => "Online",
                                Some(v0::Presence::Idle) => "Idle",
                                Some(v0::Presence::Focus) => "Focus",
                                Some(v0::Presence::Busy) => "Busy",
                                Some(v0::Presence::Invisible) => "Invisible",
                                None => "Offline",
                            },
                        ),
                )
                .position(AttachedPosition::Right)
                .child(
                    StoatButton::new()
                        .corner_radius(56.)
                        .child(
                            rect()
                                .width(Size::px(56.))
                                .height(Size::px(56.))
                                .center()
                                .child(
                                    Avatar::new(current_user.clone().into_readable(), None, 42.)
                                        .presence(true),
                                ),
                        )
                        .on_press(move |_| show_popup.toggle()),
                ),
            )
            .maybe_child(show_popup.read().then(|| {
                rect()
                    .layer(Layer::Overlay)
                    .position(Position::new_absolute().left(61.))
                    .width(Size::px(256.))
                    .corner_radius(4.)
                    .overflow(Overflow::Clip)
                    .shadow(Shadow::new().blur(3.).color(Color::BLACK))
                    .on_sized(move |e: Event<SizedEventData>| area.set(e.area))
                    .on_global_pointer_press(move |e: Event<PointerEventData>| {
                        let area = area.read();
                        let pos = e.global_location();

                        let new_area = euclid::Rect {
                            origin: Point2D {
                                x: area.origin.x as f64,
                                y: area.origin.y as f64,
                                _unit: PhantomData::<()>,
                            },
                            size: Size2D {
                                width: area.size.width as f64,
                                height: area.size.height as f64,
                                _unit: PhantomData::<()>,
                            },
                        };

                        if !new_area.contains(pos) {
                            show_popup.toggle();
                        }
                    })
                    .child(
                        rect()
                            .background(theme.md.surface_container.as_argb_u32())
                            .padding((8., 0.))
                            .child({
                                let user = current_user.read();

                                StoatButton::new().child(
                                    rect()
                                        .padding((8., 15.))
                                        // .color(0xffe3e1e9)
                                        .width(Size::Fill)
                                        .child(
                                            rect()
                                                .horizontal()
                                                .spacing(8.)
                                                .child(Avatar::new(
                                                    current_user.clone().into_readable(),
                                                    None,
                                                    32.,
                                                ))
                                                .child(
                                                    rect()
                                                        .child(
                                                            label()
                                                                .text(
                                                                    user.display_name
                                                                        .as_ref()
                                                                        .unwrap_or(&user.username)
                                                                        .clone(),
                                                                )
                                                                .font_size(14),
                                                        )
                                                        .child(
                                                            label()
                                                                .text(format!(
                                                                    "{}#{}",
                                                                    &user.username,
                                                                    &user.discriminator
                                                                ))
                                                                .font_size(12),
                                                        ),
                                                ),
                                        ),
                                )
                            })
                            .child(seperator(&theme))
                            .child(presence_button(v0::Presence::Online, &theme))
                            .child(presence_button(v0::Presence::Idle, &theme))
                            .child(presence_button(v0::Presence::Focus, &theme))
                            .child(presence_button(v0::Presence::Busy, &theme))
                            .child(presence_button(v0::Presence::Invisible, &theme))
                            .child(seperator(&theme)),
                    )
            }))
    }
}

fn seperator(theme: &Theme) -> Rect {
    rect()
        .margin((4., 0.))
        .width(Size::Fill)
        .height(Size::px(1.))
        .background(theme.md.outline_variant.as_argb_u32())
}

fn presence_button(value: v0::Presence, theme: &Theme) -> StoatButton {
    StoatButton::new().width(Size::Fill).child(
        rect()
            .padding((8., 16.))
            .horizontal()
            .width(Size::Fill)
            .cross_align(Alignment::Center)
            .spacing(8.)
            .on_press({
                let value = value.clone();

                move |_| {
                    let value = value.clone();

                    spawn(async move {
                        http()
                            .edit_user(
                                "@me",
                                &v0::DataEditUser {
                                    display_name: None,
                                    avatar: None,
                                    status: Some(v0::UserStatus {
                                        text: None,
                                        presence: Some(value),
                                    }),
                                    profile: None,
                                    badges: None,
                                    flags: None,
                                    pronouns: None,
                                    remove: Vec::new(),
                                },
                            )
                            .await
                            .unwrap();
                    });
                }
            })
            .child(
                rect()
                    .width(Size::px(10.))
                    .height(Size::px(10.))
                    .corner_radius(10.)
                    .overflow(Overflow::Clip)
                    .background(match &value {
                        v0::Presence::Online => theme.stoat.presence_online,
                        v0::Presence::Idle => theme.stoat.presence_idle,
                        v0::Presence::Focus => theme.stoat.presence_focus,
                        v0::Presence::Busy => theme.stoat.presence_busy,
                        v0::Presence::Invisible => theme.stoat.presence_invisible,
                    }),
            )
            .child(match value {
                v0::Presence::Online => "Online",
                v0::Presence::Idle => "Idle",
                v0::Presence::Focus => "Focus",
                v0::Presence::Busy => "Do Not Disturb",
                v0::Presence::Invisible => "Invisble",
            })
            .font_size(14),
    )
}
