use std::ops::Not;

use freya::{
    animation::{
        AnimNum, AnimatedValue, Ease, Function, OnChange, OnCreation,
        use_animation_with_dependencies,
    },
    prelude::*,
};

use crate::{
    SizeExt,
    components::{
        MaterialIcon, StoatButton, StoatButtonLayoutThemePartialExt, StoatColorPicker,
        material::outlined::color_lens, stoat_label,
    },
    consume_material_theme,
};

const COLOR_PALLETE: &[&[Color]] = &[
    &[
        Color::new(0xfffca5a5),
        Color::new(0xfffdba74),
        Color::new(0xfffcd34d),
        Color::new(0xff86efac),
        Color::new(0xff6ee7b7),
        Color::new(0xff67e8f9),
        Color::new(0xff93c5fd),
        Color::new(0xffc4b5fd),
        Color::new(0xfff0abfc),
        Color::new(0xfff9a8d4),
        Color::new(0xffcbd5e1),
    ],
    &[
        Color::new(0xffef4444),
        Color::new(0xfff97316),
        Color::new(0xffeab308),
        Color::new(0xff22c55e),
        Color::new(0xff10b981),
        Color::new(0xff06b6d4),
        Color::new(0xff3b82f6),
        Color::new(0xff8b5cf6),
        Color::new(0xffd946ef),
        Color::new(0xffec4899),
        Color::new(0xff64748b),
    ],
    &[
        Color::new(0xff991b1b),
        Color::new(0xff9a3412),
        Color::new(0xff854d0e),
        Color::new(0xff166534),
        Color::new(0xff065f46),
        Color::new(0xff155e75),
        Color::new(0xff1e40af),
        Color::new(0xff5b21b6),
        Color::new(0xff86198f),
        Color::new(0xff9d174d),
        Color::new(0xff334155),
    ],
];

#[derive(PartialEq)]
pub struct ColorPickerPallete {
    pub current: Option<Fill>,
    pub value: WritableSetter<Option<Color>>,
}

impl ColorPickerPallete {
    pub fn new(current: Option<Fill>, value: impl Into<WritableSetter<Option<Color>>>) -> Self {
        Self {
            current,
            value: value.into(),
        }
    }
}

impl Component for ColorPickerPallete {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let is_custom = self.current.as_ref().is_some_and(|f| {
            f.as_color()
                .is_none_or(|c| !COLOR_PALLETE.iter().any(|row| row.contains(&c)))
        });
        let is_default = self.current.is_none();

        rect()
            .spacing(8.)
            .child(rect().spacing(4.).children(COLOR_PALLETE.iter().map(|row| {
                rect()
                    .spacing(4.)
                    .horizontal()
                    .children(row.iter().map(|&color| ColorSwatch {
                        color,
                        value: self.value.clone(),
                    }))
            })))
            .child(
                rect()
                    .horizontal()
                    .width(Size::px(436.))
                    .content(Content::Flex)
                    .spacing(4.)
                    .child(
                        StoatColorPicker::new(
                            self.current
                                .as_ref()
                                .and_then(|f| f.as_color())
                                .unwrap_or(Color::WHITE),
                            {
                                let mut value = self.value.clone();
                                move |color| value.set(Some(color))
                            },
                        )
                        .width(Size::flex(1.))
                        .child(
                            rect()
                                .corner_radius(20.)
                                .background(if is_custom {
                                    theme.md.secondary_container.into()
                                } else {
                                    Color::TRANSPARENT
                                })
                                .color(if is_custom {
                                    theme.md.on_secondary_container
                                } else {
                                    theme.md.on_surface_variant
                                })
                                .border(is_custom.not().then(|| {
                                    Border::new().width(1.).fill(theme.md.outline_variant)
                                }))
                                .height(Size::px(40.))
                                .width(Size::Fill)
                                .horizontal()
                                .spacing(8.)
                                .center()
                                .child(
                                    if let Some(fill) = self.current.clone()
                                        && is_custom
                                    {
                                        rect()
                                            .size(Size::px(20.))
                                            .corner_radius(9.)
                                            .background(fill)
                                            .into_element()
                                    } else {
                                        MaterialIcon::new(color_lens())
                                            .size(Size::px(20.))
                                            .into_element()
                                    },
                                )
                                .child(stoat_label().font_weight(500).text("Custom colour")),
                        ),
                    )
                    .child(
                        StoatButton::new()
                            .corner_radius(20.)
                            .width(Size::flex(1.))
                            .on_press({
                                let mut value = self.value.clone();
                                move |_| value.set(None)
                            })
                            .child(
                                rect()
                                    .height(Size::px(40.))
                                    .width(Size::Fill)
                                    .corner_radius(20.)
                                    .background(if is_default {
                                        theme.md.secondary_container.into()
                                    } else {
                                        Color::TRANSPARENT
                                    })
                                    .color(if is_default {
                                        theme.md.on_secondary_container
                                    } else {
                                        theme.md.on_surface_variant
                                    })
                                    .border(is_default.not().then(|| {
                                        Border::new().width(1.).fill(theme.md.outline_variant)
                                    }))
                                    .horizontal()
                                    .center()
                                    .spacing(8.)
                                    .child(
                                        MaterialIcon::new(Bytes::from_static(include_bytes!(
                                            "../../assets/icons/outline.svg"
                                        )))
                                        .corner_radius(9.)
                                        .size(Size::px(18.)),
                                    )
                                    .child(stoat_label().font_weight(500).text("Default")),
                            ),
                    ),
            )
    }
}

#[derive(PartialEq)]
struct ColorSwatch {
    pub color: Color,
    pub value: WritableSetter<Option<Color>>,
}

impl Component for ColorSwatch {
    fn render(&self) -> impl IntoElement {
        let color = self.color;
        let selected = self.value.read().is_some_and(|v| v == color);

        let anim = use_animation_with_dependencies(&selected, |anim, &selected| {
            anim.on_creation(OnCreation::Finish);
            anim.on_change(OnChange::Rerun);

            let num = AnimNum::new(18., 12.)
                .time(200)
                .ease(Ease::Out)
                .function(Function::Cubic);

            if selected { num } else { num.into_reversed() }
        });

        let corner_radius = anim.read().value();

        StoatButton::new()
            .corner_radius(corner_radius)
            .on_press({
                let mut value = self.value.clone();
                move |_| value.set(Some(color.into()))
            })
            .child(rect().size(Size::px(36.)).background(color))
    }
}
