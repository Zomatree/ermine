use freya::{
    prelude::*,
};

use crate::{
    SizeExt, components::{MaterialIcon, StoatButton, StoatButtonLayoutThemePartialExt, material::{filled::{list, memory}, outlined::{bug_report, open_in_new}}}, consume_material_theme
};

#[derive(PartialEq)]
pub struct SourceCodeSettings {}

impl Component for SourceCodeSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        rect()
            .vertical()
            .spacing(2.)
            .corner_radius(28.)
            .overflow(Overflow::Clip)
            .child(
                StoatButton::new()
                    .corner_radius(12.)
                    .on_press(move |_| {
                        open::that_in_background("https://github.com/zomatree/ermine");
                    })
                    .child(
                        rect()
                            .content(Content::Flex)
                            .cross_align(Alignment::Center)
                            .background(theme.md.secondary_container.as_argb_u32())
                            .color(theme.md.secondary.as_argb_u32())
                            .corner_radius(12.)
                            .padding(13.)
                            .horizontal()
                            .spacing(16.)
                            .child(
                                rect()
                                    .width(Size::px(36.))
                                    .height(Size::px(36.))
                                    .corner_radius(18.)
                                    .background(theme.md.surface_dim.as_argb_u32())
                                    .center()
                                    .child(MaterialIcon::new(memory()).size(Size::px(22.))),
                            )
                            .child(
                                rect()
                                    .width(Size::flex(1.))
                                    .child(label().font_size(14.).text("View Ermine source code"))
                                    .child(
                                        label().font_size(12.).text("Ermine client source code."),
                                    ),
                            )
                            .child(
                                MaterialIcon::new(open_in_new())
                                    .size(Size::px(18.))
                            ),
                    ),
            )
            .child(
                StoatButton::new()
                    .corner_radius(12.)
                    .on_press(move |_| {
                        open::that_in_background("https://github.com/Zomatree/ermine/discussions");
                    })
                    .child(
                        rect()
                            .content(Content::Flex)
                            .cross_align(Alignment::Center)
                            .background(theme.md.secondary_container.as_argb_u32())
                            .color(theme.md.secondary.as_argb_u32())
                            .corner_radius(12.)
                            .padding(13.)
                            .horizontal()
                            .spacing(16.)
                            .child(
                                rect()
                                    .width(Size::px(36.))
                                    .height(Size::px(36.))
                                    .corner_radius(18.)
                                    .background(theme.md.surface_dim.as_argb_u32())
                                    .center()
                                    .child(
                                        MaterialIcon::new(list())
                                            .size(Size::px(22.))
                                    ),
                            )
                            .child(
                                rect()
                                    .width(Size::flex(1.))
                                    .child(label().font_size(14.).text("Feedback"))
                                    .child(label().font_size(12.).text("Submit feedback.")),
                            )
                            .child(
                                MaterialIcon::new(open_in_new())
                                    .size(Size::px(18.))
                            ),
                    ),
            )
            .child(
                StoatButton::new()
                    .corner_radius(12.)
                    .on_press(move |_| {
                        open::that_in_background("https://github.com/Zomatree/ermine/issues");
                    })
                    .child(
                        rect()
                            .content(Content::Flex)
                            .cross_align(Alignment::Center)
                            .background(theme.md.secondary_container.as_argb_u32())
                            .color(theme.md.secondary.as_argb_u32())
                            .corner_radius(12.)
                            .padding(13.)
                            .horizontal()
                            .spacing(16.)
                            .child(
                                rect()
                                    .width(Size::px(36.))
                                    .height(Size::px(36.))
                                    .corner_radius(18.)
                                    .background(theme.md.surface_dim.as_argb_u32())
                                    .center()
                                    .child(MaterialIcon::new(bug_report()).size(Size::px(22.))),
                            )
                            .child(
                                rect()
                                    .width(Size::flex(1.))
                                    .child(label().font_size(14.).text("Bug Tracker"))
                                    .child(
                                        label()
                                            .font_size(12.)
                                            .text("View currently active bug reports."),
                                    ),
                            )
                            .child(
                                MaterialIcon::new(open_in_new())
                                    .size(Size::px(18.))
                            ),
                    ),
            )
    }
}
