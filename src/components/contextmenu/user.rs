use freya::{
    prelude::*,
    radio::use_radio,
    text_edit::{TextEditor, TextSelection, UseEditable},
};

use crate::{
    AppChannel,
    components::{
        ContextMenuButton,
        material::outlined::{account_circle, alternate_email, badge},
    },
};

#[derive(PartialEq)]
pub struct UserContextMenu {
    pub user_id: String,
    pub server_id: Option<String>,
}

impl Component for UserContextMenu {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Users);

        // let user = radio.slice_current({
        //     let user_id = self.user_id.clone();
        //     move |state| state.users.get(&user_id).unwrap()
        // });

        let mut user_profile =
            radio.slice_mut(AppChannel::UserProfile, |state| &mut state.user_profile);

        let editable = use_consume::<Option<UseEditable>>();

        rect()
            .content(Content::Fit)
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
            .child(ContextMenuButton::new(badge(), "Copy User ID").on_press({
                let user_id = self.user_id.clone();

                move |_| {
                    Clipboard::set(user_id.clone()).unwrap();
                }
            }))
    }
}
