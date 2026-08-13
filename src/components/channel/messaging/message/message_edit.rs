use freya::{
    prelude::*,
    radio::use_radio,
    text_edit::{EditableConfig, EditableEvent, EditorLine, TextEditor, use_editable},
};
use stoat_models::v0;

use crate::{AppChannel, http};

#[derive(PartialEq)]
pub struct MessageEdit {
    pub channel: Readable<v0::Channel>,
    pub id: String,
    pub content: String,
}

impl Component for MessageEdit {
    fn render(&self) -> impl IntoElement {
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

        let a11y_id = use_a11y();

        let save_message = {
            let editing_message = editing_message.clone();
            let editable = editable.clone();
            let id = self.id.clone();
            let channel = self.channel.clone();
            let saving = saving.clone();

            move || {
                let id = id.clone();
                let new_content = editable.editor().read().to_string();
                let channel_id = channel.read().id().to_string();
                let mut editing_message = editing_message.clone();
                let mut saving = saving.clone();

                spawn(async move {
                    saving.set(true);

                    http()
                        .edit_message(
                            &channel_id,
                            &id,
                            &v0::DataEditMessage {
                                content: Some(new_content),
                                embeds: None,
                            },
                        )
                        .await.unwrap();

                    saving.set(false);
                    editing_message.set(None);
                });
            }
        };

        use_hook(|| a11y_id.request_focus());

        rect()
            .spacing(4.)
            .child(
                rect()
                    .padding(8.)
                    .corner_radius(8.)
                    .background(0xff34343a)
                    .child(
                        paragraph()
                            .margin((4., 6.))
                            .cursor_color(0xFFFFFFFF)
                            .width(Size::Fill)
                            .a11y_id(a11y_id)
                            .a11y_auto_focus(true)
                            .a11y_focusable(true)
                            .cursor_index(editable.editor().read().cursor_pos())
                            // .on_sized(move |_| a11y_id.request_focus())
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
                            .on_global_pointer_press(move |_: Event<PointerEventData>| {
                                editable.process_event(EditableEvent::Release)
                            })
                            .on_key_down({
                                let save_message = save_message.clone();
                                let mut editing_message = editing_message.clone();
                                move |e: Event<KeyboardEventData>| {
                                    if e.key == Key::Named(NamedKey::Enter) && !e.modifiers.shift()
                                    {
                                        save_message();
                                    } else if e.key == Key::Named(NamedKey::Escape) {
                                        editing_message.set(None);
                                    } else {
                                        editable.process_event(EditableEvent::KeyDown {
                                            key: &e.key,
                                            modifiers: e.modifiers,
                                        });
                                    };
                                }
                            })
                            .on_key_up(move |e: Event<KeyboardEventData>| {
                                editable.process_event(EditableEvent::KeyUp { key: &e.key });
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
                            .color(0xffb9c3ff)
                            .on_pointer_enter(move |_| {
                                Cursor::set(CursorIcon::Pointer);
                            })
                            .on_pointer_leave(move |_| {
                                Cursor::set(CursorIcon::default());
                            })
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
                            .color(0xffb9c3ff)
                            .on_pointer_enter(move |_| {
                                Cursor::set(CursorIcon::Pointer);
                            })
                            .on_pointer_leave(move |_| {
                                Cursor::set(CursorIcon::default());
                            })
                            .on_press({
                                let save_message = save_message.clone();
                                move |_| save_message()
                            }),
                    )
            })
    }

    fn render_key(&self) -> DiffKey {
        (&self.id).into()
    }
}
