use freya::{
    prelude::*,
    radio::use_radio,
    text_edit::{EditableConfig, EditableEvent, EditorLine, TextEditor, use_editable},
};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Autocomplete, AutocompleteType, MessageModel, ModalValue, use_modals},
    consume_material_theme, get_channel_server, http,
};

#[derive(PartialEq)]
pub struct MessageEdit {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
    pub content: String,
}

impl Component for MessageEdit {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let modals = use_modals();
        let radio = use_radio(AppChannel::EditingMessage);
        let editing_message = radio.slice_mut_current(|state| &mut state.editing_message);

        let saving = use_state(|| false);

        let holder = use_state(ParagraphHolder::default);
        let mut editable = use_editable(|| self.content.clone(), EditableConfig::new);

        use_hook(|| {
            editable
                .editor_mut()
                .write()
                .move_cursor_to(self.content.len())
        });

        let mut autocomplete_visible = use_state(|| false);
        let is_server = get_channel_server(&self.channel.read()).is_some();

        let autocomplete = use_side_effect_value(move || {
            let editor = editable.editor().read();
            let text = editor.rope().to_string();
            let section: &str = &text[0..text.floor_char_boundary(editor.cursor_pos())];

            if let Some(last_char) = section.chars().last()
                && !last_char.is_whitespace()
                && let Some(last_section) = section.split_whitespace().last()
            {
                let mut chars = last_section
                    .chars()
                    .enumerate()
                    .skip_while(|(_, c)| !['@', '#', ':', '%'].contains(c) && !c.is_alphabetic());
                if let Some((pos, char)) = chars.next() {
                    if let Some(ty) = match char {
                        '@' => Some(AutocompleteType::User),
                        '#' if is_server => Some(AutocompleteType::Channel),
                        ':' => Some(AutocompleteType::Emoji),
                        '%' if is_server => Some(AutocompleteType::Role),
                        _ => None,
                    } {
                        autocomplete_visible.set(true);
                        return Some((ty, last_section[pos + 1..].to_string()));
                    };
                };
            };

            autocomplete_visible.set(false);
            None
        });

        let a11y_id = use_a11y();

        let save_message = {
            let editing_message = editing_message.clone();
            let editable = editable.clone();
            let message = self.message.clone();
            let channel = self.channel.clone();
            let saving = saving.clone();

            move || {
                let message = message.clone();
                let new_content = editable.editor().read().to_string();
                let channel_id = channel.read().id().to_string();
                let mut editing_message = editing_message.clone();
                let mut saving = saving.clone();

                if new_content.is_empty() {
                    modals
                        .clone()
                        .write()
                        .push_modal(ModalValue::DeleteMessage {
                            channel: channel_id.clone(),
                            message,
                        });
                    return;
                }

                spawn(async move {
                    saving.set(true);

                    http()
                        .edit_message(
                            &channel_id,
                            &message.message.id,
                            &v0::DataEditMessage {
                                content: Some(new_content),
                                embeds: None,
                            },
                        )
                        .await
                        .unwrap();

                    saving.set(false);
                    editing_message.set(None);
                });
            }
        };

        use_hook(|| a11y_id.request_focus());
        rect()
            .maybe_child(
                autocomplete
                    .read()
                    .cloned()
                    .map(|(autocomplete, query)| Autocomplete {
                        visible: autocomplete_visible,
                        autocomplete,
                        query,
                        editable,
                        channel: self.channel.clone(),
                    }),
            )
            .child(
                rect()
                    .spacing(4.)
                    .child(
                        rect()
                            .padding(8.)
                            .corner_radius(8.)
                            .background(theme.md.surface_container_highest.as_argb_u32())
                            .cursor(CursorIcon::Text)
                            .child(
                                paragraph()
                                    .margin((4., 6.))
                                    .cursor_color(0xFFFFFFFF)
                                    .width(Size::Fill)
                                    .a11y_id(a11y_id)
                                    .a11y_auto_focus(true)
                                    .a11y_focusable(true)
                                    .cursor_index(editable.editor().read().cursor_pos())
                                    .highlights(
                                        editable
                                            .editor()
                                            .read()
                                            .get_selection()
                                            .map(|selection| vec![selection])
                                            .unwrap_or_default(),
                                    )
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
                                    .on_global_pointer_up(move |_: Event<PointerEventData>| {
                                        editable.process_event(EditableEvent::Release)
                                    })
                                    .on_key_down({
                                        let save_message = save_message.clone();
                                        let mut editing_message = editing_message.clone();
                                        move |e: Event<KeyboardEventData>| {
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

                                            if e.key == Key::Named(NamedKey::Enter)
                                                && !e.modifiers.shift()
                                            {
                                                save_message();
                                            } else if e.key == Key::Named(NamedKey::Escape) {
                                                editing_message.set(None);
                                            } else {
                                                editable.process_event(EditableEvent::KeyDown {
                                                    key: &e.key,
                                                    modifiers: e.modifiers,
                                                    editor_line: Some(EditorLine::SingleParagraph),
                                                    holder: Some(&holder.read()),
                                                });
                                            };
                                        }
                                    })
                                    .on_key_up(move |e: Event<KeyboardEventData>| {
                                        editable
                                            .process_event(EditableEvent::KeyUp { key: &e.key });
                                    })
                                    .span(editable.editor().read().to_string())
                                    .holder(holder.read().clone()),
                            ),
                    )
                    .child(if *saving.read() {
                        rect().font_size(12.).child("Saving message...")
                    } else {
                        rect()
                            .spacing(4.)
                            .horizontal()
                            .font_size(12.)
                            .child("escape to")
                            .child(
                                rect()
                                    .child("cancel")
                                    .color(theme.md.primary.as_argb_u32())
                                    .cursor(CursorIcon::Pointer)
                                    .on_press({
                                        let mut editing_message = editing_message.clone();

                                        move |_| {
                                            editing_message.set(None);
                                        }
                                    }),
                            )
                            .child("· enter to")
                            .child(
                                rect()
                                    .child("save")
                                    .color(theme.md.primary.as_argb_u32())
                                    .cursor(CursorIcon::Pointer)
                                    .on_press({
                                        let save_message = save_message.clone();
                                        move |_| save_message()
                                    }),
                            )
                    }),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.message.message.id).into()
    }
}
