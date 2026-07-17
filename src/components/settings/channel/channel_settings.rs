use std::borrow::Cow;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, ChannelSettingsPage, SelectedRole, SizeExt,
    components::{
        MaterialIcon, OverviewChannelSettings, PermissionsChannelSettings, StoatButton,
        StoatButtonColorsThemePartialExt, StoatButtonLayoutThemePartialExt,
        material::filled::{chevron_right, clear, delete},
    },
    consume_material_theme,
    theme::Theme,
};

#[derive(PartialEq)]
pub struct ChannelSettings {
    pub channel: Readable<v0::Channel>,
}

impl Component for ChannelSettings {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::ChannelSettingsPage);
        let current_page = radio.slice_mut_current(|state| &mut state.channel_settings_page);
        let servers = radio.slice(AppChannel::Servers, |state| &state.servers);

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
                                    .child(settings_category(
                                        match &*self.channel.read() {
                                            v0::Channel::TextChannel { name, .. }
                                            | v0::Channel::Group { name, .. } => name.clone(),
                                            _ => unreachable!(),
                                        },
                                        &theme,
                                        &[
                                            ChannelSettingsPage::Overview,
                                            ChannelSettingsPage::Permissions(None),
                                            ChannelSettingsPage::Webhooks,
                                        ],
                                    ))
                                    .child(DeleteChannelButton {}),
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
                                    .child(rect().padding((32., 32.)).child({
                                        let page = current_page.read().as_ref().unwrap().1.clone();

                                        let selected_role =
                                            if let ChannelSettingsPage::Permissions(Some(role)) =
                                                &page
                                            {
                                                Some(role.clone())
                                            } else {
                                                None
                                            };

                                        rect()
                                            .spacing(8.)
                                            .child(
                                                rect()
                                                    .cross_align(Alignment::Center)
                                                    .spacing(8.)
                                                    .font_size(22)
                                                    .font_weight(550)
                                                    .horizontal()
                                                    .child(label()
                                                        .text(page.title())
                                                        .maybe(selected_role.is_some(), |label|
                                                            label
                                                                .color(theme.md.outline.as_argb_u32())
                                                                .on_pointer_enter(move |_| {
                                                                    Cursor::set(CursorIcon::Pointer);
                                                                })
                                                                .on_pointer_leave(move |_| {
                                                                    Cursor::set(CursorIcon::default());
                                                                })
                                                                .on_press({
                                                                    let mut current_page =
                                                                        current_page.clone();
                                                                    move |_| {
                                                                        if let Some(v) = current_page
                                                                            .write()
                                                                            .as_mut()
                                                                        {
                                                                            v.1 = ChannelSettingsPage::Permissions(None);
                                                                        }
                                                                    }
                                                                }))

                                                    )
                                                    .maybe_child(selected_role.is_some().then(
                                                        || {
                                                            MaterialIcon::new(chevron_right())
                                                                .size(Size::px(14.))
                                                                .color(
                                                                    theme.md.outline.as_argb_u32(),
                                                                )
                                                        },
                                                    ))
                                                    .maybe_child(selected_role.map(|role| {
                                                        label().text(match role {
                                                            SelectedRole::Default => {
                                                                "Default Permissions".to_string()
                                                            }
                                                            SelectedRole::Role(id) => {
                                                                let v0::Channel::TextChannel {
                                                                    server,
                                                                    ..
                                                                } = self.channel.read().clone()
                                                                else {
                                                                    unreachable!()
                                                                };

                                                                servers
                                                                    .read()
                                                                    .get(&server)
                                                                    .unwrap()
                                                                    .roles
                                                                    .get(&id)
                                                                    .unwrap()
                                                                    .name
                                                                    .clone()
                                                            }
                                                        })
                                                    })),
                                            )
                                            .child(match page {
                                                ChannelSettingsPage::Overview => {
                                                    OverviewChannelSettings {
                                                        channel: self.channel.clone(),
                                                    }
                                                    .into_element()
                                                }
                                                ChannelSettingsPage::Permissions(selected_role) => {
                                                    PermissionsChannelSettings { channel: self.channel.clone(), selected_role }.into_element()
                                                }
                                                ChannelSettingsPage::Webhooks => {
                                                    "Coming soon!".into_element()
                                                }
                                            })
                                    })),
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
                                                    MaterialIcon::new(clear())
                                                        .size(Size::px(24.))
                                                ),
                                        ),
                                ),
                            ),
                    ),
            )
    }
}

#[derive(PartialEq)]
struct ChannelSettingsButton {
    pub page: ChannelSettingsPage,
}

impl Component for ChannelSettingsButton {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::ChannelSettingsPage);
        let current_page = radio.slice_mut_current(|state| &mut state.channel_settings_page);
        let theme = consume_material_theme();

        StoatButton::new()
            .corner_radius(8.)
            .maybe(
                current_page
                    .read()
                    .as_ref()
                    .is_some_and(|v| &v.1 == &self.page),
                |this| this.background(theme.md.primary_container.as_argb_u32()),
            )
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
                let page = self.page.clone();

                move |_| {
                    if let Some(v) = current_page.write().as_mut() {
                        v.1 = page.clone();
                    }
                }
            })
    }
}

fn settings_category(
    title: impl Into<Cow<'static, str>>,
    theme: &Theme,
    pages: &[ChannelSettingsPage],
) -> Rect {
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
                    .cloned()
                    .map(|page| ChannelSettingsButton { page }.into_element()),
            ),
        )
}

#[derive(PartialEq)]
struct DeleteChannelButton {}

impl Component for DeleteChannelButton {
    fn render(&self) -> impl IntoElement {
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
                            .text("Delete Channel"),
                    ),
            )
            .on_press(move |_| {})
    }
}
