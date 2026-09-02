use freya::{prelude::*, radio::use_radio};
use jiff::{Timestamp, tz::TimeZone};
use stoat_models::v0;
use stoat_permissions::{ChannelPermission, UserPermission};
use ulid::Ulid;

use crate::{
    AppChannel, SizeExt, calculate_server_permissions, calculate_user_permissions,
    components::{
        Dialog, GroupDMIcon, ServerIcon, SingleLineEntry, StoatButton,
        StoatButtonLayoutThemePartialExt, checkbox::StoatCheckbox, file_image, use_modals,
    },
    consume_material_theme, http, user_permissions_query,
};

#[derive(Clone)]
enum InviteDestination {
    Server((String, Readable<v0::Server>)),
    Group((String, Readable<v0::Channel>)),
}

impl InviteDestination {
    pub fn id(&self) -> &str {
        match self {
            InviteDestination::Server((id, _)) => id,
            InviteDestination::Group((id, _)) => id,
        }
    }
}

impl PartialEq for InviteDestination {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[derive(PartialEq)]
pub struct InviteBot {
    pub bot: v0::PublicBot,
}

impl Component for InviteBot {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let radio = use_radio(AppChannel::Servers);
        let servers = radio.slice_current(|state| &state.servers);
        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);
        let user = radio.slice(AppChannel::Users, |state| {
            state.users.get(state.user_id.as_ref().unwrap()).unwrap()
        });

        let query = use_state(String::new);
        let include_servers = use_state(|| true);
        let include_groups = use_state(|| true);
        let mut selected = use_state(|| None::<InviteDestination>);

        let created_at = use_hook(|| {
            Timestamp::try_from(Ulid::from_string(&self.bot.id).unwrap().datetime())
                .unwrap()
                .to_zoned(TimeZone::system())
                .strftime("%d/%m/%Y")
                .to_string()
        });

        let mut allowed_channels = use_state(Vec::<(String, Readable<v0::Channel>)>::new);
        let mut allowed_servers = use_state(Vec::<(String, Readable<v0::Server>)>::new);

        use_side_effect({
            let user = user.clone();

            move || {
                spawn({
                    let channels = channels.clone();
                    let user = user.clone();

                    async move {
                        let mut allowed = Vec::new();

                        let channels = channels.read();
                        let query = user_permissions_query(radio).user(user.read().cloned());

                        for channel in channels.values() {
                            if matches!(channel, v0::Channel::Group { .. }) {
                                let permissions = calculate_user_permissions(
                                    &mut query.clone().channel(channel.clone()),
                                )
                                .await;

                                if permissions.has_user_permission(UserPermission::Invite) {
                                    let id = channel.id().to_string();

                                    allowed.push((
                                        id.clone(),
                                        radio
                                            .slice(AppChannel::Channels, move |state| {
                                                state.channels.get(&id).unwrap()
                                            })
                                            .into_readable(),
                                    ));
                                }
                            }
                        }

                        allowed_channels.set(allowed);
                    }
                });
            }
        });

        use_side_effect({
            let user = user.clone();

            move || {
                spawn({
                    let servers = servers.clone();
                    let user = user.clone();

                    async move {
                        let mut allowed = Vec::new();

                        let servers = servers.read();
                        let query = user_permissions_query(radio).user(user.read().cloned());

                        for server in servers.values() {
                            let permissions = calculate_server_permissions(
                                &mut query.clone().server(server.clone()),
                            )
                            .await;

                            if permissions.has_channel_permission(ChannelPermission::ManageServer) {
                                let id = server.id.clone();

                                allowed.push((
                                    id.clone(),
                                    radio
                                        .slice(AppChannel::Servers, move |state| {
                                            state.servers.get(&id).unwrap()
                                        })
                                        .into_readable(),
                                ));
                            }
                        }

                        allowed_servers.set(allowed);
                    }
                });
            }
        });

        let filtered = use_side_effect_value({
            move || {
                let mut filtered = Vec::new();
                let query = query.read().to_lowercase();

                let mut destinations = Vec::new();

                if include_servers() {
                    destinations.extend(
                        allowed_servers
                            .read()
                            .iter()
                            .cloned()
                            .map(InviteDestination::Server),
                    );
                };

                if include_groups() {
                    destinations.extend(
                        allowed_channels
                            .read()
                            .iter()
                            .cloned()
                            .map(InviteDestination::Group),
                    );
                }

                for destination in destinations {
                    if query.is_empty()
                        || match &destination {
                            InviteDestination::Group((_, channel)) => channel
                                .read()
                                .name()
                                .unwrap()
                                .to_lowercase()
                                .starts_with(&query),
                            InviteDestination::Server((_, server)) => {
                                server.read().name.to_lowercase().starts_with(&query)
                            }
                        }
                    {
                        filtered.push(destination);
                    };
                }

                filtered
            }
        });

        Dialog::new()
            .body(
                rect()
                    .spacing(8.)
                    .cross_align(Alignment::Center)
                    .maybe_child((!self.bot.avatar.is_empty()).then(|| {
                        file_image(&v0::File {
                            id: self.bot.avatar.clone(),
                            tag: "avatars".to_string(),
                            filename: String::new(),
                            metadata: v0::Metadata::File,
                            content_type: String::new(),
                            size: 0,
                            deleted: None,
                            reported: None,
                            message_id: None,
                            user_id: None,
                            server_id: None,
                            object_id: None,
                        })
                        .size(Size::px(64.))
                    }))
                    .child(
                        label()
                            .font_weight(FontWeight::SEMI_BOLD)
                            .text(self.bot.username.clone()),
                    )
                    .child(
                        label()
                            .font_size(12.)
                            .text(format!("Registered since {created_at}")),
                    )
                    .child(SingleLineEntry::new("Search for group or server...", query))
                    .child(
                        rect()
                            .horizontal()
                            .content(Content::Flex)
                            .spacing(16.)
                            .child(rect().width(Size::flex(1.)).child(
                                StoatCheckbox::new(include_servers).child("Include Servers"),
                            ))
                            .child(
                                rect().width(Size::flex(1.)).child(
                                    StoatCheckbox::new(include_groups).child("Include Groups"),
                                ),
                            ),
                    )
                    .child(ScrollView::new().max_height(Size::px(240.)).children(
                        filtered.read().iter().map({
                            let current_selection = selected.read();

                            move |destination| {
                                let is_selected = current_selection
                                    .as_ref()
                                    .is_some_and(|dest| dest == destination);

                                let (icon, name) = match destination {
                                    InviteDestination::Server((_, server)) => (
                                        rect()
                                            .corner_radius(12.)
                                            .overflow(Overflow::Clip)
                                            .child(ServerIcon::from_readable(server.clone(), 24.))
                                            .into_element(),
                                        server.read().name.clone(),
                                    ),
                                    InviteDestination::Group((_, channel)) => (
                                        GroupDMIcon::new(channel.clone())
                                            .size(24.)
                                            .invert(true)
                                            .into_element(),
                                        channel.read().name().unwrap().to_string(),
                                    ),
                                };

                                StoatButton::new()
                                    .corner_radius(8.)
                                    .on_press({
                                        let destination = destination.clone();
                                        move |_| {
                                            selected.set(Some(destination.clone()));
                                        }
                                    })
                                    .child(
                                        rect()
                                            .horizontal()
                                            .padding(8.)
                                            .spacing(8.)
                                            .cross_align(Alignment::Center)
                                            .maybe(is_selected, |this| {
                                                this.background(theme.md.primary.as_argb_u32())
                                                    .color(theme.md.on_primary.as_argb_u32())
                                            })
                                            .width(Size::Fill)
                                            .child(icon)
                                            .child(name),
                                    )
                            }
                        }),
                    ))
                    .child(
                        label()
                            .font_size(11.)
                            .text("Bots are not verified by Stoat."),
                    )
                    .child(
                        label()
                            .font_size(11.)
                            .text("The bot will not be granted any permissions."),
                    ),
            )
            .default_action("Cancel")
            .action("Add", {
                let bot = self.bot.clone();

                move || {
                    let Some(selection) = selected.read().cloned() else {
                        return false;
                    };

                    spawn({
                        let bot = bot.clone();

                        async move {
                            if http()
                                .invite_bot(
                                    &bot.id,
                                    &match selection {
                                        InviteDestination::Group((group, _)) => {
                                            v0::InviteBotDestination::Group { group }
                                        }
                                        InviteDestination::Server((server, _)) => {
                                            v0::InviteBotDestination::Server { server }
                                        }
                                    },
                                )
                                .await
                                .is_ok()
                            {
                                modals.write().pop_modal();
                            }
                        }
                    });

                    false
                }
            })
    }
}
