use std::{collections::HashMap, time::Duration};

use freya::{
    animation::{AnimNum, AnimatedValue, Ease, Function, OnChange, OnCreation, use_animation},
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;
use tokio::time::sleep;

use crate::{
    AppChannel, ConnectionState, Selection, SizeExt,
    components::{
        AttachmentController, ChannelSettings, Discover, FloatingManager, Home, MaterialIcon,
        Server, ServerList, ServerSettings, Settings, UserProfile,
        material::filled::description,
    },
    consume_material_theme, map_readable,
};

#[derive(PartialEq)]
pub struct Client {}

impl Component for Client {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Selection);
        let selected = radio.slice_current(|state| &state.selection);
        let settings = radio.slice(AppChannel::SettingsPage, |state| &state.settings_page);

        let servers = radio.slice(AppChannel::Servers, |state| &state.servers);
        let server_settings = radio.slice(AppChannel::ServerSettingsPage, |state| {
            &state.server_settings_page
        });
        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);
        let channel_settings = radio.slice(AppChannel::ChannelSettingsPage, |state| {
            &state.channel_settings_page
        });
        let connection_state = radio.slice_mut(AppChannel::State, |state| &mut state.state);
        let user_profile =
            radio.slice_mut(AppChannel::UserProfile, |state| &mut state.user_profile);

        let file_hover = radio.slice_mut(AppChannel::FileHover, |state| &mut state.file_hover);

        let show_connection_state_banner = use_state(|| false);

        let show_settings = use_reactive(&settings.read().is_some());

        let settings_animation = use_animation(move |conf| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Finish);

            let opacity = AnimNum::new(0., 1.)
                .time(350)
                .ease(Ease::Out)
                .function(Function::Expo);

            if show_settings() {
                opacity
            } else {
                opacity.into_reversed()
            }
        });

        let settings_opacity = settings_animation.read().value();

        let show_server_settings = use_reactive(&server_settings.read().is_some());

        let server_settings_animation = use_animation(move |conf| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Finish);

            let opacity = AnimNum::new(0., 1.)
                .time(350)
                .ease(Ease::Out)
                .function(Function::Expo);

            if show_server_settings() {
                opacity
            } else {
                opacity.into_reversed()
            }
        });

        let server_settings_opacity = server_settings_animation.read().value();

        let show_channel_settings = use_reactive(&channel_settings.read().is_some());

        let channel_settings_animation = use_animation(move |conf| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Finish);

            let opacity = AnimNum::new(0., 1.)
                .time(350)
                .ease(Ease::Out)
                .function(Function::Expo);

            if show_channel_settings() {
                opacity
            } else {
                opacity.into_reversed()
            }
        });

        let channel_settings_opacity = channel_settings_animation.read().value();

        use_side_effect({
            let connection_state = connection_state.clone();

            move || {
                let mut show_connection_state_banner = show_connection_state_banner.clone();

                match *connection_state.read() {
                    ConnectionState::Disconnected => *show_connection_state_banner.write() = true,
                    ConnectionState::Reconnecting => *show_connection_state_banner.write() = true,
                    ConnectionState::Reconnected => {
                        *show_connection_state_banner.write() = true;

                        spawn({
                            let mut connection_state = connection_state.clone();
                            async move {
                                sleep(Duration::from_secs_f32(2.5)).await;

                                *show_connection_state_banner.write() = false;
                                *connection_state.write() = ConnectionState::Connected;
                            }
                        });
                    }
                    ConnectionState::Connected => {}
                }
            }
        });

        rect()
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .child(ServerList {})
                    .child(match selected.read().clone() {
                        Selection::Server(server_id) => {
                            let server = radio.slice(AppChannel::Servers, move |state| {
                                state.servers.get(&server_id).unwrap()
                            });

                            Server {
                                server: server.into_readable(),
                            }
                            .into_element()
                        }
                        Selection::Discover => Discover {}.into_element(),
                        Selection::Home => Home {}.into_element(),
                    }),
            )
            .maybe_child(show_connection_state_banner.read().then(|| {
                let connection_state = connection_state.read();

                let (color, background, text) = match *connection_state {
                    ConnectionState::Disconnected => (0xff90909a, 0xff292a2f, "Disconnected"),
                    ConnectionState::Connected => (0xffdde1ff, 0xff384379, "Connected"),
                    ConnectionState::Reconnecting => (0xff90909a, 0xff292a2f, "Reconnecting"),
                    ConnectionState::Reconnected => (0xffdde1ff, 0xff384379, "Reconnected"),
                };

                rect()
                    .position(Position::new_global())
                    .layer(Layer::OverlayLevel(1))
                    .width(Size::window_percent(100.))
                    .height(Size::px(30.))
                    .center()
                    .child(
                        label()
                            .text(text)
                            .font_size(16.)
                            .color(color)
                            .font_weight(FontWeight::SEMI_BOLD),
                    )
                    .background(background)
            }))
            .maybe_child(file_hover.read().then(|| {
                rect()
                    .position(Position::new_global())
                    .width(Size::window_percent(100.))
                    .height(Size::window_percent(100.))
                    .layer(Layer::OverlayLevel(2))
                    .background(0xcc000000)
                    .center()
                    .on_file_drop({
                        let file_hover = file_hover.clone();

                        move |e: Event<FileEventData>| {
                            if !e.file_paths.is_empty()
                                && let Some(controller) =
                                    try_consume_root_context::<Option<AttachmentController>>()
                                        .flatten()
                            {
                                file_hover.clone().set(false);
                                let paths = e.file_paths.clone();

                                spawn(async move {
                                    for path in paths {
                                        controller.add(path).await;
                                    };
                                });
                            };
                        }
                    })
                    .child(
                        rect()
                            .spacing(16.)
                            .child(
                                MaterialIcon::new(description())
                                    .color(theme.md.on_surface.as_argb_u32())
                                    .size(Size::px(64.)),
                            )
                            .child(
                                label()
                                    .text("Drop a file")
                                    .color(theme.md.on_surface.as_argb_u32()),
                            ),
                    )
            }))
            .maybe_child((settings_opacity > 0.).then(|| {
                rect()
                    .position(Position::new_global())
                    .width(Size::window_percent(100.))
                    .height(Size::window_percent(100.))
                    .layer(Layer::OverlayLevel(3))
                    .opacity(settings_opacity)
                    .child(Settings {})
                    .into_element()
            }))
            .maybe_child(
                server_settings
                    .read()
                    .as_ref()
                    .map(|(id, _)| id.clone())
                    .filter(|_| server_settings_opacity > 0.)
                    .map(|server_id| {
                        rect()
                            .position(Position::new_global())
                            .width(Size::window_percent(100.))
                            .height(Size::window_percent(100.))
                            .layer(Layer::OverlayLevel(4))
                            .opacity(server_settings_opacity)
                            .child(ServerSettings {
                                server: map_readable::<HashMap<String, v0::Server>, _>(
                                    servers.into_readable(),
                                    move |servers| servers.get(&server_id).unwrap(),
                                ),
                            })
                            .into_element()
                    }),
            )
            .maybe_child(
                channel_settings
                    .read()
                    .as_ref()
                    .map(|(id, _)| id.clone())
                    .filter(|_| channel_settings_opacity > 0.)
                    .map(|channel_id| {
                        rect()
                            .position(Position::new_global())
                            .width(Size::window_percent(100.))
                            .height(Size::window_percent(100.))
                            .layer(Layer::OverlayLevel(5))
                            .opacity(channel_settings_opacity)
                            .child(ChannelSettings {
                                channel: map_readable::<HashMap<String, v0::Channel>, _>(
                                    channels.into_readable(),
                                    move |channels| channels.get(&channel_id).unwrap(),
                                ),
                            })
                            .into_element()
                    }),
            )
            .maybe_child(user_profile.read().cloned().map(|user_id| {
                let user = radio.slice(AppChannel::Users, move |state| {
                    state.users.get(&user_id).unwrap()
                });

                rect()
                    .position(Position::new_global())
                    .width(Size::window_percent(100.))
                    .height(Size::window_percent(100.))
                    .layer(Layer::OverlayLevel(6))
                    .child(UserProfile {
                        user: user.into_readable(),
                    })
                    .into_element()
            }))
            .child(FloatingManager {})
    }
}
