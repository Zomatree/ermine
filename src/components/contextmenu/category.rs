use freya::prelude::*;
use stoat_permissions::{ChannelPermission, PermissionValue};

use crate::components::{
    ContextMenuButton, ModalValue,
    material::outlined::{badge, delete, edit},
    use_modals,
};

#[derive(PartialEq)]
pub struct CategoryContextMenu {
    pub server_id: String,
    pub category_id: String,
    pub current_permissions: PermissionValue,
}

impl Component for CategoryContextMenu {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        rect()
            .content(Content::Fit)
            .maybe_child(
                self.current_permissions
                    .has_channel_permission(ChannelPermission::ManageChannel)
                    .then(|| {
                        ContextMenuButton::new(edit(), "Rename Category").on_press({
                            let server_id = self.server_id.clone();
                            let category_id = self.category_id.clone();

                            move |_| {
                                modals.write().push_modal(ModalValue::RenameCategory {
                                    server: server_id.clone(),
                                    category: category_id.clone(),
                                });
                            }
                        })
                    }),
            )
            .maybe_child(
                self.current_permissions
                    .has_channel_permission(ChannelPermission::ManageChannel)
                    .then(|| {
                        ContextMenuButton::new(delete(), "Delete Category")
                            .danger()
                            .on_press({
                                let server_id = self.server_id.clone();
                                let category_id = self.category_id.clone();

                                move |_| {
                                    modals.write().push_modal(ModalValue::DeleteCategory {
                                        server: server_id.clone(),
                                        category: category_id.clone(),
                                    });
                                }
                            })
                    }),
            )
            .child(
                ContextMenuButton::new(badge(), "Copy Category ID").on_press({
                    let category_id = self.category_id.clone();

                    move |_| {
                        Clipboard::set(category_id.clone()).unwrap();
                    }
                }),
            )
    }
}
