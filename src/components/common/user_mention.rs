use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Avatar, StoatButton, StoatButtonLayoutThemePartialExt, UserCard, use_floating},
    consume_material_theme, member_display_color,
};

#[derive(PartialEq)]
pub struct UserMention {
    pub user_id: String,
    pub server: Option<Readable<v0::Server>>,
    pub text_style: TextStyleData,
}

impl UserMention {
    pub fn new(user_id: String, server: Option<Readable<v0::Server>>) -> Self {
        Self {
            user_id,
            server,
            text_style: TextStyleData::default(),
        }
    }
}

impl TextStyleExt for UserMention {
    fn get_text_style_data(&mut self) -> &mut TextStyleData {
        &mut self.text_style
    }
}

impl Component for UserMention {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut floating = use_floating();

        let radio = use_radio(AppChannel::Users);
        let members = radio.slice(AppChannel::Members, |state| &state.members);

        let state = radio.read();
        let user = state.users.get(&self.user_id);

        let member = use_memo({
            let id = self.user_id.clone();
            let server = self.server.clone();
            let members = members.clone();

            move || {
                if let Some(server) = &server {
                    let server = server.read();
                    let members = members.read();

                    if let Some(members) = members.get(&server.id)
                        && let Some(member) = members.get(&id)
                    {
                        return Some(member.clone());
                    }
                };

                None
            }
        });

        let role_color = use_memo({
            let server = self.server.clone();

            move || {
                if let Some(server) = &server
                    && let Some(member) = &*member.read()
                {
                    return member_display_color(&*member, &*server.read());
                };

                None
            }
        });

        let username = member
            .read()
            .as_ref()
            .and_then(|m| m.nickname.clone())
            .or(user.map(|u| u.display_name.clone().unwrap_or(u.username.clone())));

        let font_size = self.text_style.font_size.map_or(14, |size| *size);
        let size = (font_size as f32) * (16. / 14.);

        StoatButton::new()
            .corner_radius(16.)
            .on_press({
                let user = user.cloned();
                let member = member.clone();
                move |_| {
                    if let Some(user) = user.clone() {
                        floating.set(Some(
                            UserCard {
                                user: user.into_readable(),
                                member: member.read().cloned().map(|m| m.into_readable()),
                            }
                            .into_element(),
                        ));
                    };
                }
            })
            .child(
                rect()
                    .padding((0., 6., 0., 2.))
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .spacing(4.)
                    .background(theme.md.primary_container.as_argb_u32())
                    .color(theme.md.on_primary_container.as_argb_u32())
                    .font_weight(FontWeight::SEMI_BOLD)
                    .maybe_child(user.map(|user| {
                        Avatar::new(
                            user.clone().into_readable(),
                            member.read().cloned().map(|m| m.into_readable()),
                            size,
                        )
                    }))
                    .child(
                        label()
                            .map(role_color.read().cloned(), |label, color| {
                                label.color(color)
                            })
                            .text_height(TextHeightBehavior::All)
                            .line_height(1.5)
                            .max_lines(1)
                            .font_size(font_size)
                            .text(username.unwrap_or_else(|| "Unknown User".to_string())),
                    ),
            )
    }
}
