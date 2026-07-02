use std::{fmt::Debug, mem};

use freya::{
    prelude::*,
    text_edit::{EditableConfig, TextEditor, use_editable},
};
use indexmap::IndexMap;
use rfd::AsyncFileDialog;
use stoat_models::v0;

use crate::{
    components::{Autocomplete, MessageAttachmentsPreview, MessageModel, MessageReplyPreview, Textbox},
    map_readable,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ReplyIntent {
    pub message: MessageModel,
    pub mention: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ReplyController(pub State<Vec<ReplyIntent>>);

impl ReplyController {
    pub fn get_replies(&self) -> impl Iterator<Item = Readable<ReplyIntent>> {
        self.0.read().clone().into_iter().map(|reply| {
            let id = reply.message.message.id.clone();

            map_readable::<Vec<ReplyIntent>, ReplyIntent>(self.0.into_readable(), move |replies| {
                replies.iter().find(|r| r.message.message.id == id).unwrap()
            })
        })
    }

    pub fn toggle_mention(&mut self, message_id: &str) {
        if let Some(reply) = self
            .0
            .write()
            .iter_mut()
            .find(|r| r.message.message.id == message_id)
        {
            reply.mention = !reply.mention;
        }
    }

    pub fn add_reply(&mut self, message: MessageModel, mention: bool) {
        let message_id = &message.message.id;
        let mut replies = self.0.write();

        if replies
            .iter()
            .any(|reply| &reply.message.message.id == message_id)
        {
            return;
        };

        replies.push(ReplyIntent { message, mention });
    }

    pub fn remove_reply(&mut self, message_id: &str) {
        self.0.with_mut(|mut replies| {
            replies.retain(|r| r.message.message.id != message_id);
        });
    }

    pub fn take_replies(&mut self) -> Vec<v0::ReplyIntent> {
        let replies = std::mem::take(&mut *self.0.write());

        replies
            .into_iter()
            .map(|reply| v0::ReplyIntent {
                id: reply.message.message.id.clone(),
                mention: reply.mention,
                fail_if_not_exists: Some(true),
            })
            .collect()
    }
}

#[derive(Clone, PartialEq)]
pub struct Attachment {
    pub controller: AttachmentController,

    pub id: u64,
    pub filename: String,
    pub spoiler: bool,
    pub contents: Bytes,
}

impl Debug for Attachment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Attachment")
            .field("id", &self.id)
            .field("filename", &self.filename)
            .field("spoiler", &self.spoiler)
            .field("contents", &self.contents)
            .finish_non_exhaustive()
    }
}

impl Attachment {
    pub fn remove(&self) {
        self.controller.remove(self.id);
    }

    pub fn toggle_spoiler(&self) {
        self.controller.toggle_spoiler(self.id);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct AttachmentController(pub State<IndexMap<u64, Attachment>>);

impl AttachmentController {
    pub fn is_empty(&self) -> bool {
        self.0.read().is_empty()
    }

    pub fn not_empty(&self) -> bool {
        !self.is_empty()
    }

    pub fn get_attachments(&self) -> impl Iterator<Item = Attachment> {
        self.0.read().clone().into_values()
    }

    pub fn remove(&self, id: u64) {
        self.0.clone().write().shift_remove(&id);
    }

    pub fn take(&self) -> IndexMap<u64, Attachment> {
        mem::take(&mut *self.0.clone().write())
    }

    pub fn toggle_spoiler(&self, id: u64) {
        self.0.clone().with_mut(|mut attachments| {
            if let Some(attachment) = attachments.get_mut(&id) {
                attachment.spoiler = !attachment.spoiler;
            };
        });
    }

    pub async fn prompt(&self) {
        if let Some(file) = AsyncFileDialog::new().pick_file().await {
            let contents = file.read().await.into();
            let filename = file.file_name();

            let id = rand::random();

            let (filename, spoiler) = if let Some(filename) = filename.strip_prefix("SPOILER_") {
                (filename.to_string(), true)
            } else {
                (filename, false)
            };

            let attachment = Attachment {
                controller: *self,
                id,
                filename,
                spoiler,
                contents,
            };

            self.0.clone().write().insert(id, attachment);
        };
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AutocompleteType {
    User,
    Channel,
    Emoji,
    Role,
}

#[derive(PartialEq)]
pub struct MessageInput {
    pub channel: Readable<v0::Channel>,
    pub replies: ReplyController,
    pub attachments: AttachmentController,
}

impl Component for MessageInput {
    fn render(&self) -> impl IntoElement {
        let editable = use_editable(String::new, EditableConfig::new);

        let is_server = matches!(&*self.channel.read(), v0::Channel::TextChannel { .. });

        let autocomplete = use_side_effect_value(move || {
            let editor = editable.editor().read();
            let text = editor.rope().to_string();
            let section = &text[0..editor.cursor_pos()];

            if let Some(last_char) = section.chars().last()
                && !last_char.is_whitespace()
                && let Some(last_section) = section.split_whitespace().last()
            {
                if let Some(ty) = match last_section.chars().next() {
                    Some('@') => Some(AutocompleteType::User),
                    Some('#') if is_server => Some(AutocompleteType::Channel),
                    // Some(':') => Some(AutocompleteType::Emoji),
                    Some('%') if is_server => Some(AutocompleteType::Role),
                    _ => None,
                } {
                    return Some((ty, last_section[1..].to_string()));
                };
            };

            None
        });

        rect()
            .width(Size::Fill)
            .margin((0., 8., 8., 8.))
            .maybe_child(autocomplete.read().cloned().map(|(autocomplete, query)| {
                Autocomplete {
                    autocomplete,
                    query,
                    editable,
                    channel: self.channel.clone(),
                }
            }))
            .maybe_child(self.attachments.not_empty().then(|| {
                rect()
                    .margin((0., 0., 8., 0.))
                    .child(MessageAttachmentsPreview {
                        attachments: self.attachments,
                    })
                    .into_element()
            }))
            .child(rect().children(self.replies.get_replies().map(|reply| {
                rect()
                    .key(&reply.read().message.message.id)
                    .margin((0., 0., 8., 0.))
                    .child(MessageReplyPreview {
                        replies: self.replies,
                        reply,
                        channel: self.channel.clone(),
                    })
                    .into_element()
            })))
            .child(Textbox {
                editable,
                replies: self.replies,
                attachments: self.attachments,
                channel: self.channel.clone(),
            })
    }
}
