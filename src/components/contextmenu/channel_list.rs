use freya::prelude::*;
use stoat_permissions::{ChannelPermission, PermissionValue};

use crate::components::{
    ContextMenuButton, ModalValue,
    material::outlined::{add_circle_outline, badge, delete, edit},
    use_modals,
};

#[derive(PartialEq)]
pub struct ChannelListContextMenu {
    pub server_id: String,
    pub current_permissions: PermissionValue,
}

impl Component for ChannelListContextMenu {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        rect()
            .content(Content::Fit)
            .maybe_child(
                self.current_permissions
                    .has_channel_permission(ChannelPermission::ManageChannel)
                    .then(|| {
                        ContextMenuButton::new(add_circle_outline(), "Create Channel").on_press({
                            let server_id = self.server_id.clone();

                            move |_| {
                                modals.write().push_modal(ModalValue::CreateChannel {
                                    server: server_id.clone(),
                                });
                            }
                        })
                    }),
            )
            .maybe_child(
                self.current_permissions
                    .has_channel_permission(ChannelPermission::ManageChannel)
                    .then(|| {
                        ContextMenuButton::new(add_circle_outline(), "Create Category")
                            .on_press({
                            let server_id = self.server_id.clone();

                            move |_| {
                                modals.write().push_modal(ModalValue::CreateCategory {
                                    server: server_id.clone(),
                                });
                            }
                            })
                    }),
            )
    }
}
