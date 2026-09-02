use std::collections::HashMap;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{AppChannel, components::Avatar, map_optional_readable, map_readable};

#[derive(PartialEq)]
pub struct ChannelTyping {
    pub channel: Readable<v0::Channel>,
    pub server: Option<Readable<v0::Server>>,
}

impl Component for ChannelTyping {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Typing);
        let typing = radio.slice_current(|state| &state.typing);
        let user_id = radio.peek_state().user_id.clone().unwrap();
        let users: Readable<HashMap<String, v0::User>> = radio
            .slice(AppChannel::Users, |state| &state.users)
            .into_readable();

        let channel = self.channel.read();

        let users_typing = typing
            .read()
            .get(channel.id())
            .filter(|set| !set.is_empty())
            .cloned();

        let members: Readable<HashMap<String, HashMap<String, v0::Member>>> = radio
            .slice(AppChannel::Members, |state| &state.members)
            .into_readable();

        rect()
            .height(Size::px(26.))
            .horizontal()
            .cross_align(Alignment::Center)
            .spacing(8.)
            .map(users_typing, |this, typing| {
                let server_id = self.server.as_ref().map(|server| server.read().id.clone());

                let users_typing = typing
                    .iter()
                    .filter(|&id| id != &user_id && users.read().contains_key(id))
                    .cloned()
                    .map(|id| {
                        let user = map_readable(users.clone(), {
                            let id = id.clone();
                            move |users| users.get(&id).unwrap()
                        });

                        let member = server_id.clone().map(|server_id| {
                            map_optional_readable(members.clone(), move |members| {
                                members.get(&server_id).unwrap().get(&id)
                            })
                        });

                        (user, member)
                    })
                    .collect::<Vec<_>>();

                let names = users_typing
                    .iter()
                    .map(|(u, m)| {
                        if let Some(m) = m
                            && let Some(member) = m.read()
                        {
                            if let Some(nickname) = member.nickname.clone() {
                                return nickname;
                            }
                        }

                        let user = u.read();
                        user.display_name.clone().unwrap_or(user.username.clone())
                    })
                    .collect::<Vec<_>>();

                let names = if names.is_empty() {
                    String::new()
                } else if names.len() == 1 {
                    format!("{} is typing...", names[0])
                } else {
                    let (last, names) = names.split_last().unwrap();

                    format!("{} and {} are typing...", names.join(", "), last)
                };

                this.child(rect().padding((0., 0., 0., 6.)).horizontal().children(
                    users_typing.iter().cloned().map(|(user, member)| {
                        rect()
                            .margin((0., 0., 0., -6.))
                            .child(Avatar::new(
                                user.clone(),
                                member.and_then(|m| m.read().map(|m| m.clone().into_readable())),
                                15.,
                            ))
                            .into_element()
                    }),
                ))
                .child(label().font_size(12).text(names))
            })
    }
}
