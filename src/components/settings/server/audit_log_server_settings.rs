use std::{collections::HashMap, ops::Not};

use crate::{
    AppChannel, SizeExt,
    components::{
        Avatar, MaterialIcon,
        material::filled::{
            flag, format_list_bulleted, group, info, insert_emoticon, list, message, pin_invoke,
            smart_toy,
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
        let action_filter = use_state(|| Vec::<String>::new());
        let user_filter = use_state(|| None);
        let target_filter = use_state(|| None);

        let mut users = use_state(|| HashMap::<String, v0::User>::new());
        let mut members = use_state(|| HashMap::<String, v0::Member>::new());
        let mut audit_logs = use_state(|| Vec::<v0::AuditLogEntry>::new());

        let mut stop_fetching = use_state(|| false);

        let mut end_visible = use_state(|| false);

        let fetch_audit_logs = {
            let server_id = self.server.read().id.clone();

            move || {
                let server_id = server_id.clone();
                let actions = action_filter.read().cloned();
                let user = user_filter.read().cloned();
                let target = target_filter.read().cloned();
                let last_entry = audit_logs
                    .peek()
                    .last()
                    .map(|audit_log| audit_log.id.clone());

                spawn(async move {
                    if let Ok(response) = http()
                        .fetch_audit_logs(
                            &server_id,
                            &v0::OptionsAuditLogQuery {
                                user,
                                target,
                                r#type: if !actions.is_empty() {
                                    Some(actions)
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

        rect()
            .child(rect().spacing(4.).children({
                let users = users.into_readable();
                let members = members.into_readable();

                audit_logs.read().cloned().into_iter().map(move |entry| {
                    AuditLogEntry {
                        entry,
                        server: self.server.clone(),
                        users: users.clone(),
                        members: members.clone(),
                    }
                    .into_element()
                })
            }))
            .maybe_child(stop_fetching().not().then(
                || {
                    rect()
                        .width(Size::Fill)
                        .center()
                        .padding(8.)
                        .child(CircularLoader::new())
                        .on_sized(move |e: Event<SizedEventData>| {
                            if e.visible_area.is_empty() {
                                end_visible.set_if_modified_and_then(true, &fetch_audit_logs);
                            } else {
                                end_visible.set_if_modified(false);
                            }
                        })
                }, // StoatButton::new()
                   //     .child("Load More")
                   //     .on_press(move |_| fetch_audit_logs()),
            ))
    }
}

#[derive(Debug, Copy, Clone)]
enum IconColor {
    Green,
    Yellow,
    Red,
}

#[derive(PartialEq)]
struct AuditLogEntry {
    pub entry: v0::AuditLogEntry,
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

        let (icon, color, text) = match &self.entry.action {
            v0::AuditLogEntryAction::MessageDelete { author, channel } => (
                message(),
                IconColor::Red,
                format!(
                    "{} deleted a message by {} in #{}",
                    username,
                    get_user_name(author),
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::MessageBulkDelete { channel, count } => (
                list(),
                IconColor::Red,
                format!(
                    "{} deleted {} messages in #{}",
                    username,
                    count,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::MessagePin {
                message,
                author,
                channel,
            } => (
                pin_invoke(),
                IconColor::Green,
                format!(
                    "{} pinned a message in #{}",
                    username,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::MessageUnpin {
                message,
                author,
                channel,
            } => (
                pin_invoke(),
                IconColor::Red,
                format!(
                    "{} unpinned a message in #{}",
                    username,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::BanCreate { user } => (
                group(),
                IconColor::Red,
                format!("{} banned {}", username, get_user_name(user)),
            ),
            v0::AuditLogEntryAction::BanDelete { user } => (
                group(),
                IconColor::Green,
                format!("{} unbanned {}", username, get_user_name(user)),
            ),
            v0::AuditLogEntryAction::ChannelCreate { channel, name } => (
                format_list_bulleted(),
                IconColor::Green,
                format!("{} created #{}", username, get_channel_name(channel)),
            ),
            v0::AuditLogEntryAction::ChannelEdit {
                channel,
                before,
                after,
            } => (
                format_list_bulleted(),
                IconColor::Yellow,
                format!("{} updated #{}", username, get_channel_name(channel)),
            ),
            v0::AuditLogEntryAction::ChannelRolePermissionsEdit {
                channel,
                role,
                permissions,
            } => (
                format_list_bulleted(),
                IconColor::Yellow,
                format!(
                    "{} updated role @{} permissions in #{}",
                    username,
                    get_role_name(role),
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::ChannelDelete { channel, name } => (
                format_list_bulleted(),
                IconColor::Red,
                format!("{} deleted #{}", username, get_channel_name(channel)),
            ),
            v0::AuditLogEntryAction::MemberEdit {
                user,
                before,
                after,
            } => (
                group(),
                IconColor::Yellow,
                format!("{} updated {}", username, get_user_name(user)),
            ),
            v0::AuditLogEntryAction::MemberKick { user } => (
                group(),
                IconColor::Red,
                format!("{} kicked {}", username, get_user_name(user)),
            ),
            v0::AuditLogEntryAction::ServerEdit { before, after } => (
                info(),
                IconColor::Yellow,
                format!(
                    "{} made changes to {}",
                    username,
                    self.server.read().name.clone()
                ),
            ),
            v0::AuditLogEntryAction::RoleEdit {
                role,
                before,
                after,
            } => (
                info(),
                IconColor::Yellow,
                format!("{} updated role @{}", username, get_role_name(role)),
            ),
            v0::AuditLogEntryAction::RoleCreate { role, name } => (
                flag(),
                IconColor::Green,
                format!("{} created role @{}", username, get_role_name(role)),
            ),
            v0::AuditLogEntryAction::RoleDelete { role, name } => (
                flag(),
                IconColor::Red,
                format!("{} deleted role @{}", username, name),
            ),
            v0::AuditLogEntryAction::RolesReorder { before, after } => (
                flag(),
                IconColor::Yellow,
                format!("{} re-oredered roles", username),
            ),
            v0::AuditLogEntryAction::InviteCreate { invite, channel } => (
                flag(),
                IconColor::Green,
                format!(
                    "{} created an invite {} in #{}",
                    username,
                    invite,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::InviteDelete { invite, channel } => (
                flag(),
                IconColor::Red,
                format!(
                    "{} deleted an invite {} in #{}",
                    username,
                    invite,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::WebhookCreate {
                webhook,
                name,
                channel,
            } => (
                smart_toy(),
                IconColor::Green,
                format!(
                    "{} created a webhook {} in #{}",
                    username,
                    name,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::WebhookDelete {
                webhook,
                name,
                channel,
            } => (
                smart_toy(),
                IconColor::Red,
                format!(
                    "{} deleted a webhook {} in #{}",
                    username,
                    name,
                    get_channel_name(channel)
                ),
            ),
            v0::AuditLogEntryAction::EmojiCreate { emoji, name } => (
                smart_toy(),
                IconColor::Green,
                format!("{} created emoji {}", username, name),
            ),
            v0::AuditLogEntryAction::EmojiUpdate {
                emoji,
                before,
                after,
            } => (
                insert_emoticon(),
                IconColor::Yellow,
                format!("{} updated an emoji", username),
            ),
            v0::AuditLogEntryAction::EmojiDelete { emoji, name } => (
                smart_toy(),
                IconColor::Red,
                format!("{} deleted emoji {}", username, name),
            ),
            _ => (info(), IconColor::Green, "todo".to_string()),
        };

        rect()
            .width(Size::Fill)
            .background(theme.md.secondary_container.as_argb_u32())
            .color(theme.md.on_secondary_container.as_argb_u32())
            .corner_radius(12.)
            .padding(13.)
            .spacing(8.)
            .horizontal()
            .cross_align(Alignment::Center)
            .child(
                MaterialIcon::new(icon)
                    .size(Size::px(16.))
                    .color(match color {
                        IconColor::Green => Color::GREEN,
                        IconColor::Yellow => Color::YELLOW,
                        IconColor::Red => theme.md.error.as_argb_u32().into(),
                    }),
            )
            .child(Avatar::new(user.clone(), None, 36.))
            .child(
                rect()
                    .child(text)
                    .child(label().font_size(12.).text(format!(
                        "{:02}/{:02}/{}",
                        datetime.day(),
                        datetime.month(),
                        datetime.year()
                    ))),
            )
    }
}
