use freya::prelude::*;

use crate::{
    components::{EmojiPicker, GifPicker, StoatSegmentedButton},
    consume_material_theme,
};

#[derive(PartialEq, Clone, Copy, Default, Hash, Debug)]
pub enum PickerSelection {
    GIF,
    #[default]
    Emoji,
}

#[derive(PartialEq)]
pub struct EmojiGifPicker {
    pub initial: PickerSelection,
    pub callback: EventHandler<(String, PickerSelection)>,
}

impl EmojiGifPicker {
    pub fn new(
        initial: PickerSelection,
        callback: impl Into<EventHandler<(String, PickerSelection)>>,
    ) -> Self {
        Self {
            initial,
            callback: callback.into(),
        }
    }
}

impl Component for EmojiGifPicker {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let selection = use_state(|| self.initial);

        rect()
            .padding((8., 0.))
            .width(Size::px(400.))
            .corner_radius(16.)
            .spacing(8.)
            .background(theme.md.surface_container.as_argb_u32())
            .shadow(Shadow::new().blur(3.).color(theme.md.shadow.as_argb_u32()))
            .cross_align(Alignment::Center)
            .child(
                rect().width(Size::px(140.)).child(
                    StoatSegmentedButton::new(
                        selection,
                        vec![PickerSelection::GIF, PickerSelection::Emoji],
                        |selection| {
                            match selection {
                                PickerSelection::GIF => "GIFs",
                                PickerSelection::Emoji => "Emoji",
                            }
                            .into_element()
                        },
                    )
                    .height(40.),
                ),
            )
            .child(match selection() {
                PickerSelection::GIF => GifPicker::new({
                    let callback = self.callback.clone();
                    move |url| callback.call((url, PickerSelection::GIF))
                })
                .into_element(),
                PickerSelection::Emoji => EmojiPicker::new({
                    let callback = self.callback.clone();
                    move |url| callback.call((url, PickerSelection::Emoji))
                })
                .into_element(),
            })
    }
}
