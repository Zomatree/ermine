use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel, ThemeScheme,
    components::{StoatColorPicker, StoatSegmentedButton, checkbox::StoatCheckbox},
    use_config,
};

#[derive(PartialEq)]
pub struct AppearanceSettings {}

impl Component for AppearanceSettings {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Settings("ermine"));
        let hide_pronouns = radio.slice_mut_current(|state| {
            &mut state.settings.ermine.get_or_insert_default().hide_pronouns
        });

        let mut config = use_config();

        let scheme = use_state(|| config.read().theme.scheme);
        let mut source = use_state(|| Color::new(config.read().theme.theme_source | (0xFF << 24)));

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
            .spacing(15.)
            .child(
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
                                        |color| {
                                            rect()
                                                .expanded()
                                                .background(color.clone())
                                                .into_element()
                                        },
                                    )
                                    .height(56.),
                                ),
                            ),
                    ),
            )
            .child(label().text("Pronouns").font_size(12.))
            .child(rect().child(StoatCheckbox::new(hide_pronouns).child("Hide pronouns in chat")))
    }
}
