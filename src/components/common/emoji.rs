use freya::prelude::*;

use crate::{Tag, components::AnimatedImage, http};

#[derive(PartialEq)]
pub struct Emoji {
    value: String,

    layout: LayoutData,
}

impl Emoji {
    pub fn new(value: String) -> Self {
        Self {
            value,
            layout: LayoutData::default(),
        }
    }
}

impl LayoutExt for Emoji {
    fn get_layout(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl ContainerSizeExt for Emoji {}

impl Component for Emoji {
    fn render(&self) -> impl IntoElement {
        if self.value.len() == 26 {
            let url = format!(
                "{}/{}/{}",
                http().api_config.features.autumn.url,
                Tag::Emojis,
                &self.value
            );

            AnimatedImage::new(url.parse::<Url>().unwrap())
                .sampling_mode(SamplingMode::Trilinear)
                .layout(self.layout.clone())
                .into_element()
        } else {
            let codes = self
                .value
                .chars()
                .map(|c| format!("{:x}", c as i32))
                .collect::<Vec<String>>()
                .join("-");

            let url = format!("https://static.stoat.chat/emoji/fluent-3d/{codes}.svg?v=1");

            SvgViewer::new(url.parse::<Url>().unwrap())
                .parallel(true)
                .layout(self.layout.clone())
                .error_renderer({
                    let layout = self.layout.clone();
                    move |_| rect().layout(layout.clone()).into_element()
                })
                .into_element()
        }
    }
}
