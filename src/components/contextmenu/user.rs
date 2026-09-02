use freya::{
    prelude::*,
    radio::{use_radio, use_radio_station},
    text_edit::{TextEditor, TextSelection, UseEditable},
};
use stoat_models::v0;

use crate::{
    AppChannel, AppState, Selection,
    components::{
        ContextMenuButton, ModalValue,
        material::outlined::{account_circle, alternate_email, assignment, badge, message},
        use_modals,
    },
    http, insert_channel,
};

#[derive(PartialEq)]
pub struct UserContextMenu {
    pub user_id: String,
    pub server_id: Option<String>,
}

impl Component for UserContextMenu {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();
        let station = use_radio_station::<AppState, AppChannel>();
        let radio = use_radio(AppChannel::Users);

        let user = radio.slice_current({
            let user_id = self.user_id.clone();
            move |state| state.users.get(&user_id).unwrap()
        });

        let mut user_profile =
            radio.slice_mut(AppChannel::UserProfile, |state| &mut state.user_profile);
        let selection = radio.slice_mut(AppChannel::Selection, |state| &mut state.selection);
        let selected_channel = radio.slice_mut(AppChannel::SelectedChannel, |state| {
            &mut state.selected_channel
        });

        let editable = consume_root_context::<Option<UseEditable>>();

        let user = user.read();

        rect()
            .content(Content::Fit)
            .maybe_child(
                (user.relationship == v0::RelationshipStatus::Friend || user.bot.is_some()).then(
                    || {
                        ContextMenuButton::new(message(), "Message").on_press({
                            let user_id = self.user_id.clone();

                            move |_| {
                                let user_id = user_id.clone();
                                let mut selection = selection.clone();
                                let mut selected_channel = selected_channel.clone();

                                spawn_forever(async move {
                                    if let Ok(channel) = http().open_dm(&user_id).await {
                                        selection.set(Selection::Home);
                                        selected_channel
                                            .set(Some((channel.id().to_string(), None)));

                                        insert_channel(channel, station);
                                    }
                                });
                            }
                        })
                    },
                ),
            )
            .child(
                ContextMenuButton::new(account_circle(), "Profile").on_press({
                    let user_id = self.user_id.clone();

                    move |_| {
                        *user_profile.write() = Some(user_id.clone());
                    }
                }),
            )
            .maybe_child(editable.map(|mut editable| {
                ContextMenuButton::new(alternate_email(), "Mention").on_press({
                    let user_id = self.user_id.clone();

                    move |_| {
                        let mut editor = editable.editor_mut().write();

                        let pos = editor.cursor_pos();
                        editor.insert(&format!("<@{user_id}>"), pos);
                        *editor.selection_mut() = TextSelection::new_cursor(editor.len_chars())
                    }
                })
            }))
            .maybe_child(self.server_id.clone().map(|server| {
                ContextMenuButton::new(assignment(), "Edit Roles").on_press({
                    let user = self.user_id.clone();

                    move |_| {
                        modals.write().push_modal(ModalValue::EditRoles {
                            user: user.clone(),
                            server: server.clone(),
                        });
                    }
                })
            }))
            .child(ContextMenuButton::new(badge(), "Copy User ID").on_press({
                let user_id = self.user_id.clone();

                move |_| {
                    Clipboard::set(user_id.clone()).unwrap();
                }
            }))
    }
}
