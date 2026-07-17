use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, Config, NotificationBadge, Selection, components::{ServerContextMenu, StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip, server_icon}, consume_material_theme, get_unread_badge, is_server_muted
};

#[derive(PartialEq)]
pub struct ServerListButton {
    pub server: Readable<v0::Server>,
}

impl Component for ServerListButton {
    fn render(&self) -> impl IntoElement {
        let config = use_consume::<State<Config>>();
        let mut radio = use_radio(AppChannel::Servers);
        let theme = consume_material_theme();
        let mut hovering = use_state(|| false);

        let selection = radio.slice(AppChannel::Selection, |state| &state.selection);
        let notifications_settings = radio.slice(AppChannel::Settings("notifications"), |state| {
            &state.settings.notifications
        });
        let unreads = radio.slice(AppChannel::ChannelUnreads, |state| &state.channel_unreads);
        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);

        let server = use_memo({
            let server = self.server.clone();
            move || server.read().clone()
        });

        let muted = use_memo({
            let notifications_settings = notifications_settings.clone().into_readable();
            let server = server.clone();

            move || is_server_muted(&server.peek().id, notifications_settings.clone())
        });

        let badge = use_memo({
            let server = server.clone();

            move || {
                if !*muted.read() {
                    let badges = server
                        .read()
                        .channels
                        .iter()
                        .filter_map(|id| {
                            if let Some(unread) = unreads.read().get(id) {
                                Some((id.clone(), unread.clone()))
                            } else {
                                None
                            }
                        })
                        .filter_map(|(id, unread)| {
                            channels
                                .read()
                                .get(&id)
                                .and_then(|channel| get_unread_badge(channel, &unread))
                        })
                        .collect::<Vec<_>>();

                    let mut mention_count = 0;
                    let mut has_unread = false;

                    for badge in badges {
                        match badge {
                            NotificationBadge::Unread => has_unread = true,
                            NotificationBadge::Mentions(count) => mention_count += count,
                        }
                    }

                    if mention_count > 0 {
                        Some(NotificationBadge::Mentions(mention_count))
                    } else if has_unread {
                        Some(NotificationBadge::Unread)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        });

        let selected =
            use_memo(move || &*selection.read() == &Selection::Server(server.peek().id.clone()));

        rect()
            .horizontal()
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .on_secondary_down({
                let server_id = server.read().id.clone();

                move |e| {
                    ContextMenu::open_from_event(
                        &e,
                        Menu::new().child(ServerContextMenu { server_id: server_id.clone() }),
                    );
                }
            })
            .maybe_child(
                (*selected.read() || *badge.read() == Some(NotificationBadge::Unread)).then(|| {
                    rect()
                        .width(Size::px(4.))
                        .height(Size::px(if *selected.read() { 32. } else { 8. }))
                        .layer(Layer::Relative(1))
                        .position(Position::new_absolute().left(0.).top(if *selected.read() {
                            12.
                        } else {
                            24.
                        }))
                        .corner_radius(CornerRadius {
                            top_left: 0.,
                            top_right: 4.,
                            bottom_left: 0.,
                            bottom_right: 4.,
                            smoothing: 0.,
                        })
                        .background(theme.md.on_surface.as_argb_u32())
                }),
            )
            .child(
                StoatTooltip::new(label().max_lines(1).text(server.read().name.clone()))
                    .position(AttachedPosition::Right)
                    .child(
                        rect()
                            .center()
                            .width(Size::px(56.0))
                            .height(Size::px(56.0))
                            .child(
                                rect()
                                    .width(Size::px(42.0))
                                    .height(Size::px(42.0))
                                    .child(
                                        StoatButton::new()
                                            .corner_radius(42.)
                                            .child(
                                                rect()
                                                    .width(Size::px(42.0))
                                                    .height(Size::px(42.0))
                                                    // .overflow(Overflow::Clip)
                                                    .child(server_icon(&server.read(), &theme)),
                                            )
                                            .on_press({
                                                move |_| {
                                                    radio
                                                        .write_channel(AppChannel::Selection)
                                                        .selection =
                                                        Selection::Server(server.peek().id.clone());

                                                    let channel_id = config
                                                        .read()
                                                        .last_channels
                                                        .get(&server.peek().id)
                                                        .cloned()
                                                        .or_else(|| {
                                                            let channels = radio.slice(
                                                                AppChannel::Channels,
                                                                |state| &state.channels,
                                                            );

                                                            server
                                                                .read()
                                                                .channels
                                                                .iter()
                                                                .find(|&id| {
                                                                    channels.read().contains_key(id)
                                                                })
                                                                .cloned()
                                                        });

                                                    radio
                                                        .write_channel(AppChannel::SelectedChannel)
                                                        .selected_channel = channel_id;
                                                }
                                            }),
                                    )
                                    .maybe_child(
                                        badge
                                            .read()
                                            .as_ref()
                                            .and_then(|badge| {
                                                if let NotificationBadge::Mentions(count) = badge {
                                                    Some(count)
                                                } else {
                                                    None
                                                }
                                            })
                                            .map(|count| {
                                                rect()
                                                    .position(
                                                        Position::new_absolute().right(0.).top(0.),
                                                    )
                                                    .layer(Layer::Relative(10))
                                                    .width(Size::px(13.))
                                                    .height(Size::px(13.))
                                                    .corner_radius(13.)
                                                    .center()
                                                    .background(theme.md.error.as_argb_u32())
                                                    .color(theme.md.on_error.as_argb_u32())
                                                    .font_size(10.)
                                                    .child(if *count <= 9 {
                                                        count.to_string()
                                                    } else {
                                                        "+".to_string()
                                                    })
                                            }),
                                    ),
                            ),
                    ),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.server.peek().id).into()
    }
}
