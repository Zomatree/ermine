use std::collections::HashSet;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, Initial, SizeExt,
    components::{Dialog, checkbox::StoatCheckbox},
    consume_material_theme, http, parse_fill, use_initial,
};

#[derive(PartialEq)]
pub struct EditRoles {
    pub user: String,
    pub server: String,
}

impl Component for EditRoles {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);
        let server = radio.slice_current({
            let server = self.server.clone();
            move |state| state.servers.get(&server).unwrap()
        });

        let member = radio.slice(AppChannel::Members, {
            let user = self.user.clone();
            let server = self.server.clone();
            move |state| state.members.get(&server).unwrap().get(&user).unwrap()
        });

        let user = radio.slice(AppChannel::Users, {
            let user = self.user.clone();
            move |state| state.users.get(&user).unwrap()
        });

        let mut roles = use_initial(|| {
            member
                .read()
                .roles
                .clone()
                .into_iter()
                .collect::<HashSet<_>>()
        });

        let ordered_roles = use_memo({
            move || {
                let mut roles = server
                    .read()
                    .roles
                    .values()
                    .cloned()
                    .map(|role| {
                        let color = role.colour.as_deref().and_then(parse_fill);
                        (role, color)
                    })
                    .collect::<Vec<_>>();

                roles.sort_by(|(a, _), (b, _)| a.rank.cmp(&b.rank));
                roles
            }
        });

        Dialog::new()
            .title(format!("Edit {}'s roles", user.read().username.clone()))
            .body(
                rect()
                    .spacing(8.)
                    .children(ordered_roles.read().iter().cloned().map(|(role, colour)| {
                        RoleToggle {
                            selected: roles.read().contains(&role.id),
                            roles,
                            role,
                            colour,
                        }
                        .into_element()
                    })),
            )
            .default_action("Close")
            .action("Reset", move || {
                roles.reset();
                false
            })
            .action("Save", {
                let user = self.user.clone();
                let server = self.server.clone();

                move || {
                    let user = user.clone();
                    let server = server.clone();

                    spawn(async move {
                        if let Ok(member) = http()
                            .edit_member(
                                &server,
                                &user,
                                &v0::DataMemberEdit {
                                    nickname: None,
                                    pronouns: None,
                                    avatar: None,
                                    roles: Some(roles.read().cloned().into_iter().collect()),
                                    timeout: None,
                                    can_publish: None,
                                    can_receive: None,
                                    voice_channel: None,
                                    remove: Vec::new(),
                                },
                            )
                            .await
                        {
                            roles.set_new(member.roles.into_iter().collect::<HashSet<_>>());
                        }
                    });
                    false
                }
            })
    }
}

#[derive(PartialEq)]
struct RoleToggle {
    roles: Initial<HashSet<String>>,
    role: v0::Role,
    colour: Option<Fill>,
    selected: bool,
}

impl Component for RoleToggle {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let active = use_reactive(&self.selected);

        use_side_effect({
            let mut roles = self.roles;
            let id = self.role.id.clone();
            move || {
                if active() {
                    roles.write().insert(id.clone());
                } else {
                    roles.write().remove(&id);
                }
            }
        });

        StoatCheckbox::new(active).child(
            rect()
                .horizontal()
                .width(Size::Fill)
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(self.role.name.clone())
                .child(
                    rect().size(Size::px(16.)).corner_radius(8.).background(
                        self.colour
                            .clone()
                            .unwrap_or_else(|| theme.md.outline_variant.as_argb_u32().into()),
                    ),
                ),
        )
    }
}
