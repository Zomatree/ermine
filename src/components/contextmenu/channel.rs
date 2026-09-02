use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;
use stoat_permissions::{ChannelPermission, PermissionValue};

use crate::{
    AppChannel, ChannelSettingsPage,
    components::{
        ContextMenuButton, ModalValue,
        material::outlined::{badge, delete, logout, person_add, settings, share},
        use_modals,
    },
    http,
};

#[derive(PartialEq)]
pub struct ChannelContextMenu {
    pub channel_id: String,
    pub current_permissions: PermissionValue,
}

impl Component for ChannelContextMenu {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Channels);

        let channel = radio.slice_current({
            let channel_id = self.channel_id.clone();
            move |state| state.channels.get(&channel_id).unwrap()
        });
        let user_id = radio.slice(AppChannel::UserId, |state| state.user_id.as_ref().unwrap());

        let mut channel_settings = radio.slice_mut(AppChannel::ChannelSettingsPage, |state| {
            &mut state.channel_settings_page
        });

        let mut modals = use_modals();

        let mut menu = rect().content(Content::Fit).maybe_child(
            self.current_permissions
                .has_channel_permission(ChannelPermission::InviteOthers)
                .then(|| {
                    ContextMenuButton::new(person_add(), "Create Invite").on_press({
                        let channel_id = self.channel_id.clone();

                        move |_| {
                            let channel_id = channel_id.clone();

                            spawn_forever(async move {
                                match http().create_invite(&channel_id).await {
                                    Ok(
                                        v0::Invite::Group { code, .. }
                                        | v0::Invite::Server { code, .. },
                                    ) => {
                                        modals.write().push_modal(ModalValue::InviteInfo { code });
                                    }
                                    Err(error) => {
                                        modals.write().push_modal(ModalValue::Error { error });
                                    }
                                }
                            });
                        }
                    })
                }),
        );

        menu = match &*channel.read() {
            v0::Channel::DirectMessage { recipients, .. } => {
                menu.child(ContextMenuButton::new(badge(), "Copy User ID").on_press({
                    let user_id = user_id.read();

                    let other = recipients
                        .iter()
                        .find(|&id| id != &*user_id)
                        .unwrap()
                        .clone();

                    let user = radio.slice(AppChannel::Users, move |state| {
                        state.users.get(&other).unwrap()
                    });

                    move |_| {
                        Clipboard::set(user.read().id.clone()).unwrap();
                    }
                }))
            }
            v0::Channel::Group { id, .. } | v0::Channel::TextChannel { id, .. } => menu
                .child(ContextMenuButton::new(share(), "Copy Link").on_press({
                    let id = id.clone();

                    move |_| {
                        Clipboard::set(format!("https://stoat.chat/channel/{id}")).unwrap();
                    }
                }))
                .child(
                    ContextMenuButton::new(badge(), "Copy Channel ID").on_press({
                        let id = id.clone();

                        move |_| {
                            Clipboard::set(id.clone()).unwrap();
                        }
                    }),
                )
                .maybe_child(
                    self.current_permissions
                        .has_channel_permission(ChannelPermission::ManageChannel)
                        .then(|| {
                            ContextMenuButton::new(settings(), "Open Channel Settings").on_press({
                                let channel_id = self.channel_id.clone();

                                move |_| {
                                    *channel_settings.write() =
                                        Some((channel_id.clone(), ChannelSettingsPage::default()));
                                }
                            })
                        }),
                ),
            _ => unreachable!(),
        };

        let menu = match &*channel.read() {
            v0::Channel::Group { .. } => menu.child(
                ContextMenuButton::new(logout(), "Leave Group")
                    .danger()
                    .on_press({
                        let id = self.channel_id.clone();

                        move |_| {
                            modals.write().push_modal(ModalValue::LeaveGroup {
                                channel: id.clone(),
                            });
                        }
                    }),
            ),
            v0::Channel::TextChannel { .. } => menu.maybe_child(
                self.current_permissions
                    .has_channel_permission(ChannelPermission::ManageChannel)
                    .then(|| {
                        ContextMenuButton::new(delete(), "Delete Channel")
                            .danger()
                            .on_press({
                                let id = self.channel_id.clone();

                                move |_| {
                                    modals.write().push_modal(ModalValue::DeleteChannel {
                                        channel: id.clone(),
                                    });
                                }
                            })
                    }),
            ),
            _ => menu,
        };

        menu
    }
}
