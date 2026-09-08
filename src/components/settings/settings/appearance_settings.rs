use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;
use ulid::Ulid;

use crate::{
    AppChannel, ThemeScheme, ThemeSet, ThemeVariant, components::{
        Dropdown, Message, MessageModel, StoatColorPicker, StoatSegmentedButton,
        checkbox::StoatCheckbox,
    }, consume_material_theme, use_config
};

#[derive(PartialEq)]
pub struct AppearanceSettings {}

impl Component for AppearanceSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Settings("ermine"));

        let ermine_settings =
            radio.slice_mut_current(|state| state.settings.ermine.get_or_insert_default());

        let hide_pronouns = ermine_settings.clone().into_writable().map(
            |settings| &settings.hide_pronouns,
            |settings| &mut settings.hide_pronouns,
        );
        let message_group_spacing = ermine_settings.into_writable().map(
            |settings| &settings.message_group_spacing,
            |settings| &mut settings.message_group_spacing,
        );

        let mut config = use_config();

        let scheme = use_state(|| config.read().theme.scheme);
        let variant = use_state(|| config.read().theme.variant);
        let mut source = use_state(|| Color::new(config.read().theme.theme_source | (0xFF << 24)));
        let code_theme = config.into_writable().map(
            |config| &config.theme.code_theme,
            |config| &mut config.theme.code_theme,
        );

        use_side_effect(move || {
            let mut config = config.write();
            config.theme.scheme = scheme();
            config.theme.variant = variant();
        });

        use_side_effect(move || {
            let source = *source.read();

            config.write().theme.theme_source =
                ((source.r() as u32) << 16) | ((source.g() as u32) << 8) | (source.b() as u32);
        });

        let theme_set = consume_root_context::<ThemeSet>();

        let id_1 = use_hook(|| Ulid::new());
        let id_2 = use_hook(|| Ulid::new());

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
                    )
                    .child(
                        StoatSegmentedButton::new(
                            variant,
                            vec![
                                ThemeVariant::Monochrome,
                                ThemeVariant::Neutral,
                                ThemeVariant::TonalSpot,
                                ThemeVariant::Vibrant,
                                ThemeVariant::Content,
                                ThemeVariant::Rainbow,
                                ThemeVariant::FruitSalad,
                            ],
                            |variant| {
                                paragraph()
                                    // .width(Size::Inner)
                                    .max_lines(1)
                                    .text_overflow(TextOverflow::Ellipsis)
                                    .font_size(10.)
                                    .span(match variant {
                                        ThemeVariant::Monochrome => "Monochrome",
                                        ThemeVariant::Neutral => "Neutral",
                                        ThemeVariant::TonalSpot => "Tonal Spot",
                                        ThemeVariant::Vibrant => "Vibrant",
                                        ThemeVariant::Expressive => "Expressive",
                                        ThemeVariant::Fidelity => "Fidelity",
                                        ThemeVariant::Content => "Content",
                                        ThemeVariant::Rainbow => "Rainbow",
                                        ThemeVariant::FruitSalad => "Fruit Salad",
                                    })
                                    .into_element()
                            },
                        )
                        .height(32.),
                    ),
            )
            .child(label().text("Pronouns").font_size(14.).font_weight(600))
            .child(
                rect().child(
                    StoatCheckbox::from_writable(hide_pronouns).child("Hide pronouns in chat"),
                ),
            )
            .child(
                label()
                    .text("Display & Text")
                    .font_size(14.)
                    .font_weight(600),
            )
            .child({
                let user_id = radio.peek_state().user_id.clone().unwrap();

                let user = radio
                    .slice(AppChannel::Users, move |state| {
                        state.users.get(state.user_id.as_ref().unwrap()).unwrap()
                    })
                    .into_readable();

                let channel = v0::Channel::SavedMessages {
                    id: "0".to_string(),
                    user: user_id.clone(),
                }
                .into_readable();

                rect()
                    .padding(8.)
                    .background(theme.md.surface_container_lowest.as_argb_u32())
                    .width(Size::Fill)
                    .height(Size::px(126.))
                    .corner_radius(16.)
                    .interactive(false)
                    .spacing(*message_group_spacing.read() as f32)
                    .child(Message {
                        channel: channel.clone(),
                        message: MessageModel {
                            message: v0::Message {
                                id: id_1.to_string(),
                                nonce: None,
                                channel: "0".to_string(),
                                author: user_id.clone(),
                                user: None,
                                member: None,
                                webhook: None,
                                content: Some("Hello World!".to_string()),
                                system: None,
                                attachments: None,
                                edited: None,
                                embeds: None,
                                mentions: None,
                                role_mentions: None,
                                replies: None,
                                reactions: Default::default(),
                                interactions: Default::default(),
                                masquerade: None,
                                pinned: None,
                                flags: 0,
                            },
                            user: user.clone(),
                            member: None,
                        },
                    })
                    .child(Message {
                        channel: channel.clone(),
                        message: MessageModel {
                            message: v0::Message {
                                id: id_2.to_string(),
                                nonce: None,
                                channel: "0".to_string(),
                                author: user_id.clone(),
                                user: None,
                                member: None,
                                webhook: None,
                                content: Some("`Lorem ipsum`".to_string()),
                                system: None,
                                attachments: None,
                                edited: None,
                                embeds: None,
                                mentions: None,
                                role_mentions: None,
                                replies: None,
                                reactions: Default::default(),
                                interactions: Default::default(),
                                masquerade: None,
                                pinned: None,
                                flags: 0,
                            },
                            user: user.clone(),
                            member: None,
                        },
                    })
            })
            .child(label().text("Message Group Spacing").font_size(12.))
            .child(
                rect()
                    .horizontal()
                    .spacing(8.)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .font_size(12.)
                            .color(theme.md.on_surface_variant.as_argb_u32())
                            .text(format!("{}px", message_group_spacing.read())),
                    )
                    .child(
                        Slider::new({
                            let mut message_group_spacing = message_group_spacing.clone();
                            move |value| {
                                message_group_spacing.set((16. * (value / 100.)) as u32);
                            }
                        })
                        .scroll_enabled(false)
                        .value(((*message_group_spacing.read() as f64) / 16.) * 100.)
                        .step(100. / 16.)
                        .cursor_icon(CursorIcon::Pointer)
                        .border_fill(Color::TRANSPARENT)
                        .background(theme.md.surface_container_highest.as_argb_u32())
                        .thumb_background(theme.md.primary.as_argb_u32())
                        .thumb_inner_background(theme.md.primary.as_argb_u32()),
                    ),
            )
            .child(Dropdown::new(
                "Codeblock Theme",
                code_theme,
                theme_set.themes.keys().cloned().collect(),
                |theme| {
                    match theme.as_str() {
                        "OneHalfDark" => "One Half (dark)",
                        "OneHalfLight" => "One Half (light)",
                        "CatppuccinMacchiato" => "Catppuccin Macchiato (dark)",
                        "base16-ocean.dark" => "Base16 Ocean (dark)",
                        "base16-eighties.dark" => "Base16 Eighties (dark)",
                        "base16-mocha.dark" => "Base16 Mocha (dark)",
                        "base16-ocean.light" => "Base16 Ocean (light)",
                        "InspiredGitHub" => "Github (light)",
                        "Solarized (dark)" => "Solarized (dark)",
                        "Solarized (light)" => "Solarized (light)",
                        name => name,
                    }
                    .into_element()
                },
            ))
    }
}
