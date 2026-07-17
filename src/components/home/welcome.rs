use freya::{
    // icons::lucide::{banknote, circle_plus, compass, house, message_square_text, users},
    prelude::*,
    radio::use_radio,
};

use crate::{
    AppChannel, Selection, SettingsPage, SizeExt, components::{
        HideSidebarHeader, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt, material::{MaterialIcon, filled::{add_circle, explore, groups_3, home, payments, rate_review, settings}},
    }, consume_material_theme
};

#[derive(PartialEq)]
pub struct Welcome {}

impl Component for Welcome {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Selection);
        let selection = radio.slice_mut_current(|state| &mut state.selection);

        rect()
            .child(
                rect()
                    .horizontal()
                    .height(Size::px(48.))
                    .padding((0., 8.))
                    .margin((8., 8., 8., 0.))
                    .spacing(10.)
                    .cross_align(Alignment::Center)
                    .child(
                            HideSidebarHeader {
                                icon: home()
                            }

                    )
                    .child(label().text("Home").font_size(16))
            )
            .child(
                rect().horizontal().child(
                    rect()
                        .margin((0., 8., 8., 8.))
                        .width(Size::Fill)
                        .height(Size::Fill)
                        .corner_radius(28.)
                        .background(theme.md.surface_container_lowest.as_argb_u32())
                        .overflow(Overflow::Clip)
                        .child(
                            rect()
                                .width(Size::Fill)
                                .height(Size::Fill)
                                .padding((48., 8.))
                                .spacing(32.)
                                .center()
                                .child(label().text("Stoat").color(theme.md.on_surface.as_argb_u32()).font_size(46.))
                                .child(
                                    rect()
                                        .spacing(8.)
                                        .padding(8.)
                                        .horizontal()
                                        .child(
                                            rect()
                                                .width(Size::px(260.))
                                                .spacing(8.)
                                                .child(WelcomeButton::new(
                                                    add_circle(),
                                                    "Create a group or server",
                                                    "Invite all of your friends, some cool bots, and throw a big party.",
                                                    move |_| {}
                                                ))
                                                .child(WelcomeButton::new(
                                                    groups_3(),
                                                    "Go to the Stoat Lounge",
                                                    "You can report issues and discuss improvements with us directly here.",
                                                    move |_| {}
                                                ))
                                                .child(WelcomeButton::new(
                                                    payments(),
                                                    "Donate to Stoat",
                                                    "Support the project by donating - thank you!",
                                                    move |_| {}
                                                ))
                                        )
                                        .child(
                                            rect()
                                                .width(Size::px(260.))
                                                .spacing(8.)
                                                .child(WelcomeButton::new(
                                                    explore(),
                                                    "Discover Stoat",
                                                    "Find a community based on your hobbies or interests.",
                                                    {
                                                        let mut selection = selection.clone();

                                                        move |_| {
                                                            *selection.write() = Selection::Discover
                                                        }
                                                    }
                                                ))
                                                .child(WelcomeButton::new(
                                                    rate_review(),
                                                    "Give feedback on Stoat",
                                                    "Let us know how we can improve our app by giving us feedback.",
                                                    move |_| {}
                                                ))
                                                .child(WelcomeButton::new(
                                                    settings(),
                                                    "Open settings",
                                                    "You can also click the gear icon in the bottom left.",
                                                    {
                                                        let mut radio = radio.clone();

                                                        move |_| {
                                                            radio.write_channel(AppChannel::SettingsPage).settings_page = Some(SettingsPage::default())
                                                        }
                                                    }
                                                ))
                                        )
                                ),
                        ),
                ),
            )
    }
}

#[derive(PartialEq)]
struct WelcomeButton {
    icon: Bytes,
    title: &'static str,
    contents: &'static str,
    on_press: EventHandler<Event<PressEventData>>,
}

impl WelcomeButton {
    pub fn new(
        icon: Bytes,
        title: &'static str,
        contents: &'static str,
        on_press: impl Into<EventHandler<Event<PressEventData>>>,
    ) -> Self {
        Self {
            icon,
            title,
            contents,
            on_press: on_press.into(),
        }
    }
}

impl Component for WelcomeButton {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        StoatButton::new()
            .color(theme.md.on_secondary_container.as_argb_u32())
            .background(theme.md.secondary_container.as_argb_u32())
            .corner_radius(12.)
            .width(Size::Fill)
            .child(
                rect()
                    .width(Size::Fill)
                    .padding(13.)
                    .horizontal()
                    .spacing(16.)
                    .cross_align(Alignment::Center)
                    .child(
                        rect()
                            .background(theme.md.surface_dim.as_argb_u32())
                            .width(Size::px(36.))
                            .height(Size::px(36.))
                            .corner_radius(36.)
                            .center()
                            .child(
                                MaterialIcon::new(self.icon.clone())
                                    .size(Size::px(24.))
                                    .color(theme.md.on_surface.as_argb_u32())
                            ),
                    )
                    .child(
                        rect()
                            .spacing(2.)
                            .child(label().text(self.title).font_size(14))
                            .child(label().text(self.contents).font_size(12)),
                    ),
            )
            .on_press(self.on_press.clone())
    }
}
