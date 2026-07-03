use freya::{
    icons::lucide::{bug, cpu, external_link, list_ordered},
    prelude::*,
};

use crate::{
    components::{StoatButton, StoatButtonLayoutThemePartialExt},
    use_material_theme,
};

#[derive(PartialEq)]
pub struct SourceCodeSettings {}

impl Component for SourceCodeSettings {
    fn render(&self) -> impl IntoElement {
        let theme = use_material_theme();

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
                                    .child(SvgViewer::new(cpu()).width(Size::px(22.)).height(Size::px(22.))),
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
                                SvgViewer::new(external_link())
                                    .width(Size::px(18.))
                                    .height(Size::px(18.)),
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
                                        SvgViewer::new(list_ordered())
                                            .width(Size::px(22.))
                                            .height(Size::px(22.)),
                                    ),
                            )
                            .child(
                                rect()
                                    .width(Size::flex(1.))
                                    .child(label().font_size(14.).text("Feedback"))
                                    .child(label().font_size(12.).text("Submit feedback.")),
                            )
                            .child(
                                SvgViewer::new(external_link())
                                    .width(Size::px(18.))
                                    .height(Size::px(18.)),
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
                                    .child(SvgViewer::new(bug()).width(Size::px(22.)).height(Size::px(22.))),
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
                                SvgViewer::new(external_link())
                                    .width(Size::px(18.))
                                    .height(Size::px(18.)),
                            ),
                    ),
            )
    }
}
