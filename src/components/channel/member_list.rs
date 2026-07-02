use std::{borrow::Cow, collections::HashMap};

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{
        Avatar, StoatButton, StoatButtonLayoutThemePartialExt, UserCard, image, use_floating,
    },
    http, member_display_color,
};

#[derive(Clone)]
enum ListValue {
    Name(String, Option<v0::File>, usize),
    Member(Readable<v0::User>, Readable<v0::Member>),
}

impl PartialEq for ListValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Name(name0, icon0, length0), Self::Name(name1, icon1, length1)) => {
                name0 == name1 && icon0 == icon1 && length0 == length1
            }
            (Self::Member(user0, _), Self::Member(user1, _)) => user0.peek().id == user1.peek().id,
            _ => false,
        }
    }
}

#[derive(PartialEq)]
pub struct MemberList {
    pub server: Readable<v0::Server>,
}

impl Component for MemberList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Members);

        let slice = radio.slice_current({
            let server = self.server.clone();
            move |state| state.members.get(&server.peek().id).unwrap()
        });

        use_future({
            let radio = radio.clone();
            let server = self.server.clone();

            move || {
                let mut radio = radio.clone();
                let server = server.clone();

                async move {
                    let server_id = server.peek().id.clone();

                    let response = http()
                        .fetch_server_members(
                            &server_id,
                            &v0::OptionsFetchAllMembers {
                                exclude_offline: Some(true),
                            },
                        )
                        .await
                        .unwrap();

                    let mut state = radio.write_channel(AppChannel::Users);

                    for user in response.users {
                        state.users.insert(user.id.clone(), user);
                    }

                    drop(state);

                    let mut state = radio.write_channel(AppChannel::Members);
                    let server_members = state.members.get_mut(&server_id).unwrap();

                    for member in response.members {
                        server_members.insert(member.id.user.clone(), member);
                    }
                }
            }
        });

        let hoisted_roles = use_memo({
            let server = self.server.clone();
            move || {
                let mut roles = server
                    .read()
                    .roles
                    .values()
                    .filter(|role| role.hoist)
                    .cloned()
                    .collect::<Vec<_>>();
                roles.sort_by(|a, b| a.rank.cmp(&b.rank));
                roles
            }
        });

        let groups = use_memo({
            let slice = slice.clone();

            move || {
                let members = slice.read();
                let roles = hoisted_roles.read();

                let mut groups = HashMap::new();
                groups.insert("default".to_string(), Vec::new());

                for role in roles.iter() {
                    groups.insert(role.id.clone(), Vec::new());
                }

                'a: for member in members.values() {
                    let user_slice = radio.slice(AppChannel::Users, {
                        let user_id = member.id.user.clone();
                        move |state| state.users.get(&user_id).unwrap()
                    });
                    let user = user_slice.read();

                    if !user.online {
                        continue;
                    };

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

                    groups
                        .get_mut("default")
                        .unwrap()
                        .push(member.id.user.clone());
                }

                let mut out = Vec::new();

                for role in roles.iter() {
                    let members = groups.remove(&role.id).unwrap();

                    if !members.is_empty() {
                        out.push((role.name.clone(), role.icon.clone(), members));
                    };
                }

                let default = groups.remove("default").unwrap();

                if !members.is_empty() {
                    out.push(("Online".to_string(), None, default));
                };

                out
            }
        });

        let role_members = use_memo({
            let server_id = self.server.peek().id.clone();

            move || {
                let mut out = Vec::new();

                for (role, icon, user_ids) in groups.read().iter() {
                    let mut users = user_ids
                        .iter()
                        .cloned()
                        .map(|user_id| {
                            let user = radio.slice(AppChannel::Users, {
                                let user_id = user_id.clone();
                                move |state| state.users.get(&user_id).unwrap()
                            });

                            let member = radio.slice(AppChannel::Members, {
                                let server_id = server_id.clone();
                                move |state| {
                                    state
                                        .members
                                        .get(&server_id)
                                        .unwrap()
                                        .get(&user_id)
                                        .unwrap()
                                }
                            });

                            (user.into_readable(), member.into_readable())
                        })
                        .collect::<Vec<(Readable<v0::User>, Readable<v0::Member>)>>();

                    users.sort_by(|(user1, member1), (user2, member2)| {
                        let user1 = user1.read();
                        let member1 = member1.read();

                        let user2 = user2.read();
                        let member2 = member2.read();

                        let a = member1
                            .nickname
                            .as_ref()
                            .or(user1.display_name.as_ref())
                            .unwrap_or(&user1.username)
                            .to_ascii_lowercase();

                        let b = member2
                            .nickname
                            .as_ref()
                            .or(user2.display_name.as_ref())
                            .unwrap_or(&user2.username)
                            .to_ascii_lowercase();

                        a.cmp(&b)

                        // match (member1.nickname.as_ref(), member2.nickname.as_ref()) {
                        //     (Some(nick1), Some(nick2)) => nick1.cmp(nick2),
                        //     (Some(nick1), None) => {
                        //         let user2 = user2.read();

                        //         nick1.cmp(user2.display_name.as_ref().unwrap_or(&user2.username))
                        //     }
                        //     (None, Some(nick2)) => {
                        //         let user1 = user1.read();

                        //         user1
                        //             .display_name
                        //             .as_ref()
                        //             .unwrap_or(&user1.username)
                        //             .cmp(nick2)
                        //     }
                        //     (None, None) => {
                        //         let user1 = user1.read();
                        //         let user2 = user2.read();

                        //         user1
                        //             .display_name
                        //             .as_ref()
                        //             .unwrap_or(&user1.username)
                        //             .cmp(user2.display_name.as_ref().unwrap_or(&user2.username))
                        //     }
                        // }
                    });

                    out.push((role.clone(), icon.clone(), users));
                }

                out
            }
        });

        let elements = use_memo({
            let role_members = role_members.clone();
            // let groups = groups.clone();
            // let radio = radio.clone();
            // let server_id = self.server.peek().id.clone();

            move || {
                let mut elements = Vec::new();

                for (title, icon, members) in role_members.read().iter() {
                    elements.push(ListValue::Name(title.clone(), icon.clone(), members.len()));

                    for (user, member) in members.clone() {
                        elements.push(ListValue::Member(user, member));
                    }
                }

                elements
            }
        });

        rect().child(
            VirtualScrollView::new({
                let elements = elements.clone();
                let server = self.server.clone();

                move |i, _| {
                    let element = elements.read()[i].clone();

                    match element {
                        ListValue::Name(name, icon, count) => rect()
                            .key(&name)
                            .horizontal()
                            .height(Size::px(42.))
                            .cross_align(Alignment::End)
                            .spacing(6.)
                            .padding((0., 14.))
                            .main_align(Alignment::End)
                            .maybe_child(icon.map(|icon| {
                                image(&icon).width(Size::px(16.)).height(Size::px(16.))
                            }))
                            .child(label().text(format!("{name} - {count}")).font_size(11.))
                            .into_element(),
                        ListValue::Member(user, member) => MemberListMember {
                            server: server.clone(),
                            member,
                            user,
                        }
                        .into_element(),
                    }
                }
            })
            .item_size(42.)
            .length(elements.read().len()),
        )
    }
}

#[derive(PartialEq)]
pub struct MemberListMember {
    pub server: Readable<v0::Server>,
    pub member: Readable<v0::Member>,
    pub user: Readable<v0::User>,
}

impl Component for MemberListMember {
    fn render(&self) -> impl IntoElement {
        let floating = use_floating();

        let role_color = use_memo({
            let server = self.server.clone();
            let member = self.member.clone();

            move || member_display_color(&member.read(), &server.read())
        });

        let display_name = use_memo({
            let user = self.user.clone();
            let member = self.member.clone();

            move || {
                member.read().nickname.clone().unwrap_or_else(|| {
                    let user = user.read();

                    user.display_name.as_ref().unwrap_or(&user.username).clone()
                })
            }
        });

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

                        move |_| {
                            floating.clone().set(Some(
                                UserCard {
                                    user: user.clone(),
                                    member: Some(member.clone()),
                                }
                                .into_element(),
                            ));
                        }
                    })
                    .child(
                        rect()
                            .padding((0., 8.))
                            .horizontal()
                            .expanded()
                            .cross_align(Alignment::Center)
                            .spacing(8.)
                            .on_secondary_down({
                                let user = self.user.clone();

                                move |e| {
                                    ContextMenu::open_from_event(
                                        &e,
                                        Menu::new().child(
                                            MenuButton::new()
                                                .child(label().font_size(14.).text("Copy User ID"))
                                                .on_press({
                                                    let user = user.clone();

                                                    move |_| {
                                                        Clipboard::set(user.read().id.clone())
                                                            .unwrap();
                                                    }
                                                }),
                                        ),
                                    );
                                }
                            })
                            .child(
                                Avatar::new(self.user.clone(), Some(self.member.clone()), 32.)
                                    .presence(true),
                            )
                            .child(
                                rect()
                                    .child(
                                        label()
                                            .text(display_name.read().clone())
                                            .map(role_color.read().clone(), |mut this, color| {
                                                this.get_text_style_data().color = Some(color);
                                                this
                                            })
                                            .font_size(14)
                                            .max_lines(1)
                                            .text_overflow(TextOverflow::Ellipsis),
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
                                                label()
                                                    .text(text)
                                                    .font_size(11)
                                                    .max_lines(1)
                                                    .text_overflow(TextOverflow::Ellipsis)
                                            }),
                                    ),
                            ),
                    ),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.member.peek().id).into()
    }
}
