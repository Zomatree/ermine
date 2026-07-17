use std::borrow::Cow;

use bytes::Bytes;
use freya::prelude::*;

mod category;
mod channel;
mod message;
mod server;
mod user;

pub use category::*;
pub use channel::*;
pub use message::*;
pub use server::*;
pub use user::*;

use crate::{
    SizeExt,
    components::{
        MaterialIcon, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt,
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct ContextMenuButton {
    icon: Bytes,
    title: Cow<'static, str>,
    danger: bool,
    on_press: Option<EventHandler<Event<PressEventData>>>,
}

impl ContextMenuButton {
    pub fn new(icon: Bytes, title: impl Into<Cow<'static, str>>) -> Self {
        Self {
            icon,
            title: title.into(),
            danger: false,
            on_press: None,
        }
    }

    pub fn danger(mut self) -> Self {
        self.danger = true;

        self
    }

    pub fn on_press(mut self, on_press: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_press = Some(on_press.into());
        self
    }
}

impl Component for ContextMenuButton {
    fn render(&self) -> impl IntoElement {
        StoatButton::new()
            .width(Size::FillMinimum)
            .map(self.on_press.clone(), |btn, on_press| {
                btn.on_press(on_press)
            })
            .maybe(self.danger, |btn| {
                btn.color(consume_material_theme().md.error.as_argb_u32())
            })
            .child(
                rect()
                    .height(Size::px(36.))
                    .padding((8., 15.))
                    .horizontal()
                    .spacing(8.)
                    .cross_align(Alignment::Center)
                    .child(MaterialIcon::new(self.icon.clone()).size(Size::px(16.)))
                    .child(label().text(self.title.clone())),
            )
    }
}
