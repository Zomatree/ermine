use freya::{
    prelude::*,
    radio::{use_radio, use_radio_station},
    text_edit::{TextEditor, TextSelection, UseEditable},
};
use iso8601_timestamp::Timestamp;
use stoat_models::v0;
use stoat_permissions::{ChannelPermission, PermissionValue};

use crate::{
    AppChannel, AppState, Selection,
    components::{
        ContextMenuButton, ContextMenuDivider, ModalValue,
        material::outlined::{
            account_circle, add_circle_outline, alternate_email, assignment, badge,
            do_not_disturb_on, face, message, not_interested, person_remove, report, timer,
            timer_off,
        },
        use_modals,
    },
    http, insert_channel, is_inferior,
};

#[derive(PartialEq)]
pub struct UserContextMenu {
    pub user_id: String,
    pub server_id: Option<String>,
    pub permissions: PermissionValue,
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
        let current_user_id =
            radio.slice(AppChannel::UserId, |state| state.user_id.as_ref().unwrap());

        let server = self.server_id.clone().map(|server_id| {
            radio.slice(AppChannel::Servers, move |state| {
                state.servers.get(&server_id).unwrap()
            })
        });
        let member = self.server_id.clone().map(|server_id| {
            let user_id = self.user_id.clone();

            radio.slice(AppChannel::Members, move |state| {
                state
                    .members
                    .get(&server_id)
                    .unwrap()
                    .get(&user_id)
                    .unwrap()
            })
        });
        let current_member = self.server_id.clone().map(|server_id| {
            radio.slice(AppChannel::Members, move |state| {
                state
                    .members
                    .get(&server_id)
                    .unwrap()
                    .get(state.user_id.as_ref().unwrap())
                    .unwrap()
            })
        });

        let editable = try_consume_root_context::<Option<UseEditable>>().flatten();

        let user = user.read();
        let is_ourself = &user.id == &*current_user_id.read();

        rect()
            .content(Content::Fit)
            .child(
                ContextMenuButton::new(account_circle(), "Profile").on_press({
                    let user_id = self.user_id.clone();
                    let server_id = self.server_id.clone();

                    move |_| {
                        *user_profile.write() = Some((user_id.clone(), server_id.clone()));
                    }
                }),
            )
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
            .child(ContextMenuDivider)
            .map(
                server.zip(member).zip(current_member),
                |this, ((server, member), current_member)| {
                    let server = server.read();
                    let member = member.read();
                    let current_member = current_member.read();
                    let is_inferior = is_inferior(&current_member, &member, &server);

                    let edit_identity = !is_ourself
                        && (self
                            .permissions
                            .has_channel_permission(ChannelPermission::ManageNicknames)
                            || self
                                .permissions
                                .has_channel_permission(ChannelPermission::RemoveAvatars))
                        && is_inferior;

                    let edit_own_identity = is_ourself
                        && (self
                            .permissions
                            .has_channel_permission(ChannelPermission::ChangeNickname)
                            || self
                                .permissions
                                .has_channel_permission(ChannelPermission::ChangeAvatar));

                    let edit_roles = &server.owner == &current_member.id.user
                        || (self
                            .permissions
                            .has_channel_permission(ChannelPermission::AssignRoles)
                            && is_inferior);

                    let kick_members = !is_ourself
                        && self
                            .permissions
                            .has_channel_permission(ChannelPermission::KickMembers)
                        && is_inferior;

                    let ban_members = !is_ourself
                        && self
                            .permissions
                            .has_channel_permission(ChannelPermission::BanMembers)
                        && is_inferior;

                    let timeout_members = !is_ourself
                        && self
                            .permissions
                            .has_channel_permission(ChannelPermission::TimeoutMembers)
                        && is_inferior;

                    let is_timed_out = member.timeout.is_some_and(|ts| ts > Timestamp::now_utc());

                    this.maybe_child(edit_own_identity.then(|| {
                        ContextMenuButton::new(face(), "Edit Your Identity").on_press({
                            let server = server.id.clone();

                            move |_| {
                                modals
                                    .write()
                                    .push_modal(ModalValue::EditOwnServerIdentity {
                                        server: server.clone(),
                                    });
                            }
                        })
                    }))
                    .maybe_child(edit_identity.then(|| {
                        ContextMenuButton::new(face(), "Edit Identity").on_press({
                            let user = self.user_id.clone();
                            let server = server.id.clone();

                            move |_| {
                                modals.write().push_modal(ModalValue::EditServerIdentity {
                                    user: user.clone(),
                                    server: server.clone(),
                                });
                            }
                        })
                    }))
                    .maybe_child(edit_roles.then(|| {
                        ContextMenuButton::new(assignment(), "Edit Roles").on_press({
                            let user = self.user_id.clone();
                            let server = server.id.clone();

                            move |_| {
                                modals.write().push_modal(ModalValue::EditRoles {
                                    user: user.clone(),
                                    server: server.clone(),
                                });
                            }
                        })
                    }))
                    .maybe_child(
                        ((edit_own_identity || edit_identity || edit_roles)
                            && (kick_members || ban_members || timeout_members))
                            .then(|| ContextMenuDivider),
                    )
                    .maybe_child(timeout_members.then(|| {
                        ContextMenuButton::new(
                            if is_timed_out { timer_off() } else { timer() },
                            if is_timed_out {
                                "Remove Timeout"
                            } else {
                                "Timeout Member"
                            },
                        )
                        .danger()
                        .on_press({
                            let user = self.user_id.clone();
                            let server = server.id.clone();

                            move |_| {
                                modals.write().push_modal(if is_timed_out {
                                    ModalValue::RemoveTimeoutMember {
                                        user: user.clone(),
                                        server: server.clone(),
                                    }
                                } else {
                                    ModalValue::TimeoutMember {
                                        user: user.clone(),
                                        server: server.clone(),
                                    }
                                });
                            }
                        })
                    }))
                    .maybe_child(kick_members.then(|| {
                        ContextMenuButton::new(person_remove(), "Kick Member")
                            .danger()
                            .on_press({
                                let user = self.user_id.clone();
                                let server = server.id.clone();

                                move |_| {
                                    modals.write().push_modal(ModalValue::KickMember {
                                        user: user.clone(),
                                        server: server.clone(),
                                    });
                                }
                            })
                    }))
                    .maybe_child(ban_members.then(|| {
                        ContextMenuButton::new(do_not_disturb_on(), "Ban Member")
                            .danger()
                            .on_press({
                                let user = self.user_id.clone();
                                let server = server.id.clone();

                                move |_| {
                                    modals.write().push_modal(ModalValue::BanMember {
                                        user: user.clone(),
                                        server: server.clone(),
                                    });
                                }
                            })
                    }))
                    .maybe_child(
                        (edit_own_identity
                            || edit_identity
                            || edit_roles
                            || kick_members
                            || ban_members
                            || timeout_members)
                            .then(|| ContextMenuDivider),
                    )
                },
            )
            .maybe(!is_ourself, |this| {
                let is_blocked = user.relationship == v0::RelationshipStatus::Blocked;

                this.child(
                    ContextMenuButton::new(
                        if is_blocked {
                            add_circle_outline()
                        } else {
                            not_interested()
                        },
                        if is_blocked {
                            "Unblock User"
                        } else {
                            "Block User"
                        },
                    )
                    .maybe(!is_blocked, |this| this.danger())
                    .on_press({
                        let user_id = self.user_id.clone();

                        move |_| {
                            let user_id = user_id.clone();

                            spawn_forever(async move {
                                if is_blocked {
                                    http().unblock_user(&user_id).await
                                } else {
                                    http().block_user(&user_id).await
                                }
                                .unwrap();
                            });
                        }
                    }),
                )
                .child(ContextMenuButton::new(report(), "Report User").danger())
                .child(ContextMenuDivider)
            })
            .child(ContextMenuButton::new(badge(), "Copy User ID").on_press({
                let user_id = self.user_id.clone();

                move |_| {
                    Clipboard::set(user_id.clone()).unwrap();
                }
            }))
    }
}
