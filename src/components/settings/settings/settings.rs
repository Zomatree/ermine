use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel, SettingsPage, SizeExt,
    components::{
        AccountSettings, AppearanceSettings, Avatar, MaterialIcon, ProfileSettings,
        SourceCodeSettings, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt,
        material::filled::{clear, delete},
    },
    consume_material_theme,
    theme::Theme,
    use_config,
};

#[derive(PartialEq)]
pub struct Settings {}

impl Component for Settings {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::SettingsPage);
        let current_page = radio.slice_mut_current(|state| &mut state.settings_page);
        let theme = consume_material_theme();

        let close_settings = {
            let current_page = current_page.clone();
            move || {
                *current_page.clone().write() = None;
            }
        };

        let mut window_size = use_state(Area::default);

        let fullscreen = (window_size.read().width() != 0. && window_size.read().width() <= 1000.)
            || (window_size.read().height() != 0. && window_size.read().height() <= 500.);

        rect()
            .expanded()
            .center()
            .background(0x99000000)
            .on_press({
                let close_settings = close_settings.clone();
                move |_| close_settings()
            })
            .on_global_key_down({
                let close_settings = close_settings.clone();
                move |e: Event<KeyboardEventData>| {
                    if e.key == Key::Named(NamedKey::Escape) {
                        close_settings()
                    }
                }
            })
            .on_sized(move |e: Event<SizedEventData>| window_size.set_if_modified(e.area))
            .child(
                rect()
                    .on_press(|e: Event<PressEventData>| e.stop_propagation())
                    .corner_radius(if !fullscreen { 16. } else { 0. })
                    .overflow(Overflow::Clip)
                    .background(theme.md.surface_container_highest.as_argb_u32())
                    .horizontal()
                    .height(Size::func_data(
                        move |size| {
                            if !fullscreen {
                                Some((size.parent - 100.).max(300.).min(size.parent))
                            } else {
                                Some(size.parent)
                            }
                        },
                        &fullscreen,
                    ))
                    .child(
                        ScrollView::new()
                            .height(Size::Fill)
                            .width(Size::px(230.))
                            .child(
                                rect()
                                    .padding((24., 16., 16., 16.))
                                    .spacing(15.)
                                    .child(MyAccountButton {})
                                    .child(settings_category(
                                        "USER SETTINGS",
                                        &theme,
                                        &[SettingsPage::Profile, SettingsPage::Sessions],
                                    ))
                                    .child(settings_category(
                                        "STOAT",
                                        &theme,
                                        &[SettingsPage::MyBots, SettingsPage::Feedback],
                                    ))
                                    .child(settings_category(
                                        "CLIENT SETTINGS",
                                        &theme,
                                        &[
                                            SettingsPage::Voice,
                                            SettingsPage::Appearance,
                                            SettingsPage::Language,
                                        ],
                                    ))
                                    .child(settings_category(
                                        "MISC",
                                        &theme,
                                        &[
                                            SettingsPage::SourceCode,
                                            SettingsPage::Advanced,
                                            SettingsPage::Donate,
                                        ],
                                    ))
                                    .child(LogoutButton {}),
                            ),
                    )
                    .child(
                        rect()
                            .corner_radius(CornerRadius {
                                top_left: 16.,
                                top_right: 0.,
                                bottom_right: 0.,
                                bottom_left: 16.,
                                smoothing: 0.,
                            })
                            .background(theme.md.surface_container_low.as_argb_u32())
                            .horizontal()
                            .content(Content::Flex)
                            .child(
                                ScrollView::new()
                                    .width(Size::flex(1.))
                                    .max_width(Size::px(740.))
                                    .child(rect().padding((32., 32.)).maybe_child(
                                        current_page.read().clone().map(|page| {
                                            rect()
                                                .spacing(8.)
                                                .child(
                                                    label()
                                                        .text(page.title())
                                                        .font_size(22)
                                                        .line_height(1.75)
                                                        .font_weight(550),
                                                )
                                                .child(match page {
                                                    SettingsPage::Account => {
                                                        AccountSettings {}.into_element()
                                                    }
                                                    SettingsPage::Profile => {
                                                        ProfileSettings {}.into_element()
                                                    }
                                                    SettingsPage::Sessions => {
                                                        "Coming soon!".into_element()
                                                    }
                                                    SettingsPage::MyBots => {
                                                        "Coming soon!".into_element()
                                                    }
                                                    SettingsPage::Feedback => {
                                                        "Coming soon!".into_element()
                                                    }
                                                    SettingsPage::Voice => {
                                                        "Coming soon!".into_element()
                                                    }
                                                    SettingsPage::Appearance => {
                                                        AppearanceSettings {}.into_element()
                                                    }
                                                    SettingsPage::Language => {
                                                        "Coming soon!".into_element()
                                                    }
                                                    SettingsPage::SourceCode => {
                                                        SourceCodeSettings {}.into_element()
                                                    }
                                                    SettingsPage::Advanced => {
                                                        "Coming soon!".into_element()
                                                    }
                                                    SettingsPage::Donate => {
                                                        "Coming soon!".into_element()
                                                    }
                                                })
                                        }),
                                    )),
                            )
                            .child(
                                rect().padding((32., 32.)).child(
                                    StoatButton::new()
                                        .corner_radius(40.)
                                        .background(theme.md.secondary_container.as_argb_u32())
                                        .color(theme.md.on_secondary_container.as_argb_u32())
                                        .on_press(move |_| close_settings())
                                        .child(
                                            rect()
                                                .center()
                                                .width(Size::px(40.))
                                                .height(Size::px(40.))
                                                .child(
                                                    MaterialIcon::new(clear()).size(Size::px(24.)),
                                                ),
                                        ),
                                ),
                            ),
                    ),
            )
    }
}

#[derive(PartialEq)]
struct MyAccountButton {}

impl Component for MyAccountButton {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let theme = consume_material_theme();

        let current_page =
            radio.slice_mut(AppChannel::SettingsPage, |state| &mut state.settings_page);
        let current_user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());
        let current_user = radio
            .slice(AppChannel::Users, move |state| {
                state.users.get(&*current_user_id.read()).unwrap()
            })
            .into_readable();

        StoatButton::new()
            .margin((0., 0., 6., 0.))
            .corner_radius(8.)
            .maybe(
                *current_page.read() == Some(SettingsPage::Account),
                |this| this.background(theme.md.primary_container.as_argb_u32()),
            )
            .child(
                rect()
                    .horizontal()
                    .width(Size::Fill)
                    .spacing(8.)
                    .padding((6., 8.))
                    .cross_align(Alignment::Center)
                    .child(Avatar::new(current_user.clone(), None, 32.))
                    .child(
                        rect()
                            .child(
                                label()
                                    .text(current_user.read().username.clone())
                                    .font_size(11.),
                            )
                            .child(label().text("My Account").font_size(15.)),
                    ),
            )
            .on_press({
                let mut current_page = current_page.clone();
                move |_| *current_page.write() = Some(SettingsPage::Account)
            })
    }
}

#[derive(PartialEq)]
struct SettingsButton {
    pub page: SettingsPage,
}

impl Component for SettingsButton {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::SettingsPage);
        let current_page = radio.slice_mut_current(|state| &mut state.settings_page);
        let theme = consume_material_theme();

        StoatButton::new()
            .corner_radius(8.)
            .maybe(*current_page.read() == Some(self.page), |this| {
                this.background(theme.md.primary_container.as_argb_u32())
            })
            .child(
                rect()
                    .horizontal()
                    .width(Size::Fill)
                    .padding((6., 8.))
                    .spacing(8.)
                    .cross_align(Alignment::Center)
                    .child(MaterialIcon::new(self.page.icon()).size(Size::px(20.)))
                    .child(
                        label()
                            .font_size(15)
                            .margin((0., 0., 2., 0.))
                            .text(self.page.title()),
                    ),
            )
            .on_press({
                let mut current_page = current_page.clone();
                let page = self.page;
                move |_| *current_page.write() = Some(page)
            })
    }
}

fn settings_category(title: &'static str, theme: &Theme, pages: &[SettingsPage]) -> Rect {
    rect()
        .spacing(8.)
        .child(
            label()
                .text(title)
                .color(theme.md.outline.as_argb_u32())
                .font_size(12)
                .font_weight(FontWeight::BOLD)
                .margin((0., 8.)),
        )
        .child(
            rect().spacing(6.).children(
                pages
                    .into_iter()
                    .map(|page| SettingsButton { page: *page }.into_element()),
            ),
        )
}

#[derive(PartialEq)]
struct LogoutButton {}

impl Component for LogoutButton {
    fn render(&self) -> impl IntoElement {
        let mut config = use_config();
        let theme = consume_material_theme();

        StoatButton::new()
            .corner_radius(8.)
            .color(theme.md.error.as_argb_u32())
            .child(
                rect()
                    .padding((6., 8.))
                    .width(Size::Fill)
                    .horizontal()
                    .spacing(8.)
                    .cross_align(Alignment::Center)
                    .child(MaterialIcon::new(delete()).size(Size::px(20.)))
                    .child(
                        label()
                            .font_size(15)
                            .margin((0., 0., 2., 0.))
                            .text("Log Out"),
                    ),
            )
            .on_press(move |_| config.write().token = None)
    }
}
