use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel, SizeExt,
    components::{AnimatedImage, StoatTooltip},
    http,
};

#[derive(PartialEq)]
pub struct Emoji {
    pub id: String,
    pub font_size: f32,
}

impl Component for Emoji {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Emojis);

        let mut emoji = use_state(|| radio.read().emojis.get(&self.id).cloned());
        let mut fetching = use_state(|| false);

        let size = self.font_size * 1.4;
        let url = format!("https://cdn.stoatusercontent.com/emojis/{}", &self.id)
            .parse::<Url>()
            .unwrap();

        StoatTooltip::new(
            rect()
                .on_sized({
                    let id = self.id.clone();

                    move |_| {
                        if !fetching() && emoji.read().is_none() {
                            fetching.set(true);

                            let id = id.clone();

                            spawn(async move {
                                if let Ok(response) = http().fetch_emoji(&id).await {
                                    emoji.set(Some(response));
                                }
                            });
                        };
                    }
                })
                .child(if let Some(emoji) = &*emoji.read() {
                    rect()
                        .horizontal()
                        .spacing(15.)
                        .cross_align(Alignment::Center)
                        .child(
                            AnimatedImage::new(url.clone())
                                .size(Size::px(33.))
                                .sampling_mode(SamplingMode::Trilinear)
                                .aspect_ratio(AspectRatio::Min)
                                .image_cover(ImageCover::Center),
                        )
                        .child(
                            label()
                                .color(Color::WHITE)
                                .font_size(11.)
                                .text(format!(":{}:", emoji.name)),
                        )
                        .into_element()
                } else {
                    CircularLoader::new().size(33.).into_element()
                }),
        )
        .position(AttachedPosition::Top)
        .child(
            rect().padding((0., 0.7, 0., 1.4)).child(
                AnimatedImage::new(url)
                    .sampling_mode(SamplingMode::Trilinear)
                    .aspect_ratio(AspectRatio::Min)
                    .image_cover(ImageCover::Center)
                    .size(Size::px(size)),
            ),
        )
    }
}
