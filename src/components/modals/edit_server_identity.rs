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
pub struct EditServerIdentity {
    pub user: String,
    pub server: String,
}

impl Component for EditServerIdentity {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();
        let radio = use_radio(AppChannel::Users);

        let user = radio.slice_current({
            let user_id = self.user.clone();
            move |state| state.users.get(&user_id).unwrap()
        });
        let member = radio.slice(AppChannel::Members, {
            let server = self.server.clone();
            let user_id = self.user.clone();

            move |state| {
                state
                    .members
                    .get(&server)
                    .unwrap()
                    .get(state.user_id.as_ref().unwrap())
                    .unwrap()
            }
        });

        let nickname = use_state(|| member.read().nickname.clone().unwrap_or_default());
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(1.5).text(format!(
                "Change {}'s nickname",
                user.read().username.clone()
            )))
            .body(
                rect()
                    .spacing(8.)
                    .child(SingleLineEntry::new("Nickname", nickname)),
            )
            .default_action("Cancel")
            .action("Save", {
                let server_id = self.server.clone();
                let user_id = self.user.clone();
                move || {
                    let server_id = server_id.clone();
                    let user_id = user_id.clone();
                    let nickname = nickname.read().clone();

                    spawn(async move {
                        match http()
                            .edit_member(
                                &server_id,
                                &user_id,
                                &v0::DataMemberEdit {
                                    remove: if nickname.is_empty() {
                                        vec![v0::FieldsMember::Nickname]
                                    } else {
                                        Vec::new()
                                    },
                                    nickname: if nickname.is_empty() {
                                        None
                                    } else {
                                        Some(nickname)
                                    },
                                    pronouns: None,
                                    avatar: None,
                                    roles: None,
                                    timeout: None,
                                    can_publish: None,
                                    can_receive: None,
                                    voice_channel: None,
                                },
                            )
                            .await
                        {
                            Ok(_) => {
                                modals.write().pop_modal();
                            }
                            Err(e) => {
                                error.set(Some(format_error(&e, "Member")));
                            }
                        }
                    });
                    false
                }
            })
    }
}
