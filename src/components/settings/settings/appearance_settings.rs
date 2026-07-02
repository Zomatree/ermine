use freya::prelude::*;

use crate::{
    ThemeScheme,
    components::{StoatColorPicker, StoatSegmentedButton},
    use_config,
};

#[derive(PartialEq)]
pub struct AppearanceSettings {}

impl Component for AppearanceSettings {
    fn render(&self) -> impl IntoElement {
        let mut config = use_config();

        let scheme = use_state(|| config.read().theme.scheme);
        let mut source = use_state(|| {
            println!("new source");
            Color::new(config.read().theme.theme_source | (0xFF << 24))
        });

        use_side_effect(move || {
            let scheme = *scheme.read();

            config.write().theme.scheme = scheme;
        });

        use_side_effect(move || {
            let source = *source.read();

            config.write().theme.theme_source =
                ((source.r() as u32) << 16) | ((source.g() as u32) << 8) | (source.b() as u32);
        });

        rect()
            .spacing(8.)
            .child(
                StoatSegmentedButton::new(
                    scheme,
                    vec![ThemeScheme::Light, ThemeScheme::Dark],
                    |scheme| {
                        match scheme {
                            ThemeScheme::Light => "Light",
                            ThemeScheme::Dark => "Dark",
                        }
                        .into_element()
                    },
                )
                .height(40.),
            )
            .child(
                rect()
                    .horizontal()
                    .width(Size::Fill)
                    .center()
                    .spacing(8.)
                    .child(StoatColorPicker::new(
                        Color::new(config.read().theme.theme_source | (0xFF << 24)),
                        move |c: Color| {
                            source.set(c);
                        },
                    ))
                    .child(
                        rect().child(
                            StoatSegmentedButton::new(
                                source,
                                vec![
                                    Color::new(0xffff5733),
                                    Color::new(0xffffdc2f),
                                    Color::new(0xff9bf088),
                                    Color::new(0xff54ecc1),
                                    Color::new(0xff549bec),
                                    Color::new(0xff5470ec),
                                    Color::new(0xff8c5fd3),
                                ],
                                |color| rect().expanded().background(color.clone()).into_element(),
                            )
                            .height(56.),
                        ),
                    ),
            )
    }
}
