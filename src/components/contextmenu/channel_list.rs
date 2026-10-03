use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;
use stoat_permissions::{ChannelPermission, PermissionValue};

use crate::{AppChannel, components::{
    ContextMenuButton, ModalValue,
    material::outlined::{add_circle_outline, badge, delete, edit},
    use_modals,
}};

#[derive(PartialEq)]
pub struct ChannelListContextMenu {
    pub server_id: String,
    pub current_permissions: PermissionValue,
}

impl Component for ChannelListContextMenu {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();
        let radio = use_radio(AppChannel::SelectedChannel);
        let selected_channel = radio.slice_mut_current(|state| &mut state.selected_channel);

        rect()
            .content(Content::Fit)
            .maybe_child(
                self.current_permissions
                    .has_channel_permission(ChannelPermission::ManageChannel)
                    .then(|| {
                        ContextMenuButton::new(add_circle_outline(), "Create Channel").on_press({
                            let server_id = self.server_id.clone();

                            move |_| {
                                let mut selected_channel = selected_channel.clone();

                                modals.write().push_modal(ModalValue::CreateChannel {
                                    server: server_id.clone(),
                                    callback: EventHandler::new(move |channel: v0::Channel| {
                                        selected_channel.set(Some((channel.id().to_string(), None)));
                                    })
                                });
                            }
                        })
                    }),
            )
            .maybe_child(
                self.current_permissions
                    .has_channel_permission(ChannelPermission::ManageChannel)
                    .then(|| {
                        ContextMenuButton::new(add_circle_outline(), "Create Category").on_press({
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
