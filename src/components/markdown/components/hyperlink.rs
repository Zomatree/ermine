use freya::prelude::*;

use crate::components::{
    ModalValue,
    markdown::components::{Invite, MessageUrl},
    stoat_paragraph, use_modals,
};

#[derive(PartialEq)]
pub struct Hyperlink {
    pub span: Span<'static>,
    pub foreground: Color,
}

impl Component for Hyperlink {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();
        let mut hover = use_state(|| false);

        let invite_id = if let Ok(url) = Url::parse(&self.span.text)
            && let Some(host) = url.host_str()
            && let Some(mut path) = url.path_segments()
        {
            if host == "stoat.chat"
                && path.next() == Some("invite")
                && let Some(id) = path.next()
            {
                Some(id.to_string())
            } else if host == "stt.gg"
                && let Some(id) = path.next()
            {
                Some(id.to_string())
            } else {
                None
            }
        } else {
            None
        };

        let holder = use_hook(|| ParagraphHolder::default());

        rect()
            .on_secondary_down({
                let url = self.span.text.to_string();
                move |_| {
                    provide_root_context(Some(MessageUrl(url.clone())));
                }
            })
            .child(if let Some(id) = invite_id {
                Invite { id }.into_element()
            } else {
                rect()
                    .cursor(CursorIcon::Pointer)
                    .child(
                        stoat_paragraph()
                            .holder(holder)
                            .line_height(1.5)
                            .span(self.span.clone().color(self.foreground).text_decoration(
                                if hover() {
                                    TextDecoration::Underline
                                } else {
                                    TextDecoration::None
                                },
                            ))
                            .on_press({
                                let url = self.span.text.to_string();

                                move |_| {
                                    modals
                                        .write()
                                        .push_modal(ModalValue::OpenLink { url: url.clone() });
                                }
                            })
                            .on_pointer_enter(move |_| {
                                hover.set_if_modified(true);
                            })
                            .on_pointer_leave(move |_| {
                                hover.set_if_modified(false);
                            }),
                    )
                    .into_element()
            })
    }
}
