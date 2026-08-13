use freya::{
    prelude::*,
    radio::use_radio,
    text_edit::{TextEditor, TextSelection, UseEditable},
};
use stoat_permissions::{ChannelPermission, PermissionValue};

use crate::{
    AppChannel, EditingMessage,
    components::{
        ContextMenuButton, EmojiPicker, MessageModel, ModalValue, ReplyController,
        material::{
            filled::reply,
            outlined::{
                alternate_email, badge, content_copy, delete, edit, insert_emoticon, pin_invoke,
                share,
            },
        },
        use_floating, use_modals,
    },
    http,
};

#[derive(PartialEq)]
pub struct MessageContextMenu {
    pub message: MessageModel,
    pub replies: ReplyController,
    pub current_permissions: PermissionValue,
}

impl Component for MessageContextMenu {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());
        let mut editing_message = radio.slice_mut(AppChannel::EditingMessage, |state| {
            &mut state.editing_message
        });

        let mut shift = use_state(|| false);

        let mut floating = use_floating();
        let mut modals = use_modals();
        let editable = use_consume::<Option<UseEditable>>();

        rect()
            .content(Content::Fit)
            .on_global_key_down(move |e: Event<KeyboardEventData>| {
                if e.key == Key::Named(NamedKey::Shift) {
                    shift.set(true);
                }
            })
            .on_global_key_up(move |e: Event<KeyboardEventData>| {
                if e.key == Key::Named(NamedKey::Shift) {
                    shift.set(false);
                }
            })
            .child(ContextMenuButton::new(reply(), "Reply").on_press({
                let message = self.message.clone();
                let mut replies = self.replies.clone();

                move |_| {
                    replies.add_reply(message.clone(), true);
                }
            }))
            .maybe_child(editable.map(|mut editable| {
                ContextMenuButton::new(alternate_email(), "Mention").on_press({
                    let user_id = self.message.message.author.clone();

                    move |_| {
                        let mut editor = editable.editor_mut().write();

                        let pos = editor.cursor_pos();
                        editor.insert(&format!("<@{user_id}>"), pos);
                        *editor.selection_mut() = TextSelection::new_cursor(editor.len_chars())
                    }
                })
            }))
            .child(
                ContextMenuButton::new(content_copy(), "Copy text").on_press({
                    let message = self.message.clone();
                    move |_| {
                        if let Some(content) = message.message.content.clone() {
                            Clipboard::set(content).unwrap();
                        }
                    }
                }),
            )
            .maybe_child(
                self.current_permissions
                    .has_channel_permission(ChannelPermission::React)
                    .then(|| {
                        ContextMenuButton::new(insert_emoticon(), "React").on_press({
                            let message_id = self.message.message.id.clone();
                            let channel_id = self.message.message.channel.clone();

                            move |_| {
                                let message_id = message_id.clone();
                                let channel_id = channel_id.clone();

                                *floating.write() = Some(
                                    EmojiPicker::new({
                                        move |id: String| {
                                            let message_id = message_id.clone();
                                            let channel_id = channel_id.clone();
                                            floating.set(None);

                                            spawn_forever(async move {
                                                http()
                                                    .react_message(&channel_id, &message_id, &id)
                                                    .await
                                                    .unwrap();
                                            });
                                        }
                                    })
                                    .into_element(),
                                )
                            }
                        })
                    }),
            )
            .maybe_child((&self.message.message.author == &*user_id.read()).then(|| {
                ContextMenuButton::new(edit(), "Edit Message").on_press({
                    let message = self.message.clone();

                    move |_| {
                        *editing_message.write() = Some(EditingMessage {
                            id: message.message.id.clone(),
                            content: message.message.content.clone().unwrap_or_default(),
                        })
                    }
                })
            }))
            .maybe_child(
                (self
                    .current_permissions
                    .has_channel_permission(ChannelPermission::ManageMessages)
                    && self.message.message.pinned.is_none_or(|pinned| !pinned))
                .then(|| {
                    ContextMenuButton::new(pin_invoke(), "Pin Message").on_press({
                        let message_id = self.message.message.id.clone();
                        let channel_id = self.message.message.channel.clone();

                        move |_| {
                            let message_id = message_id.clone();
                            let channel_id = channel_id.clone();

                            spawn_forever(async move {
                                http().pin_message(&channel_id, &message_id).await.unwrap();
                            });
                        }
                    })
                }),
            )
            .maybe_child(
                (self
                    .current_permissions
                    .has_channel_permission(ChannelPermission::ManageMessages)
                    && self.message.message.pinned == Some(true))
                .then(|| {
                    ContextMenuButton::new(pin_invoke(), "Unpin Message").on_press({
                        let message_id = self.message.message.id.clone();
                        let channel_id = self.message.message.channel.clone();

                        move |_| {
                            let message_id = message_id.clone();
                            let channel_id = channel_id.clone();

                            spawn_forever(async move {
                                http()
                                    .unpin_message(&channel_id, &message_id)
                                    .await
                                    .unwrap();
                            });
                        }
                    })
                }),
            )
            .maybe_child(
                (self
                    .current_permissions
                    .has_channel_permission(ChannelPermission::ManageMessages)
                    || &self.message.message.author == &*user_id.read())
                    .then(|| {
                        ContextMenuButton::new(delete(), "Delete Message")
                            .danger()
                            .on_press({
                                let message_id = self.message.message.id.clone();
                                let channel_id = self.message.message.channel.clone();

                                move |_| {
                                    let message_id = message_id.clone();
                                    let channel_id = channel_id.clone();

                                    if shift() {
                                        spawn_forever(async move {
                                            http()
                                                .delete_message(&channel_id, &message_id)
                                                .await
                                                .unwrap();
                                        });
                                    } else {
                                        modals.write().push_modal(ModalValue::DeleteMessage {
                                            channel: channel_id.clone(),
                                            message: message_id.clone(),
                                        });
                                    }
                                }
                            })
                    }),
            )
            .child(
                ContextMenuButton::new(share(), "Copy Message Link").on_press({
                    let message_id = self.message.message.id.clone();
                    let channel_id = self.message.message.channel.clone();

                    move |_| {
                        Clipboard::set(format!(
                            "https://stoat.chat/channel/{channel_id}/{message_id}"
                        ))
                        .unwrap();
                    }
                }),
            )
            .child(
                ContextMenuButton::new(badge(), "Copy Message ID").on_press({
                    let message_id = self.message.message.id.clone();

                    move |_| {
                        Clipboard::set(message_id.clone()).unwrap();
                    }
                }),
            )
    }
}
