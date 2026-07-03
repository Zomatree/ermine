use freya::{
    icons::lucide::{bell, circle_slash, hand, inbox, plus, users_round},
    prelude::*,
};

use crate::{
    components::{
        FriendsList, HideSidebarHeader, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt, StoatTooltip,
    },
    use_material_theme,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub enum FriendPage {
    #[default]
    Online,
    All,
    Pending,
    Blocked,
}

#[derive(PartialEq)]
pub struct Friends {}

impl Component for Friends {
    fn render(&self) -> impl IntoElement {
        let mut page = use_state(FriendPage::default);
        let theme = use_material_theme();

        rect()
            .child(
                rect()
                    .horizontal()
                    .height(Size::px(48.))
                    .padding((0., 8.))
                    .spacing(10.)
                    .margin((8., 8., 8., 0.))
                    .cross_align(Alignment::Center)
                    .child(HideSidebarHeader {
                        icon: users_round(),
                    })
                    .child(label().text("Friends").font_size(16)),
            )
            .child(
                rect().horizontal().child(
                    rect()
                        .margin((0., 8., 8., 8.))
                        .padding((0., 8.))
                        .width(Size::Fill)
                        .height(Size::Fill)
                        .corner_radius(28.)
                        .background(theme.md.surface_container_lowest.as_argb_u32())
                        .overflow(Overflow::Clip)
                        .child(
                            rect()
                                .horizontal()
                                .child(
                                    rect()
                                        .padding((8., 0.))
                                        .width(Size::px(56.))
                                        .cross_align(Alignment::Center)
                                        .child(
                                            StoatTooltip::new(
                                                label()
                                                    .max_lines(1)
                                                    .font_size(11.)
                                                    .text("Add a new friend"),
                                            )
                                            .position(AttachedPosition::Right)
                                            .child(
                                                StoatButton::new()
                                                    .margin((6., 0., 12., 0.))
                                                    .corner_radius(12.)
                                                    .child(
                                                        rect()
                                                            .width(Size::px(40.))
                                                            .height(Size::px(40.))
                                                            .background(
                                                                theme.md.primary.as_argb_u32(),
                                                            )
                                                            .color(
                                                                theme.md.on_primary.as_argb_u32(),
                                                            )
                                                            .center()
                                                            .child(
                                                                SvgViewer::new(plus())
                                                                    .width(Size::px(24.))
                                                                    .height(Size::px(24.)),
                                                            ),
                                                    ),
                                            ),
                                        )
                                        .children(
                                            [
                                                (hand(), "Online", FriendPage::Online),
                                                (inbox(), "All", FriendPage::All),
                                                (bell(), "Pending", FriendPage::Pending),
                                                (circle_slash(), "Blocked", FriendPage::Blocked),
                                            ]
                                            .into_iter()
                                            .map(
                                                |(icon, title, value)| {
                                                    StoatButton::new()
                                                        .corner_radius(16.)
                                                        .maybe(*page.read() == value, |this| {
                                                            this.background(
                                                                theme
                                                                    .md
                                                                    .secondary_container
                                                                    .as_argb_u32(),
                                                            )
                                                        })
                                                        .on_press(move |_| {
                                                            page.set_if_modified(value)
                                                        })
                                                        .child(
                                                            rect()
                                                                .cross_align(Alignment::Center)
                                                                .child(
                                                                    rect()
                                                                        .center()
                                                                        .width(Size::px(56.))
                                                                        .height(Size::px(32.))
                                                                        .child(
                                                                            SvgViewer::new(icon)
                                                                                .width(Size::px(
                                                                                    24.,
                                                                                ))
                                                                                .height(Size::px(
                                                                                    24.,
                                                                                )),
                                                                        ),
                                                                )
                                                                .child(
                                                                    rect()
                                                                        .width(Size::px(56.))
                                                                        .height(Size::px(24.))
                                                                        .center()
                                                                        .child(
                                                                            label()
                                                                                .text(title)
                                                                                .font_size(12),
                                                                        ),
                                                                ),
                                                        )
                                                        .into_element()
                                                },
                                            ),
                                        ), // .child(Button::new().child("Online").on_press(move |_| {
                                           //     page.set_if_modified(FriendPage::Online)
                                           // }))
                                           // .child(Button::new().child("All").on_press(move |_| {
                                           //     page.set_if_modified(FriendPage::All)
                                           // }))
                                           // .child(Button::new().child("Pending").on_press(move |_| {
                                           //     page.set_if_modified(FriendPage::Pending)
                                           // }))
                                           // .child(Button::new().child("Blocked").on_press(
                                           //     move |_| page.set_if_modified(FriendPage::Blocked),
                                           // )),
                                )
                                .child(FriendsList { page }),
                        ),
                ),
            )
    }
}
