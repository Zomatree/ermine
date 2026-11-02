use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel, Selection, SettingsPage, SizeExt,
    components::{
        CurrentUserButton, HomeButton, ModalValue, ReorderServers, StoatButton,
        StoatButtonLayoutThemePartialExt, StoatTooltip,
        material::{
            MaterialIcon,
            filled::{add, explore, settings},
        },
        stoat_scrollbar, use_modals,
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct ServerList {}

impl Component for ServerList {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::Servers);
        let theme = consume_material_theme();
        let mut modals = use_modals();

        rect()
            .child(
                ScrollView::new()
                    .scrollbar(stoat_scrollbar)
                    .child(
                        rect()
                            .width(Size::fill())
                            .cross_align(Alignment::Center)
                            .child(HomeButton {})
                            .child(CurrentUserButton {})
                            .child(
                                rect()
                                    .height(Size::px(1.))
                                    .width(Size::px(32.))
                                    .margin((6., 0.))
                                    .background(theme.md.outline_variant.as_argb_u32()),
                            )
                            .child(ReorderServers {})
                            .child(
                                StoatTooltip::new(
                                    label()
                                        .max_lines(1)
                                        .font_size(11.)
                                        .text("Create or join a server"),
                                )
                                .position(AttachedPosition::Right)
                                .child(
                                    rect()
                                        .width(Size::px(56.))
                                        .height(Size::px(56.))
                                        .center()
                                        .child(
                                            StoatButton::new()
                                                .corner_radius(42.)
                                                .child(
                                                    rect()
                                                        .center()
                                                        .width(Size::px(42.0))
                                                        .height(Size::px(42.0))
                                                        .background(
                                                            theme
                                                                .md
                                                                .surface_container_low
                                                                .as_argb_u32(),
                                                        )
                                                        .child(
                                                            MaterialIcon::new(add())
                                                                .size(Size::px(32.0)),
                                                        ),
                                                )
                                                .on_press(move |_| {
                                                    modals
                                                        .write()
                                                        .push_modal(ModalValue::CreateJoinServer);
                                                }),
                                        ),
                                ),
                            )
                            .child(
                                StoatTooltip::new(
                                    label()
                                        .max_lines(1)
                                        .font_size(11.)
                                        .text("Find new servers to join"),
                                )
                                .position(AttachedPosition::Right)
                                .child(
                                    rect()
                                        .width(Size::px(56.))
                                        .height(Size::px(56.))
                                        .center()
                                        .child(
                                            StoatButton::new()
                                                .corner_radius(42.)
                                                .child(
                                                    rect()
                                                        .center()
                                                        .width(Size::px(42.0))
                                                        .height(Size::px(42.0))
                                                        .background(
                                                            theme
                                                                .md
                                                                .surface_container_low
                                                                .as_argb_u32(),
                                                        )
                                                        .child(
                                                            MaterialIcon::new(explore())
                                                                .width(Size::px(32.0))
                                                                .height(Size::px(32.0)),
                                                        ),
                                                )
                                                .on_press(move |_| {
                                                    radio
                                                        .write_channel(AppChannel::Selection)
                                                        .selection = Selection::Discover;
                                                }),
                                        ),
                                ),
                            ),
                    )
                    .show_scrollbar(false)
                    .width(Size::px(56.))
                    .height(Size::func(|size| Some(size.parent - 56.))),
            )
            .child(
                StoatTooltip::new(label().max_lines(1).font_size(11.).text("Settings"))
                    .position(AttachedPosition::Right)
                    .child(
                        rect()
                            .width(Size::px(56.))
                            .height(Size::px(56.))
                            .center()
                            .child(
                                StoatButton::new()
                                    .corner_radius(42.)
                                    .child(
                                        rect()
                                            .center()
                                            .width(Size::px(42.0))
                                            .height(Size::px(42.0))
                                            .overflow(Overflow::Clip)
                                            .background(
                                                theme.md.surface_container_low.as_argb_u32(),
                                            )
                                            .child(
                                                MaterialIcon::new(settings()).size(Size::px(32.0)),
                                            ),
                                    )
                                    .on_press(move |_| {
                                        radio
                                            .write_channel(AppChannel::SettingsPage)
                                            .settings_page = Some(SettingsPage::default());
                                    }),
                            ),
                    ),
            )
    }
}
