use std::collections::HashMap;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, OptionalReadable, components::Avatar, consume_material_theme, map_optional_readable,
};

#[derive(PartialEq)]
pub struct UserMention {
    pub user_id: String,
    pub server_id: Option<String>,
}

impl UserMention {
    pub fn new(user_id: String, server_id: Option<String>) -> Self {
        Self { user_id, server_id }
    }
}

impl Component for UserMention {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Users);

        let user = use_hook(|| {
            let users = radio.slice_current(|state| &state.users);

            let users: Readable<HashMap<String, v0::User>> = users.into_readable();

            map_optional_readable(users, {
                let user_id = self.user_id.clone();
                move |users| users.get(&user_id)
            })
        });

        let member = use_hook(|| {
            if let Some(server_id) = self.server_id.clone() {
                let members = radio.slice(AppChannel::Members, move |state| {
                    state.members.get(&server_id).unwrap()
                });

                let members: Readable<HashMap<String, v0::Member>> = members.into_readable();

                map_optional_readable(members, {
                    let user_id = self.user_id.clone();
                    move |members| members.get(&user_id)
                })
            } else {
                OptionalReadable::new(Box::new(|| None), Box::new(|| None))
            }
        });

        let user = user.read();
        let member = member.read();

        let name = member
            .as_ref()
            .and_then(|member| member.nickname.clone())
            .or_else(|| {
                user.as_ref()
                    .map(|user| user.display_name.clone().unwrap_or(user.username.clone()))
            })
            .unwrap_or_else(|| "Unknown User".to_string());

        rect()
            .padding((0., 6., 0., 2.))
            .horizontal()
            .cross_align(Alignment::Center)
            .spacing(4.)
            .corner_radius(16.)
            .background(theme.md.primary_container.as_argb_u32())
            .color(theme.md.on_primary_container.as_argb_u32())
            .font_weight(FontWeight::SEMI_BOLD)
            .maybe_child(
                user.as_ref().map(|user| {
                    Avatar::new(
                        (*user).clone().into_readable(),
                        member.as_ref().map(|member| (*member).clone().into_readable()),
                        16.,
                    )
                })
            )
            .child(
                label()
                    .line_height(1.5)
                    .max_lines(1)
                    .font_size(14.)
                    .text(name.to_string()),
            )
    }
}
