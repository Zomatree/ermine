use freya::{
    prelude::*,
    radio::{use_radio, use_radio_station},
};
use stoat_models::v0;

use crate::{
    AppChannel, Selection,
    components::{
        ModalValue, ServerIcon, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt, use_modals,
    },
    consume_material_theme, http, insert_channel, insert_member, insert_server, insert_user,
};

#[derive(PartialEq)]
pub struct Invite {
    pub id: String,
}

impl Component for Invite {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let station = use_radio_station();
        let radio = use_radio(AppChannel::Servers);
        let servers = radio.slice_current(|state| &state.servers);
        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);
        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());
        let selected_channel = radio.slice_mut(AppChannel::SelectedChannel, |state| {
            &mut state.selected_channel
        });
        let selection = radio.slice_mut(AppChannel::Selection, |state| &mut state.selection);

        let mut info = use_state(|| None);

        use_hook(|| {
            let id = self.id.clone();

            spawn(async move {
                info.set(Some(http().fetch_invite(&id).await.ok()));
            });
        });

        let info = info.read();

        rect().width(Size::Fill).padding((2., 0.)).child(
            rect()
                .height(Size::px(64.))
                .width(Size::px(320.))
                .horizontal()
                .corner_radius(12.)
                .padding(8.)
                .spacing(8.)
                .cross_align(Alignment::Center)
                .content(Content::Flex)
                .background(theme.md.secondary_container.as_argb_u32())
                .color(theme.md.on_secondary_container.as_argb_u32())
                .maybe_child(info.is_none().then(|| {
                    CircularLoader::new()
                        .size(42.)
                        .primary_color(theme.md.on_secondary_container.as_argb_u32())
                }))
                .map(info.as_ref(), |this, info| {
                    if let Some(info) = info {
                        let has_joined = match info {
                            v0::InviteResponse::Server { server_id, .. } => {
                                servers.read().contains_key(server_id)
                            }
                            v0::InviteResponse::Group { channel_id, .. } => {
                                channels.read().contains_key(channel_id)
                            }
                        };

                        this.maybe_child(
                            if let v0::InviteResponse::Server {
                                server_icon,
                                server_name,
                                ..
                            } = info
                            {
                                Some(rect().corner_radius(21.).overflow(Overflow::Clip).child(
                                    ServerIcon::from_values(
                                        server_icon.clone(),
                                        server_name.clone(),
                                        42.,
                                    ),
                                ))
                            } else {
                                None
                            },
                        )
                        .child(match info {
                            v0::InviteResponse::Server {
                                server_name,
                                member_count,
                                ..
                            } => rect()
                                .width(Size::flex(1.))
                                .child(
                                    label()
                                        .font_size(14.)
                                        .font_weight(600)
                                        .text(server_name.clone()),
                                )
                                .child(
                                    label()
                                        .font_size(12.)
                                        .text(format!("{member_count} members")),
                                )
                                .into_element(),

                            v0::InviteResponse::Group { channel_name, .. } => label()
                                .width(Size::flex(1.))
                                .font_size(14.)
                                .font_weight(600)
                                .text(channel_name.clone())
                                .into_element(),
                        })
                        .child(
                            StoatButton::new()
                                .enabled(!has_joined)
                                .corner_radius(20.)
                                .color(theme.md.on_primary.as_argb_u32())
                                .background(theme.md.primary.as_argb_u32())
                                .on_press({
                                    let id = self.id.clone();

                                    move |_| {
                                        let id = id.clone();
                                        let user_id = user_id.clone();
                                        let mut selected_channel = selected_channel.clone();
                                        let mut selection = selection.clone();

                                        spawn_forever(async move {
                                            match http().join_invite(&id).await {
                                                Ok(response) => match response {
                                                    v0::InviteJoinResponse::Server {
                                                        channels,
                                                        server,
                                                    } => {
                                                        let id = server.id.clone();

                                                        if let Some(first_channel) =
                                                            channels.first()
                                                        {
                                                            *selected_channel.write() = Some((
                                                                first_channel.id().to_string(),
                                                                None,
                                                            ));
                                                        }

                                                        for channel in channels {
                                                            insert_channel(channel, station);
                                                        }

                                                        insert_server(server, station);

                                                        let user_id = user_id.read().clone();

                                                        let member = http()
                                                            .fetch_member(&id, &user_id)
                                                            .await
                                                            .unwrap();

                                                        insert_member(member, station);

                                                        *selection.write() =
                                                            Selection::Server(id.clone());
                                                    }
                                                    v0::InviteJoinResponse::Group {
                                                        channel,
                                                        users,
                                                    } => {
                                                        let id = channel.id().to_string();

                                                        insert_channel(channel, station);

                                                        for user in users {
                                                            insert_user(user, station);
                                                        }

                                                        *selected_channel.write() =
                                                            Some((id, None));
                                                        *selection.write() = Selection::Home;
                                                    }
                                                },
                                                Err(error) => {
                                                    modals
                                                        .write()
                                                        .push_modal(ModalValue::Error { error });
                                                }
                                            };
                                        });
                                    }
                                })
                                .child(
                                    rect()
                                        .center()
                                        .padding((0., 16.))
                                        .height(Size::px(40.))
                                        .child(if has_joined { "Joined" } else { "Join" }),
                                ),
                        )
                    } else {
                        this.child("Unknown Invite")
                    }
                }),
        )
    }
}
