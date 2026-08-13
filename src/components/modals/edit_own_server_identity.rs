use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, SizeExt, Tag,
    components::{
        Dialog, MaterialIcon, SingleLineEntry, StoatButton, StoatButtonLayoutThemePartialExt,
        file_image, material::outlined::clear, use_modals,
    },
    consume_material_theme, format_error, http, prompt_image_upload, use_initial,
};

#[derive(PartialEq)]
pub struct EditOwnServerIdentity {
    pub server: String,
}

impl Component for EditOwnServerIdentity {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();
        let radio = use_radio(AppChannel::Servers);

        let server = radio.slice_current({
            let server = self.server.clone();
            move |state| state.servers.get(&server).unwrap()
        });
        let member = radio.slice(AppChannel::Members, {
            let server = self.server.clone();
            move |state| {
                state
                    .members
                    .get(&server)
                    .unwrap()
                    .get(state.user_id.as_ref().unwrap())
                    .unwrap()
            }
        });

        let member_value = member.read();

        let avatar = use_initial(|| member_value.avatar.clone());
        let nickname = use_initial(|| member_value.nickname.clone().unwrap_or_default());
        let pronouns = use_initial(|| member_value.pronouns.clone().unwrap_or_default());

        let mut error = use_state(|| None);

        let edit_member = {
            let server_id = self.server.clone();
            let user_id = radio.peek_state().user_id.clone().unwrap();

            move |payload| {
                let server_id = server_id.clone();
                let user_id = user_id.clone();

                async move {
                    match http().edit_member(&server_id, &user_id, &payload).await {
                        Ok(member) => Some(member),
                        Err(e) => {
                            error.set(Some(format_error(&e, "Member")));
                            None
                        }
                    }
                }
            }
        };

        let remove_field = {
            let edit_member = edit_member.clone();

            move |field| {
                let edit_member = edit_member.clone();

                async move {
                    edit_member(v0::DataMemberEdit {
                        nickname: None,
                        pronouns: None,
                        avatar: None,
                        roles: None,
                        timeout: None,
                        can_publish: None,
                        can_receive: None,
                        voice_channel: None,
                        remove: vec![field],
                    })
                    .await
                }
            }
        };

        Dialog::new()
            .title(
                label()
                    .line_height(1.5)
                    .text(format!("Change identity on {}", server.read().name.clone())),
            )
            .body(
                rect()
                    .spacing(8.)
                    .child(label().font_size(12.).text("Server Avatar"))
                    .child(
                        rect()
                            .horizontal()
                            .spacing(15.)
                            .cross_align(Alignment::Center)
                            .child(
                                StoatButton::new()
                                    .corner_radius(48.)
                                    .on_press({
                                        let edit_member = edit_member.clone();

                                        move |_| {
                                            let edit_member = edit_member.clone();

                                            spawn(async move {
                                                if let Some(id) =
                                                    prompt_image_upload(Tag::Avatars).await
                                                {
                                                    edit_member(v0::DataMemberEdit {
                                                        nickname: None,
                                                        pronouns: None,
                                                        avatar: Some(id),
                                                        roles: None,
                                                        timeout: None,
                                                        can_publish: None,
                                                        can_receive: None,
                                                        voice_channel: None,
                                                        remove: Vec::new(),
                                                    })
                                                    .await;
                                                };
                                            });
                                        }
                                    })
                                    .child(
                                        rect()
                                            .width(Size::px(96.))
                                            .height(Size::px(96.))
                                            .background(theme.md.surface_dim.as_argb_u32())
                                            .maybe_child(avatar.read().as_ref().map(|icon| {
                                                rect()
                                                    .layer(Layer::Relative(1))
                                                    .width(Size::Fill)
                                                    .height(Size::Fill)
                                                    .child(file_image(icon))
                                            })),
                                    ),
                            )
                            .child(
                                StoatButton::new()
                                    .corner_radius(16.)
                                    .on_press({
                                        let remove_field = remove_field.clone();

                                        move |_| {
                                            let remove_field = remove_field.clone();

                                            spawn(async move {
                                                remove_field(v0::FieldsMember::Avatar).await;
                                            });
                                        }
                                    })
                                    .child(
                                        rect()
                                            .width(Size::px(36.))
                                            .height(Size::px(36.))
                                            .center()
                                            .child(
                                                MaterialIcon::new(clear())
                                                    .size(Size::px(24.))
                                                    .color(theme.md.primary.as_argb_u32()),
                                            ),
                                    ),
                            ),
                    )
                    .child(SingleLineEntry::new("Nickname", nickname))
                    .child(SingleLineEntry::new("Pronouns", pronouns)),
            )
            .default_action("Cancel")
            .action("Save", move || {
                let edit_member = edit_member.clone();

                spawn(async move {
                    let mut payload = v0::DataMemberEdit {
                        nickname: None,
                        pronouns: None,
                        avatar: None,
                        roles: None,
                        timeout: None,
                        can_publish: None,
                        can_receive: None,
                        voice_channel: None,
                        remove: Vec::new(),
                    };

                    if let Some(nickname) = nickname.get_if_different() {
                        if nickname.is_empty() {
                            payload.remove.push(v0::FieldsMember::Nickname);
                        } else {
                            payload.nickname = Some(nickname)
                        }
                    };

                    if let Some(pronouns) = pronouns.get_if_different() {
                        if pronouns.is_empty() {
                            payload.remove.push(v0::FieldsMember::Pronouns);
                        } else {
                            payload.pronouns = Some(pronouns)
                        }
                    };

                    if edit_member(payload).await.is_some() {
                        modals.write().pop_modal();
                    };
                });
                false
            })
    }
}
