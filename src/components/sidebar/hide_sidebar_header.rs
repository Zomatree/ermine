use freya::{icons::lucide::chevron_left, prelude::*};

use crate::{
    components::{StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip},
    use_config,
};

#[derive(PartialEq)]
pub struct HideSidebarHeader {
    pub icon: Bytes,
}

impl Component for HideSidebarHeader {
    fn render(&self) -> impl IntoElement {
        let mut config = use_config();

        let hide_channel_list = config.read().hide_channel_list;

        StoatTooltip::new(
            label()
                .max_lines(1)
                .font_size(11.)
                .text("Toggle main sidebar"),
        )
        .child(
            StoatButton::new()
                .corner_radius(12.)
                .on_press(move |_| {
                    config.write().hide_channel_list = !hide_channel_list;
                })
                .child(
                    rect()
                        .center()
                        .height(Size::px(40.))
                        .padding((0., 4.))
                        .horizontal()
                        .child(
                            SvgViewer::new(chevron_left())
                                .width(Size::px(18.))
                                .height(Size::px(18.))
                                .rotation(if hide_channel_list { 180. } else { 0. }),
                        )
                        .child(
                            SvgViewer::new(self.icon.clone())
                                .width(Size::px(24.))
                                .height(Size::px(24.)),
                        ),
                ),
        )
    }
}
