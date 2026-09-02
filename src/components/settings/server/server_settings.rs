use std::{borrow::Cow, mem::discriminant};

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, SelectedRole, ServerSettingsPage, SizeExt,
    components::{
        AuditLogServerSettings, EmojiServerSettings, InviteServerSettings, MaterialIcon,
        OverviewServerSettings, RoleServerSettings, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt,
        material::filled::{chevron_right, clear, delete},
    },
    consume_material_theme,
    theme::Theme,
};

#[derive(PartialEq)]
pub struct ServerSettings {
    pub server: Readable<v0::Server>,
}

impl Component for ServerSettings {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::ServerSettingsPage);
        let current_page = radio.slice_mut_current(|state| &mut state.server_settings_page);
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
                                        self.server.read().name.to_uppercase(),
                                        &theme,
                                        &[ServerSettingsPage::Overview],
                                    ))
                                    .child(settings_category(
                                        "CUSTOMISATION",
                                        &theme,
                                        &[ServerSettingsPage::Emojis],
                                    ))
                                    .child(settings_category(
                                        "MODERATION",
                                        &theme,
                                        &[ServerSettingsPage::AuditLogs],
                                    ))
                                    .child(settings_category(
                                        "USER MANAGEMENT",
                                        &theme,
                                        &[
                                            ServerSettingsPage::Roles(None),
                                            ServerSettingsPage::Invites,
                                            ServerSettingsPage::Bans,
                                        ],
                                    ))
                                    .child(DeleteServerButton {}),
                            ),
                    )
                    .child(
                        rect()
                            .corner_radius(CornerRadius::new(
                                16.,
                                0.,
                                0.,
                                16.,
                            ))
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
                                            if let ServerSettingsPage::Roles(Some(role)) = &page {
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
                                                    .child(
                                                        rect()
                                                            .child(page.title())
                                                            .maybe(selected_role.is_some(), |label|
                                                                label
                                                                    .color(theme.md.outline.as_argb_u32())
                                                                    .cursor(CursorIcon::Pointer)
                                                                    .on_press({
                                                                        let mut current_page = current_page.clone();
                                                                        move |_| {
                                                                            if let Some(v) = current_page
                                                                                .write()
                                                                                .as_mut()
                                                                            {
                                                                                v.1 = ServerSettingsPage::Roles(None);
                                                                            }
                                                                        }
                                                                    }))
                                                    )
                                                    .maybe_child(selected_role.is_some().then(
                                                        || {
                                                            MaterialIcon::new(chevron_right())
                                                                .size(Size::px(14.))
                                                                .color(theme.md.outline.as_argb_u32())
                                                        },
                                                    ))
                                                    .maybe_child(selected_role.map(|role| {
                                                                                                                label()
                                                            .text(

                                                        match role {
                                                            SelectedRole::Default => {
                                                                "Default Permissions".to_string()
                                                            }
                                                            SelectedRole::Role(id) => self
                                                                .server
                                                                .read()
                                                                .roles
                                                                .get(&id)
                                                                .unwrap()
                                                                .name
                                                                .clone(),
                                                        })
                                                    })),
                                            )
                                            .child(match page {
                                                ServerSettingsPage::Overview => {
                                                    OverviewServerSettings {
                                                        server: self.server.clone(),
                                                    }
                                                    .into_element()
                                                }
                                                ServerSettingsPage::Emojis => EmojiServerSettings {
                                                    server: self.server.clone(),
                                                }
                                                .into_element(),
                                                ServerSettingsPage::Roles(selected_role) => {
                                                    RoleServerSettings {
                                                        server: self.server.clone(),
                                                        selected_role,
                                                    }
                                                    .into_element()
                                                }
                                                ServerSettingsPage::Invites => {
                                                    InviteServerSettings {
                                                        server: self.server.clone(),
                                                    }
                                                    .into_element()
                                                }
                                                ServerSettingsPage::Bans => {
                                                    "Coming soon!".into_element()
                                                },
                                                ServerSettingsPage::AuditLogs => {
                                                    AuditLogServerSettings {  server: self.server.clone() }.into_element()
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
struct ServerSettingsButton {
    pub page: ServerSettingsPage,
}

impl Component for ServerSettingsButton {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::ServerSettingsPage);
        let current_page = radio.slice_mut_current(|state| &mut state.server_settings_page);
        let theme = consume_material_theme();

        StoatButton::new()
            .corner_radius(8.)
            .maybe(
                current_page
                    .read()
                    .as_ref()
                    .is_some_and(|(_, page)| discriminant(page) == discriminant(&self.page)),
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
    pages: &[ServerSettingsPage],
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
                    .map(|page| ServerSettingsButton { page: page.clone() }.into_element()),
            ),
        )
}

#[derive(PartialEq)]
struct DeleteServerButton {}

impl Component for DeleteServerButton {
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
                            .text("Delete Server"),
                    ),
            )
            .on_press(move |_| {})
    }
}
