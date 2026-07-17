use std::borrow::Cow;

use freya::{
    prelude::*,
    radio::use_radio,
    text_edit::*,
};
use stoat_models::v0;
use stoat_permissions::{ChannelPermission, PermissionValue};

use crate::{
    AppChannel, LocalFile, SizeExt, calculate_channel_permissions, components::{
        AttachmentController, EmojiPicker, ReplyController, StoatButton, StoatButtonLayoutThemePartialExt, material::{MaterialIcon, filled::{add, do_not_disturb}, round::insert_emoticon}, use_floating
    }, consume_material_theme, http, user_permissions_query
};

#[derive(PartialEq)]
pub struct Textbox {
    pub editable: UseEditable,
    pub autocomplete_visible: State<bool>,
    pub replies: ReplyController,
    pub attachments: AttachmentController,
    pub channel: Readable<v0::Channel>,
}

impl Component for Textbox {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let users_last_message = radio.slice(AppChannel::UsersLastMessage, |state| {
            &state.users_last_message
        });
        let mut editing_message = radio.slice_mut(AppChannel::EditingMessage, |state| {
            &mut state.editing_message
        });
        let file_hover = radio.slice_mut(AppChannel::FileHover, |state| &mut state.file_hover);

        let theme = consume_material_theme();
        let holder = use_state(ParagraphHolder::default);
        let a11y_id = use_a11y();
        let mut floating = use_floating();

        let mut editable = self.editable;

        let permissions = use_state(|| PermissionValue::from_raw(0));

        use_side_effect({
            let radio = radio.clone();
            let channel = self.channel.clone();

            move || {
                let radio = radio.clone();
                let channel = channel.clone();

                spawn(async move {
                    let mut query =
                        user_permissions_query(radio.clone()).channel(channel.read().clone());

                    let value = calculate_channel_permissions(&mut query).await;
                    permissions.clone().set(value);
                });
            }
        });

        let can_send_messages = permissions
            .read()
            .has_channel_permission(ChannelPermission::SendMessage);

        use_hook(|| a11y_id.request_focus());

        use_side_effect({
            let editing_message = editing_message.clone();
            move || {
                if editing_message.read().is_none() {
                    a11y_id.request_focus()
                }
            }
        });

        rect()
            .width(Size::Fill)
            .font_size(14)
            .horizontal()
            // .spacing(8.)
            .content(Content::Flex)
            .background(theme.md.surface_container_high.as_argb_u32())
            .corner_radius(28.)
            .padding((4., 8., 4., 0.))
            .cross_align(Alignment::Center)
            .min_height(Size::px(48.))
            .child(
                rect()
                    .horizontal()
                    .width(Size::px(62.))
                    .height(Size::px(40.))
                    .center()
                    .maybe_child(if !can_send_messages {
                        Some(
                            MaterialIcon::new(do_not_disturb())
                                .size(Size::px(24.))
                                .into_element(),
                        )
                    } else if permissions
                        .read()
                        .has_channel_permission(ChannelPermission::UploadFiles)
                    {
                        Some(
                            rect()
                                .on_global_file_hover({
                                    let file_hover = file_hover.clone();

                                    move |_| file_hover.clone().set(true)
                                })
                                .on_global_file_hover_cancelled(move |_| {
                                    file_hover.clone().set(false)
                                })
                                .child(
                                    StoatButton::new()
                                        .corner_radius(40.)
                                        .on_press({
                                            let attachments = self.attachments.clone();

                                            move |_| {
                                                spawn(async move {
                                                    attachments.prompt().await;
                                                });
                                            }
                                        })
                                        .child(
                                            rect()
                                                .width(Size::px(40.))
                                                .height(Size::px(40.))
                                                .center()
                                                .child(
                                                    MaterialIcon::new(add())
                                                        .size(Size::px(24.))
                                                ),
                                        ),
                                )
                                .into_element(),
                        )
                    } else {
                        None
                    }),
            )
            .child(if !can_send_messages {
                rect().width(Size::flex(1.)).child(
                    label()
                        .font_size(14.)
                        .text("You don't have permission to send messages in this channel."),
                )
            } else {
                rect()
                    .width(Size::flex(1.))
                    .padding((4., 0.))
                    .child(
                        paragraph()
                            .a11y_focusable(Focusable::Enabled)
                            .line_height(1.4)
                            .width(Size::Fill)
                            .a11y_id(a11y_id)
                            .a11y_auto_focus(true)
                            .cursor_index(editable.editor().read().cursor_pos())
                            .cursor_style(CursorStyle::Line)
                            .cursor_color(0xFFFFFFFF)
                            .highlights(
                                editable
                                    .editor()
                                    .read()
                                    .get_selection()
                                    .map(|selection| vec![selection])
                                    .unwrap_or_default(),
                            )
                            .on_pointer_enter(move |_| {
                                Cursor::set(CursorIcon::Text);
                            })
                            .on_pointer_leave(move |_| {
                                Cursor::set(CursorIcon::default());
                            })
                            .on_mouse_down(move |e: Event<MouseEventData>| {
                                a11y_id.request_focus();
                                editable.process_event(EditableEvent::Down {
                                    location: e.element_location,
                                    editor_line: EditorLine::SingleParagraph,
                                    holder: &holder.read(),
                                });
                            })
                            .on_mouse_move(move |e: Event<MouseEventData>| {
                                editable.process_event(EditableEvent::Move {
                                    location: e.element_location,
                                    editor_line: EditorLine::SingleParagraph,
                                    holder: &holder.read(),
                                });
                            })
                            .on_global_pointer_press(move |_: Event<PointerEventData>| {
                                editable.process_event(EditableEvent::Release)
                            })
                            .on_key_down({
                                let channel = self.channel.clone();
                                let mut replies = self.replies.clone();
                                let attachments = self.attachments.clone();
                                let autocomplete_visible = self.autocomplete_visible;

                                move |e: Event<KeyboardEventData>| {
                                    println!("textbox down");

                                    if autocomplete_visible()
                                        && matches!(
                                            e.key,
                                            Key::Named(
                                                NamedKey::Enter
                                                    | NamedKey::ArrowUp
                                                    | NamedKey::ArrowDown
                                            )
                                        )
                                    {
                                        return;
                                    };

                                    if e.key == Key::Named(NamedKey::Enter) && !e.modifiers.shift()
                                    {
                                        let editor = editable.editor_mut();
                                        let mut writer = editor.write();
                                        let content = writer.to_string();

                                        if content.is_empty() && attachments.is_empty() {
                                            return;
                                        }

                                        *writer = RopeEditor::new(
                                            String::new(),
                                            TextSelection::new_cursor(0),
                                            0,
                                            writer.editor_history().clone(),
                                        );
                                        drop(writer);

                                        let message_replies = replies.take_replies();

                                        let attachments = attachments.take();

                                        spawn({
                                            let channel_id =
                                                channel.clone().read().id().to_string();

                                            async move {
                                                let mut attachment_ids = Vec::new();

                                                for attachment in attachments.into_values() {
                                                    let file = http()
                                                        .upload_file(
                                                            "attachments",
                                                            LocalFile {
                                                                name: if attachment.spoiler {
                                                                    format!(
                                                                        "SPOILER_{}",
                                                                        attachment.filename
                                                                    )
                                                                } else {
                                                                    attachment.filename
                                                                },
                                                                body: attachment.contents.into(),
                                                            },
                                                        )
                                                        .await
                                                        .unwrap();
                                                    attachment_ids.push(file.id);
                                                }

                                                http()
                                                    .send_message(
                                                        &channel_id,
                                                        &v0::DataMessageSend {
                                                            nonce: None,
                                                            content: Some(content),
                                                            attachments: Some(attachment_ids),
                                                            replies: Some(message_replies),
                                                            embeds: None,
                                                            masquerade: None,
                                                            interactions: None,
                                                            flags: None,
                                                        },
                                                    )
                                                    .await
                                                    .unwrap();
                                            }
                                        });
                                    } else if e.key == Key::Named(NamedKey::ArrowUp)
                                        && !e.modifiers.shift()
                                        && editable.editor().read().to_string().len() == 0
                                    {
                                        let last_message = users_last_message.read().clone();

                                        if let Some(last_message) = last_message {
                                            *editing_message.write() = Some(last_message);
                                        }
                                    } else {
                                        editable.process_event(EditableEvent::KeyDown {
                                            key: &e.key,
                                            modifiers: e.modifiers,
                                        });
                                    }
                                }
                            })
                            .on_key_up(move |e: Event<KeyboardEventData>| {
                                println!("textbox up");
                                editable.process_event(EditableEvent::KeyUp { key: &e.key });
                            })
                            .span(editable.editor().read().to_string())
                            .holder(holder.read().clone()),
                    )
                    .maybe_child(editable.editor().read().to_string().is_empty().then(|| {
                        rect()
                            .child(
                                label()
                                    .text(format!(
                                        "Message {}",
                                        match &*self.channel.read() {
                                            v0::Channel::DirectMessage { recipients, .. } => {
                                                let user_id =
                                                    radio.peek_state().user_id.clone().unwrap();

                                                let other = recipients
                                                    .iter()
                                                    .find(|&id| id != &*user_id)
                                                    .unwrap()
                                                    .clone();

                                                let user = radio
                                                    .slice(AppChannel::Users, move |state| {
                                                        state.users.get(&other).unwrap()
                                                    });

                                                Cow::Owned(user.read().username.clone())
                                            }
                                            v0::Channel::Group { name, .. }
                                            | v0::Channel::TextChannel { name, .. } =>
                                                Cow::Owned(name.clone()),
                                            v0::Channel::SavedMessages { .. } =>
                                                Cow::Borrowed("Saved Messages"),
                                        }
                                    ))
                                    .color(0xff888888),
                            )
                            .layer(Layer::OverlayLevel(1))
                            .position(Position::new_absolute())
                    }))
            })
            .maybe_child(can_send_messages.then(|| {
                StoatButton::new()
                    .corner_radius(40.)
                    .on_press({
                        move |_| {
                            floating.set(Some(
                                EmojiPicker::new(move |e: String| {
                                    floating.set(None);

                                    let e = if e.len() == 26 { format!(":{e}:") } else { e };

                                    let mut editor = editable.editor_mut().write();
                                    let selection = editor.get_selection_range();
                                    if let Some((start, end)) = selection {
                                        editor.remove(start..end);
                                        editor.move_cursor_to(start);
                                    }
                                    let cursor_pos = editor.cursor_pos();
                                    let last_idx = e.encode_utf16().count() + cursor_pos;
                                    editor.insert(&e, cursor_pos);
                                    editor.selection_mut().move_to(last_idx);
                                    editor.selection_mut().set_as_cursor();
                                })
                                .into_element(),
                            ));
                        }
                    })
                    .child(
                        rect()
                            .width(Size::px(40.))
                            .height(Size::px(40.))
                            .center()
                            .child(
                                MaterialIcon::new(insert_emoticon())
                                    .size(Size::px(24.))
                            ),
                    )
            }))
    }
}
