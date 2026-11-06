use std::{borrow::Cow, collections::HashMap};

use freya::{
    prelude::*,
    radio::{use_radio, use_radio_station},
};
use stoat_models::v0;

use crate::{
    AppChannel, OptionalReadable, SizeExt, calculate_server_permissions,
    components::{
        Avatar, MarkdownViewer, MaterialIcon, StoatButton, StoatButtonLayoutThemePartialExt,
        StoatTooltip, UserCard, UserContextMenu, file_image, material::filled::smart_toy,
        stoat_label, stoat_scrollbar, use_floating,
    },
    consume_material_theme, http, insert_user, map_optional_readable, member_display_color,
    user_permissions_query,
};

#[derive(Clone)]
enum ListValue {
    Name(String, Option<v0::File>, usize),
    Member(Readable<v0::User>, OptionalReadable<v0::Member>, bool),
}

impl PartialEq for ListValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Name(name0, icon0, length0), Self::Name(name1, icon1, length1)) => {
                name0 == name1 && icon0 == icon1 && length0 == length1
            }
            (Self::Member(user0, _, _), Self::Member(user1, _, _)) => {
                user0.peek().id == user1.peek().id
            }
            _ => false,
        }
    }
}

#[derive(PartialEq)]
pub struct MemberList {
    pub server: Option<Readable<v0::Server>>,
    pub channel: Readable<v0::Channel>,
}

impl Component for MemberList {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Members);
        let station = use_radio_station();

        let slice = self.server.clone().map(|server| {
            radio.slice_current(move |state| state.members.get(&server.peek().id).unwrap())
        });

        let users = radio.slice(AppChannel::Users, |state| &state.users);
        let exclude_offline = use_hook(|| {
            self.server.as_ref()
                .is_some_and(|s| s.read().approximate_member_count > 1000)
        });

        use_side_effect({
            let radio = radio.clone();
            let server = self.server.clone();

            move || {
                if let Some(server) = server.clone() {
                    let mut radio = radio.clone();

                    spawn(async move {
                        let server_id = server.peek().id.clone();

                        let response = http()
                            .fetch_server_members(
                                &server_id,
                                &v0::OptionsFetchAllMembers {
                                    exclude_offline: Some(exclude_offline),
                                },
                            )
                            .await
                            .unwrap();

                        for user in response.users {
                            insert_user(user, station);
                        }

                        let mut state = radio.write_channel(AppChannel::Members);
                        let server_members = state.members.get_mut(&server_id).unwrap();

                        for member in response.members {
                            server_members.insert(member.id.user.clone(), member);
                        }
                    });
                }
            }
        });

        let hoisted_roles = use_memo({
            let server = self.server.clone();

            move || {
                if let Some(server) = &server {
                    let mut roles = server
                        .read()
                        .roles
                        .values()
                        .filter(|role| role.hoist)
                        .cloned()
                        .collect::<Vec<_>>();
                    roles.sort_by(|a, b| a.rank.cmp(&b.rank));
                    roles
                } else {
                    Vec::new()
                }
            }
        });

        let groups = use_memo({
            let slice = slice.clone();
            let users = users.clone();
            let channel = self.channel.clone();

            move || {
                let roles = hoisted_roles.read();
                let users = users.read();

                let mut groups = HashMap::new();
                groups.insert("default".to_string(), Vec::new());
                groups.insert("offline".to_string(), Vec::new());

                if let Some(slice) = &slice {
                    let members = slice.read();

                    for role in roles.iter() {
                        groups.insert(role.id.clone(), Vec::new());
                    }

                    'a: for member in members.values() {
                        if let Some(user) = users.get(&member.id.user) {
                            if user.online {
                                if !member.roles.is_empty() {
                                    for hoisted_role in roles.iter() {
                                        if member.roles.contains(&hoisted_role.id) {
                                            groups
                                                .get_mut(&hoisted_role.id)
                                                .unwrap()
                                                .push(member.id.user.clone());
                                            continue 'a;
                                        }
                                    }
                                };

                                groups.get_mut("default").unwrap().push(user.id.clone());
                            } else if !exclude_offline {
                                groups.get_mut("offline").unwrap().push(user.id.clone());
                            }
                        };
                    }
                } else {
                    let channel = channel.read();

                    if let v0::Channel::DirectMessage { recipients, .. }
                    | v0::Channel::Group { recipients, .. } = &*channel
                    {
                        for user_id in recipients {
                            if let Some(user) = users.get(user_id) {
                                if user.online {
                                    groups.get_mut("default").unwrap().push(user_id.clone());
                                } else {
                                    groups.get_mut("offline").unwrap().push(user_id.clone());
                                }
                            }
                        }
                    }
                }

                let mut out = Vec::new();

                for role in roles.iter() {
                    let members = groups.remove(&role.id).unwrap();

                    if !members.is_empty() {
                        out.push((role.name.clone(), role.icon.clone(), members, true));
                    };
                }

                let default = groups.remove("default").unwrap();
                let offline = groups.remove("offline").unwrap();

                if !default.is_empty() {
                    out.push(("Online".to_string(), None, default, true));
                };

                if !offline.is_empty() {
                    out.push(("Offline".to_string(), None, offline, false));
                };

                out
            }
        });

        let role_members = use_memo({
            let members: Option<Readable<HashMap<String, v0::Member>>> =
                slice.clone().map(|s| s.into_readable());

            move || {
                let mut out = Vec::new();

                for (role, icon, user_ids, is_online) in groups.read().iter() {
                    let mut users = user_ids
                        .iter()
                        .cloned()
                        .map(|user_id| {
                            let user = radio.slice(AppChannel::Users, {
                                let user_id = user_id.clone();
                                move |state| state.users.get(&user_id).unwrap()
                            });

                            let member = if let Some(members) = &members {
                                map_optional_readable(members.clone(), move |members| {
                                    members.get(&user_id)
                                })
                            } else {
                                OptionalReadable::none()
                            };

                            (user.into_readable(), member)
                        })
                        .collect::<Vec<(Readable<v0::User>, OptionalReadable<v0::Member>)>>();

                    users.sort_by(|(user1, member1), (user2, member2)| {
                        let user1 = user1.read();
                        let member1 = member1.read();

                        let user2 = user2.read();
                        let member2 = member2.read();

                        let a = member1
                            .as_ref()
                            .and_then(|m| m.nickname.as_ref())
                            .or(user1.display_name.as_ref())
                            .unwrap_or(&user1.username)
                            .to_ascii_lowercase();

                        let b = member2
                            .as_ref()
                            .and_then(|m| m.nickname.as_ref())
                            .or(user2.display_name.as_ref())
                            .unwrap_or(&user2.username)
                            .to_ascii_lowercase();

                        a.cmp(&b)
                    });

                    out.push((role.clone(), icon.clone(), *is_online, users));
                }

                out
            }
        });

        let online_count = use_memo({
            let role_members = role_members.clone();

            move || {
                if exclude_offline {
                    return None;
                };

                let count = role_members
                    .read()
                    .iter()
                    .filter(|(_, _, online, _)| *online)
                    .map(|(_, _, _, members)| members.len())
                    .sum::<usize>();

                Some(count)
            }
        });

        let elements = use_memo({
            let role_members = role_members.clone();

            move || {
                let mut elements = Vec::new();

                for (title, icon, is_online, members) in role_members.read().iter() {
                    elements.push(ListValue::Name(title.clone(), icon.clone(), members.len()));

                    for (user, member) in members.clone() {
                        elements.push(ListValue::Member(user, member, *is_online));
                    }
                }

                elements
            }
        });

        rect().child(
            VirtualScrollView::new({
                let elements = elements.clone();
                let server = self.server.clone();
                let channel = self.channel.clone();

                move |item, _| {
                    let element = elements.read()[item.index].clone();

                    match element {
                        ListValue::Name(name, icon, count) => {
                            let container = rect()
                                .padding((0., 14.))
                                .height(Size::px(42.))
                                .font_size(11.);

                            let title = rect()
                                .horizontal()
                                .spacing(6.)
                                .maybe_child(icon.map(|icon| {
                                    file_image(&icon)
                                        .size(Size::px(16.))
                                        .image_cover(ImageCover::Center)
                                        .aspect_ratio(AspectRatio::Max)
                                        .corner_radius(8.)
                                }))
                                .child(
                                    stoat_label()
                                        .font_weight(500)
                                        .text(format!("{name} - {count}")),
                                );

                            if item.index == 0
                                && let Some(online_count) = online_count()
                            {
                                container
                                    .main_align(Alignment::SpaceBetween)
                                    .child(
                                        rect()
                                            .horizontal()
                                            .spacing(8.)
                                            .cross_align(Alignment::Center)
                                            .child(
                                                rect()
                                                    .size(Size::px(8.))
                                                    .corner_radius(4.)
                                                    .background(theme.stoat.presence_online),
                                            )
                                            .child(
                                                stoat_label()
                                                    .font_weight(500)
                                                    .text(format!("{online_count} members online")),
                                            ),
                                    )
                                    .child(title)
                            } else {
                                container.main_align(Alignment::End).child(title)
                            }
                            .into_element()
                        }
                        ListValue::Member(user, member, is_online) => MemberListMember {
                            server: server.clone(),
                            channel: channel.clone(),
                            member,
                            user,
                            is_online,
                        }
                        .into_element(),
                    }
                }
            })
            .scrollbar(stoat_scrollbar)
            .item_size(42.)
            .length(elements.read().len()),
        )
    }
}

#[derive(PartialEq)]
pub struct MemberListMember {
    pub server: Option<Readable<v0::Server>>,
    pub channel: Readable<v0::Channel>,
    pub member: OptionalReadable<v0::Member>,
    pub user: Readable<v0::User>,
    pub is_online: bool,
}

impl Component for MemberListMember {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let floating = use_floating();

        let radio = use_radio(AppChannel::Users);

        let role_color = use_memo({
            let server = self.server.clone();
            let member = self.member.clone();

            move || {
                if let Some(member) = member.read()
                    && let Some(server) = &server
                {
                    member_display_color(&member, &server.read())
                } else {
                    None
                }
            }
        });

        let display_name = use_memo({
            let user = self.user.clone();
            let member = self.member.clone();

            move || {
                member
                    .read()
                    .and_then(|m| m.nickname.clone())
                    .unwrap_or_else(|| {
                        let user = user.read();

                        user.display_name.as_ref().unwrap_or(&user.username).clone()
                    })
            }
        });

        let is_bot = use_hook(|| self.user.read().bot.is_some());

        rect()
            .padding((0., 8.))
            .height(Size::px(42.))
            .width(Size::Fill)
            .child(
                StoatButton::new()
                    .corner_radius(28.)
                    .on_press({
                        let user = self.user.clone();
                        let member = self.member.clone();
                        let server = self.server.clone();
                        let channel = self.channel.clone();

                        move |_| {
                            let user = user.clone();
                            let member = member.read().map(|m| m.clone().into_readable());
                            let server = server.as_ref().map(|s| s.read().clone());
                            let channel = channel.read().clone();

                            spawn(async move {
                                let mut query = user_permissions_query(radio.clone())
                                    .server(server)
                                    .channel(channel);
                                let permissions = calculate_server_permissions(&mut query).await;

                                floating.clone().set(Some(
                                    UserCard {
                                        user,
                                        member,
                                        permissions,
                                    }
                                    .into_element(),
                                ))
                            });
                        }
                    })
                    .child(
                        rect()
                            .opacity(if self.is_online { 1. } else { 0.4 })
                            .padding((0., 8.))
                            .horizontal()
                            .expanded()
                            .cross_align(Alignment::Center)
                            .spacing(8.)
                            .on_secondary_down({
                                let user = self.user.clone();
                                let server = self.server.clone();
                                let channel = self.channel.clone();
                                let radio = radio.clone();

                                move |_| {
                                    let user = user.read().clone();
                                    let server = server.as_ref().map(|s| s.read().clone());
                                    let channel = channel.read().clone();
                                    let radio = radio.clone();

                                    spawn(async move {
                                        let mut query = user_permissions_query(radio)
                                            .user(user.clone())
                                            .server(server.clone()).channel(channel);
                                        let permissions =
                                            calculate_server_permissions(&mut query).await;

                                        ContextMenu::open(Menu::new().key(&user.id).child(
                                            UserContextMenu {
                                                user_id: user.id.clone(),
                                                server_id: server.map(|s| s.id),
                                                permissions,
                                            },
                                        ));
                                    });
                                }
                            })
                            .child(
                                Avatar::new(
                                    self.user.clone(),
                                    self.member.read().map(|m| m.clone().into_readable()),
                                    32.,
                                )
                                .presence(self.is_online),
                            )
                            .child(
                                rect()
                                    .child(
                                        rect()
                                            .horizontal()
                                            .spacing(4.)
                                            .cross_align(Alignment::Center)
                                            .child(
                                                stoat_label()
                                                    .text(display_name.read().clone())
                                                    .map(
                                                        role_color.read().clone(),
                                                        |this, color| this.color(color),
                                                    )
                                                    .font_size(14)
                                                    .max_lines(1)
                                                    .font_weight(500)
                                                    .line_height(1.25)
                                                    .text_overflow(TextOverflow::Ellipsis),
                                            )
                                            .maybe_child(is_bot.then(|| {
                                                MaterialIcon::new(smart_toy())
                                                    .color(theme.md.outline.as_argb_u32())
                                                    .size(Size::px(12.))
                                            })),
                                    )
                                    .maybe_child(
                                        self.user
                                            .read()
                                            .status
                                            .as_ref()
                                            .and_then(|status| {
                                                status
                                                    .text
                                                    .as_ref()
                                                    .map(|text| Cow::Owned(text.clone()))
                                                    .or(status.presence.as_ref().map(|presence| {
                                                        match presence {
                                                            v0::Presence::Online => {
                                                                Cow::Borrowed("Online")
                                                            }
                                                            v0::Presence::Idle => {
                                                                Cow::Borrowed("Idle")
                                                            }
                                                            v0::Presence::Focus => {
                                                                Cow::Borrowed("Focus")
                                                            }
                                                            v0::Presence::Busy => {
                                                                Cow::Borrowed("Busy")
                                                            }
                                                            v0::Presence::Invisible => {
                                                                Cow::Borrowed("Invisible")
                                                            }
                                                        }
                                                    }))
                                            })
                                            .map(|text| {
                                                StoatTooltip::new(
                                                    label()
                                                        .font_size(11)
                                                        .max_lines(1)
                                                        .text(text.clone()),
                                                )
                                                .position(AttachedPosition::Top)
                                                .child(
                                                    MarkdownViewer::new(
                                                        text,
                                                        self.server.clone(),
                                                    )
                                                    .font_size(11)
                                                    .max_lines(1)
                                                    .text_overflow(TextOverflow::Ellipsis),
                                                )
                                            }),
                                    ),
                            ),
                    ),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.user.peek().id).into()
    }
}
