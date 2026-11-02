use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel,
    components::{
        ContextMenuButton, ContextMenuDivider, ModalValue,
        material::outlined::{delete, drive_file_rename_outline},
        use_modals,
    },
};

#[derive(PartialEq)]
pub struct ServerFolderMenu {
    pub folder_id: String,
}

impl Component for ServerFolderMenu {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        let radio = use_radio(AppChannel::Settings("ordering"));
        let mut ordering = radio.slice_mut_current(|state| {
            state
                .settings
                .ordering
                .get_or_insert_default()
                .server_sidebar
                .get_or_insert_default()
        });
        let mut folders = radio.slice_mut(AppChannel::Settings("server-folders"), |state| {
            &mut state
                .settings
                .server_folders
                .get_or_insert_default()
                .folders
        });

        rect()
            .content(Content::Fit)
            .child(
                ContextMenuButton::new(drive_file_rename_outline(), "Edit Folder").on_press({
                    let id = self.folder_id.clone();
                    move |_| {
                        modals
                            .write()
                            .push_modal(ModalValue::EditServerFolder { folder: id.clone() });
                    }
                }),
            )
            .child(ContextMenuDivider)
            .child(
                ContextMenuButton::new(delete(), "Delete Folder")
                    .danger()
                    .on_press({
                        let id = self.folder_id.clone();
                        move |_| {
                            folders.write().retain(|f| &f.id != &id);
                            ordering.write().retain(|e| e != &id);
                        }
                    }),
            )
    }
}
