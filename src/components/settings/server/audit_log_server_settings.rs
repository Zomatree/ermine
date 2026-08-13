use std::{collections::HashMap, ops::Not, time::SystemTime};

use crate::{
    AppChannel, SizeExt,
    components::{
        Avatar, Dropdown, MaterialIcon, SingleLineEntry, StoatButton,
        StoatButtonLayoutThemePartialExt,
        material::{
            filled::{
                add_link, flag, format_list_bulleted, group, info, insert_emoticon, link, list,
                message, pin_invoke, smart_toy,
            },
            outlined::{chevron_right, expand_more},
        },
    },
    consume_material_theme, http,
};
use freya::{prelude::*, radio::use_radio};
use jiff::{Timestamp, tz::TimeZone};
use stoat_models::v0;

#[derive(PartialEq)]
pub struct AuditLogServerSettings {
    pub server: Readable<v0::Server>,
}

impl Component for AuditLogServerSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let action_filter = use_state(|| None::<&'static str>);
        let user_filter = use_state(String::new);
        let target_filter = use_state(String::new);

        let mut users = use_state(|| HashMap::<String, v0::User>::new());
        let mut members = use_state(|| HashMap::<String, v0::Member>::new());
        let mut audit_logs = use_state(|| Vec::<v0::AuditLogEntry>::new());

        let mut stop_fetching = use_state(|| false);
        let mut end_visible = use_state(|| true);
        let expanded = use_state(|| None);

        let fetch_audit_logs = {
            let server_id = self.server.read().id.clone();

            move || {
                let server_id = server_id.clone();
                let action = action_filter.read().cloned().map(|s| s.to_string());
                let user = user_filter.read().cloned();
                let target = target_filter.read().cloned();
                let last_entry = audit_logs
                    .peek()
                    .last()
                    .map(|audit_log| audit_log.id.clone());

                spawn(async move {
                    stop_fetching.set(false);
                    end_visible.set(true);

                    if let Ok(response) = http()
                        .fetch_audit_logs(
                            &server_id,
                            &v0::OptionsAuditLogQuery {
                                user: if !user.len() == 26 { Some(user) } else { None },
                                target: if !target.len() == 26 {
                                    Some(target)
                                } else {
                                    None
                                },
                                r#type: if let Some(action) = action {
                                    Some(vec![action])
                                } else {
                                    None
                                },
                                before: last_entry,
                                after: None,
                                limit: Some(50),
                            },
                        )
                        .await
                    {
                        let mut users = users.write();
                        for user in response.users {
                            users.insert(user.id.clone(), user);
                        }
                        let mut members = members.write();
                        for member in response.members {
                            members.insert(member.id.user.clone(), member);
                        }

                        if response.audit_logs.len() < 50 {
                            stop_fetching.set(true);
                        }

                        audit_logs.write().extend(response.audit_logs);
                    }
                });
            }
        };

        use_side_effect({
            let fetch_audit_logs = fetch_audit_logs.clone();

            move || {
                audit_logs.write().clear();
                fetch_audit_logs()
            }
        });

        let entries = audit_logs.read();

        rect()
            .spacing(8.)
            .child(
                rect()
                    .horizontal()
                    .spacing(4.)
                    .content(Content::Flex)
                    .child(SingleLineEntry::new("Author", user_filter).width(Size::flex(1.)))
                    .child(SingleLineEntry::new("Target", target_filter).width(Size::flex(1.)))
                    .child(
                        Dropdown::new(
                            "Action Type",
                            action_filter.into_writable(),
                            vec![
                                None,
                                Some("MessageDelete"),
                                Some("MessageBulkDelete"),
                                Some("MessagePin"),
                                Some("BanCreate"),
                                Some("BanDelete"),
                                Some("ChannelCreate"),
                                Some("ChannelEdit"),
                                Some("ChannelRolePermissionsEdit"),
                                Some("ChannelDelete"),
                                Some("MemberEdit"),
                                Some("MemberKick"),
                                Some("ServerEdit"),
                                Some("RoleEdit"),
                                Some("RoleCreate"),
                                Some("RoleDelete"),
                                Some("RolesReorder"),
                                Some("InviteCreate"),
                                Some("InviteDelete"),
                                Some("WebhookCreate"),
                                Some("WebhookDelete"),
                                Some("EmojiCreate"),
                                Some("EmojiUpdate"),
                                Some("EmojiDelete"),
                            ],
                            move |action| {
                                label()
                                    .text(match action {
                                        None => "Empty",
                                        Some("MessageDelete") => "Message Delete",
                                        Some("MessageBulkDelete") => "Message Bulk Delete",
                                        Some("MessagePin") => "Message Pin",
                                        Some("BanCreate") => "Ban Create",
                                        Some("BanDelete") => "Ban Delete",
                                        Some("ChannelCreate") => "Channel Create",
                                        Some("ChannelEdit") => "Channel Edit",
                                        Some("ChannelRolePermissionsEdit") => {
                                            "Channel Role Permissions Edit"
                                        }
                                        Some("ChannelDelete") => "Channel Delete",
                                        Some("MemberEdit") => "Member Edit",
                                        Some("MemberKick") => "Member Kick",
                                        Some("ServerEdit") => "Server Edit",
                                        Some("RoleEdit") => "Role Edit",
                                        Some("RoleCreate") => "Role Create",
                                        Some("RoleDelete") => "Role Delete",
                                        Some("RolesReorder") => "Roles Reorder",
                                        Some("InviteCreate") => "Invite Create",
                                        Some("InviteDelete") => "Invite Delete",
                                        Some("WebhookCreate") => "Webhook Create",
                                        Some("WebhookDelete") => "Webhook Delete",
                                        Some("EmojiCreate") => "Emoji Create",
                                        Some("EmojiUpdate") => "Emoji Update",
                                        Some("EmojiDelete") => "Emoji Delete",
                                        v => unreachable!("{v:?}"),
                                    })
                                    .color(
                                        if action == &None {
                                            theme.md.on_surface_variant
                                        } else {
                                            theme.md.on_surface
                                        }
                                        .as_argb_u32(),
                                    )
                                    .into_element()
                            },
                        )
                        .width(Size::flex(1.)),
                    ),
            )
            .child(rect().spacing(4.).children({
                let users = users.into_readable();
                let members = members.into_readable();

                entries.iter().cloned().map(move |entry| {
                    AuditLogEntry {
                        entry,
                        expanded,
                        server: self.server.clone(),
                        users: users.clone(),
                        members: members.clone(),
                    }
                    .into_element()
                })
            }))
            .child(
                rect()
                    .width(Size::Fill)
                    .center()
                    .padding(8.)
                    .maybe_child(stop_fetching().not().then(|| {
                        rect().child(CircularLoader::new()).on_sized(
                            move |e: Event<SizedEventData>| {
                                if e.visible_area.origin.y
                                    <= Platform::get().root_size.read().height
                                {
                                    end_visible.set_if_modified_and_then(true, &fetch_audit_logs);
                                } else {
                                    end_visible.set(false);
                                }
                            },
                        )
                    }))
                    .maybe_child((stop_fetching() && entries.is_empty()).then(|| {
                        label()
                            .color(theme.md.on_surface_variant.as_argb_u32())
                            .font_size(12.)
                            .text("No Audit Logs.")
                            .into_element()
                    })),
            )
    }
}

#[derive(Debug, Copy, Clone)]
enum IconStyle {
    Create,
    Modify,
    Delete,
}

#[derive(PartialEq)]
struct AuditLogEntry {
    pub entry: v0::AuditLogEntry,
    pub expanded: State<Option<String>>,
    pub server: Readable<v0::Server>,
    pub users: Readable<HashMap<String, v0::User>>,
    pub members: Readable<HashMap<String, v0::Member>>,
}

impl Component for AuditLogEntry {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let radio = use_radio(AppChannel::Channels);
        let channels = radio.slice_current(|state| &state.channels);

        let user = self.users.map(
            {
                let user_id = self.entry.user.clone();
                move |users| users.get(&user_id).unwrap()
            },
            {
                let user_id = self.entry.user.clone();
                move |user| &user.id == &user_id
            },
        );

        let datetime = use_hook(|| {
            Timestamp::try_from(ulid::Ulid::from_string(&self.entry.id).unwrap().datetime())
                .unwrap()
                .to_zoned(TimeZone::system())
        });

        let get_channel_name = |id| {
            channels
                .read()
                .get(id)
                .and_then(|channel| channel.name().map(|s| s.to_string()))
                .unwrap_or("Unknown Channel".to_string())
        };

        let get_user_name = |id| {
            self.users
                .read()
                .get(id)
                .map(|user| user.username.clone())
                .unwrap_or("Unknown User".to_string())
        };

        let get_role_name = |id| {
            self.server
                .read()
                .roles
                .get(id)
                .map(|role| role.name.clone())
                .unwrap_or("Unknown Role".to_string())
        };

        let username = user.read().username.clone();

        let is_expanded = self
            .expanded
            .read()
            .as_ref()
            .is_some_and(|id| id == &self.entry.id);

        let (icon, color, text) = match &self.entry.action {
            v0::AuditLogEntryAction::MessageDelete { author, channel } => (
                message(),
                IconStyle::Delete,
                format!(
                    "{} deleted a message by {} in #{}",
                    username,
                    get_user_name(author),
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::MessageBulkDelete { channel, count } => (
                list(),
                IconStyle::Delete,
                format!(
                    "{} deleted {} messages in #{}",
                    username,
                    count,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::MessagePin {
                message: _,
                author: _,
                channel,
            } => (
                pin_invoke(),
                IconStyle::Create,
                format!(
                    "{} pinned a message in #{}",
                    username,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::MessageUnpin {
                message: _,
                author: _,
                channel,
            } => (
                pin_invoke(),
                IconStyle::Delete,
                format!(
                    "{} unpinned a message in #{}",
                    username,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::BanCreate { user } => (
                group(),
                IconStyle::Delete,
                format!("{} banned {}", username, get_user_name(user)),
            ),
            v0::AuditLogEntryAction::BanDelete { user } => (
                group(),
                IconStyle::Create,
                format!("{} unbanned {}", username, get_user_name(user)),
            ),
            v0::AuditLogEntryAction::ChannelCreate { channel, name: _ } => (
                format_list_bulleted(),
                IconStyle::Create,
                format!("{} created #{}", username, get_channel_name(channel)),
            ),
            v0::AuditLogEntryAction::ChannelEdit {
                channel,
                before: _,
                after: _,
            } => (
                format_list_bulleted(),
                IconStyle::Modify,
                format!("{} updated #{}", username, get_channel_name(channel)),
            ),
            v0::AuditLogEntryAction::ChannelRolePermissionsEdit {
                channel,
                role,
                permissions: _,
            } => (
                format_list_bulleted(),
                IconStyle::Modify,
                format!(
                    "{} updated role @{} permissions in #{}",
                    username,
                    get_role_name(role),
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::ChannelDelete { channel, name: _ } => (
                format_list_bulleted(),
                IconStyle::Delete,
                format!("{} deleted #{}", username, get_channel_name(channel)),
            ),
            v0::AuditLogEntryAction::MemberEdit {
                user,
                before: _,
                after: _,
            } => (
                group(),
                IconStyle::Modify,
                format!("{} updated {}", username, get_user_name(user)),
            ),
            v0::AuditLogEntryAction::MemberKick { user } => (
                group(),
                IconStyle::Delete,
                format!("{} kicked {}", username, get_user_name(user)),
            ),
            v0::AuditLogEntryAction::ServerEdit {
                before: _,
                after: _,
            } => (
                info(),
                IconStyle::Modify,
                format!(
                    "{} made changes to {}",
                    username,
                    self.server.read().name.clone()
                ),
            ),
            v0::AuditLogEntryAction::RoleEdit {
                role,
                before: _,
                after: _,
            } => (
                info(),
                IconStyle::Modify,
                format!("{} updated role @{}", username, get_role_name(role)),
            ),
            v0::AuditLogEntryAction::RoleCreate { role, name: _ } => (
                flag(),
                IconStyle::Create,
                format!("{} created role @{}", username, get_role_name(role)),
            ),
            v0::AuditLogEntryAction::RoleDelete { role: _, name } => (
                flag(),
                IconStyle::Delete,
                format!("{} deleted role @{}", username, name),
            ),
            v0::AuditLogEntryAction::RolesReorder {
                before: _,
                after: _,
            } => (
                flag(),
                IconStyle::Modify,
                format!("{} re-oredered roles", username),
            ),
            v0::AuditLogEntryAction::InviteCreate { invite, channel } => (
                add_link(),
                IconStyle::Create,
                format!(
                    "{} created an invite {} in #{}",
                    username,
                    invite,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::InviteDelete { invite, channel } => (
                link(),
                IconStyle::Delete,
                format!(
                    "{} deleted an invite {} in #{}",
                    username,
                    invite,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::WebhookCreate {
                webhook: _,
                name,
                channel,
            } => (
                smart_toy(),
                IconStyle::Create,
                format!(
                    "{} created a webhook {} in #{}",
                    username,
                    name,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::WebhookDelete {
                webhook: _,
                name,
                channel,
            } => (
                smart_toy(),
                IconStyle::Delete,
                format!(
                    "{} deleted a webhook {} in #{}",
                    username,
                    name,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::EmojiCreate { emoji: _, name } => (
                smart_toy(),
                IconStyle::Create,
                format!("{} created emoji {}", username, name),
            ),
            v0::AuditLogEntryAction::EmojiUpdate {
                emoji: _,
                before: _,
                after: _,
            } => (
                insert_emoticon(),
                IconStyle::Modify,
                format!("{} updated an emoji", username),
            ),
            v0::AuditLogEntryAction::EmojiDelete { emoji: _, name } => (
                smart_toy(),
                IconStyle::Delete,
                format!("{} deleted emoji {}", username, name),
            ),
        };

        StoatButton::new()
            .corner_radius(12.)
            .on_press({
                let id = self.entry.id.clone();
                let mut expanded = self.expanded;
                move |_| {
                    let current = expanded.read().cloned();
                    if let Some(current) = current
                        && &current == &id
                    {
                        expanded.set(None);
                    } else {
                        expanded.set(Some(id.clone()));
                    }
                }
            })
            .child(
                rect()
                    .width(Size::Fill)
                    .background(theme.md.secondary_container.as_argb_u32())
                    .color(theme.md.on_secondary_container.as_argb_u32())
                    .padding(13.)
                    .spacing(16.)
                    .child(
                        rect()
                            .spacing(8.)
                            .horizontal()
                            .cross_align(Alignment::Center)
                            .content(Content::Flex)
                            .child(
                                MaterialIcon::new(icon)
                                    .size(Size::px(16.))
                                    .color(match color {
                                        IconStyle::Create => theme.stoat.presence_online,
                                        IconStyle::Modify => theme.stoat.presence_idle,
                                        IconStyle::Delete => theme.stoat.presence_busy,
                                    }),
                            )
                            .child(Avatar::new(user.clone(), None, 36.))
                            .child(
                                rect().width(Size::flex(1.)).child(text).child(
                                    label()
                                        .font_size(12.)
                                        .color(theme.md.on_surface_variant.as_argb_u32())
                                        .text(format!(
                                            "{:02}/{:02}/{}",
                                            datetime.day(),
                                            datetime.month(),
                                            datetime.year()
                                        )),
                                ),
                            )
                            .child(
                                MaterialIcon::new(if is_expanded {
                                    expand_more()
                                } else {
                                    chevron_right()
                                })
                                .size(Size::px(16.)),
                            ),
                    )
                    .maybe_child(is_expanded.then(|| {
                        let lines = match &self.entry.action {
                            v0::AuditLogEntryAction::MessageDelete { .. } => Vec::new(),
                            v0::AuditLogEntryAction::MessageBulkDelete { .. } => Vec::new(),
                            v0::AuditLogEntryAction::MessagePin { .. } => Vec::new(),
                            v0::AuditLogEntryAction::MessageUnpin {
                                message: _,
                                author: _,
                                channel: _,
                            } => Vec::new(),
                            v0::AuditLogEntryAction::BanCreate { .. } => Vec::new(),
                            v0::AuditLogEntryAction::BanDelete { .. } => Vec::new(),
                            v0::AuditLogEntryAction::ChannelCreate { .. } => Vec::new(),
                            v0::AuditLogEntryAction::ChannelEdit {
                                channel: _,
                                before,
                                after,
                            } => {
                                vec![
                                    ("Name", before.name.clone(), after.name.clone()),
                                    (
                                        "Description",
                                        before.description.clone(),
                                        after.description.clone(),
                                    ),
                                    (
                                        "Icon",
                                        before.icon.is_some().then(|| "TODO".to_string()),
                                        after.icon.is_some().then(|| "TODO".to_string()),
                                    ),
                                    (
                                        "Mature",
                                        before.nsfw.map(|nsfw| nsfw.to_string()),
                                        after.nsfw.map(|nsfw| nsfw.to_string()),
                                    ),
                                    (
                                        "Max users",
                                        before.voice.as_ref().and_then(|voice| {
                                            voice.max_users.map(|count| count.to_string())
                                        }),
                                        after.voice.as_ref().and_then(|voice| {
                                            voice.max_users.map(|count| count.to_string())
                                        }),
                                    ),
                                    (
                                        "Slowmode",
                                        before.slowmode.map(|secs| format!("{secs}s")),
                                        after.slowmode.map(|secs| format!("{secs}s")),
                                    ),
                                ]
                            }
                            v0::AuditLogEntryAction::ChannelRolePermissionsEdit { .. } => {
                                Vec::new()
                            } // TODO
                            v0::AuditLogEntryAction::ChannelDelete { .. } => Vec::new(),
                            v0::AuditLogEntryAction::MemberEdit {
                                user: _,
                                before,
                                after,
                            } => {
                                vec![
                                    ("Nickname", before.nickname.clone(), after.nickname.clone()),
                                    ("Pronouns", before.pronouns.clone(), after.pronouns.clone()),
                                    (
                                        "Avatar",
                                        before.avatar.is_some().then(|| "TODO".to_string()),
                                        after.avatar.is_some().then(|| "TODO".to_string()),
                                    ),
                                    (
                                        "Timeout",
                                        before.timeout.map(|ts| format!("until {}", format_ts(ts))),
                                        after.timeout.map(|ts| format!("until {}", format_ts(ts))),
                                    ),
                                    (
                                        "Muted",
                                        before
                                            .can_publish
                                            .map(|not_muted| (!not_muted).to_string()),
                                        after.can_publish.map(|not_muted| (!not_muted).to_string()),
                                    ),
                                    (
                                        "Deafened",
                                        before
                                            .can_receive
                                            .map(|not_deafend| (!not_deafend).to_string()),
                                        after
                                            .can_receive
                                            .map(|not_deafend| (!not_deafend).to_string()),
                                    ),
                                ]
                            }
                            v0::AuditLogEntryAction::MemberKick { .. } => Vec::new(),
                            v0::AuditLogEntryAction::ServerEdit { before, after } => {
                                vec![
                                    (
                                        "Owner",
                                        before.owner.as_ref().map(|id| get_user_name(&id)),
                                        after.owner.as_ref().map(|id| get_user_name(&id)),
                                    ),
                                    ("Name", before.name.clone(), after.name.clone()),
                                    (
                                        "Description",
                                        before.description.clone(),
                                        after.description.clone(),
                                    ),
                                    (
                                        "Categories",
                                        before.categories.is_some().then(|| "TODO".to_string()),
                                        after.categories.is_some().then(|| "TODO".to_string()),
                                    ),
                                    (
                                        "User joined channel",
                                        before.system_messages.as_ref().and_then(|system| {
                                            system
                                                .user_joined
                                                .as_ref()
                                                .map(|id| get_channel_name(id))
                                        }),
                                        after.system_messages.as_ref().and_then(|system| {
                                            system
                                                .user_joined
                                                .as_ref()
                                                .map(|id| get_channel_name(id))
                                        }),
                                    ),
                                    (
                                        "User left channel",
                                        before.system_messages.as_ref().and_then(|system| {
                                            system.user_left.as_ref().map(|id| get_channel_name(id))
                                        }),
                                        after.system_messages.as_ref().and_then(|system| {
                                            system.user_left.as_ref().map(|id| get_channel_name(id))
                                        }),
                                    ),
                                    (
                                        "User kicked channel",
                                        before.system_messages.as_ref().and_then(|system| {
                                            system
                                                .user_kicked
                                                .as_ref()
                                                .map(|id| get_channel_name(id))
                                        }),
                                        after.system_messages.as_ref().and_then(|system| {
                                            system
                                                .user_kicked
                                                .as_ref()
                                                .map(|id| get_channel_name(id))
                                        }),
                                    ),
                                    (
                                        "User banned channel",
                                        before.system_messages.as_ref().and_then(|system| {
                                            system
                                                .user_banned
                                                .as_ref()
                                                .map(|id| get_channel_name(id))
                                        }),
                                        after.system_messages.as_ref().and_then(|system| {
                                            system
                                                .user_banned
                                                .as_ref()
                                                .map(|id| get_channel_name(id))
                                        }),
                                    ),
                                ]
                            }
                            v0::AuditLogEntryAction::RoleEdit {
                                role: _,
                                before,
                                after,
                            } => vec![
                                ("Name", before.name.clone(), after.name.clone()),
                                (
                                    "Permissions",
                                    before.permissions.is_some().then(|| "TODO".to_string()),
                                    after.permissions.is_some().then(|| "TODO".to_string()),
                                ),
                                ("Colour", before.colour.clone(), after.colour.clone()),
                                (
                                    "Hoist",
                                    before.hoist.map(|hoist| hoist.to_string()),
                                    after.hoist.map(|hoist| hoist.to_string()),
                                ),
                                (
                                    "Icon",
                                    before.icon.is_some().then(|| "TODO".to_string()),
                                    after.icon.is_some().then(|| "TODO".to_string()),
                                ),
                            ],
                            v0::AuditLogEntryAction::RoleCreate { .. } => Vec::new(),
                            v0::AuditLogEntryAction::RoleDelete { .. } => Vec::new(),
                            v0::AuditLogEntryAction::RolesReorder { .. } => {
                                Vec::new() // TODO
                            }
                            v0::AuditLogEntryAction::InviteCreate { .. } => Vec::new(),
                            v0::AuditLogEntryAction::InviteDelete { .. } => Vec::new(),
                            v0::AuditLogEntryAction::WebhookCreate { .. } => Vec::new(),
                            v0::AuditLogEntryAction::WebhookDelete { .. } => Vec::new(),
                            v0::AuditLogEntryAction::EmojiCreate { .. } => Vec::new(),
                            v0::AuditLogEntryAction::EmojiUpdate {
                                emoji: _,
                                before,
                                after,
                            } => vec![("Name", before.name.clone(), after.name.clone())],
                            v0::AuditLogEntryAction::EmojiDelete { .. } => Vec::new(),
                        };

                        rect()
                            .padding((0., 0., 0., 24.))
                            .children(
                                lines
                                    .into_iter()
                                    .map(|(title, before, after)| match (before, after) {
                                        (None, Some(after)) => {
                                            Some(format!("{title}: Set to {after}"))
                                        }
                                        (Some(before), None) => {
                                            Some(format!("{title}: Removed {before}"))
                                        }
                                        (Some(before), Some(after)) => Some(format!(
                                            "{title}: Updated from {before} to {after}"
                                        )),
                                        (None, None) => None,
                                    })
                                    .flatten()
                                    .map(|s| s.into_element()),
                            )
                            .maybe_child(
                                self.entry
                                    .reason
                                    .clone()
                                    .map(|reason| format!("Reason: {reason}")),
                            )
                    })),
            )
    }
}

fn format_ts(ts: iso8601_timestamp::Timestamp) -> String {
    let ts = Timestamp::try_from(SystemTime::from(ts))
        .unwrap()
        .to_zoned(TimeZone::system());

    ts.strftime("%A, %B %d, %Y at %H:%M").to_string()
}
