use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;
use stoat_permissions::OverrideField;

use crate::{
    AppChannel, ChannelSettingsPage, SelectedRole, SizeExt,
    components::{
        MaterialIcon, PermissionsEditor, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt,
        material::filled::{chevron_right, list},
    },
    consume_material_theme, http, parse_fill, use_initial,
};

#[derive(PartialEq)]
pub struct PermissionsChannelSettings {
    pub channel: Readable<v0::Channel>,
    pub selected_role: Option<SelectedRole>,
}

impl Component for PermissionsChannelSettings {
    fn render(&self) -> impl IntoElement {
        match &*self.channel.read() {
            v0::Channel::Group {
                id, permissions, ..
            } => GroupPermissionsChannelSettings {
                id: id.clone(),
                permissions: permissions.unwrap_or_default(),
            }
            .into_element(),
            v0::Channel::TextChannel {
                id,
                server,
                default_permissions,
                role_permissions,
                ..
            } => match self.selected_role.clone() {
                Some(selected_role) => {
                    let permissions = match &selected_role {
                        SelectedRole::Default => default_permissions.clone().unwrap_or_default(),
                        SelectedRole::Role(role_id) => {
                            role_permissions.get(role_id).cloned().unwrap_or_default()
                        }
                    };

                    ServerChannelPermissionsSettings {
                        id: id.clone(),
                        server: server.clone(),
                        selected_role,
                        permissions,
                    }
                    .into_element()
                }
                None => ServerChannelPermissionsOverviewSettings {
                    channel: self.channel.clone(),
                    server_id: server.clone(),
                }
                .into_element(),
            },
            _ => unreachable!(),
        }
    }
}

#[derive(PartialEq)]
pub struct ServerChannelPermissionsOverviewSettings {
    pub channel: Readable<v0::Channel>,
    pub server_id: String,
}

impl Component for ServerChannelPermissionsOverviewSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut radio = use_radio(AppChannel::ChannelSettingsPage);

        let server = radio.slice(AppChannel::Servers, {
            let server_id = self.server_id.clone();
            move |state| state.servers.get(&server_id).unwrap()
        });

        let set_selected_role = {
            let channel = self.channel.read().id().to_string();

            move |role| {
                radio.write().channel_settings_page = Some((
                    channel.clone(),
                    ChannelSettingsPage::Permissions(Some(role)),
                ))
            }
        };

        let ordered_roles = use_memo({
            let server = server.clone();

            move || {
                let mut roles = server
                    .read()
                    .roles
                    .values()
                    .cloned()
                    .map(|role| {
                        let color = role.colour.as_deref().and_then(parse_fill);
                        (role, color)
                    })
                    .collect::<Vec<_>>();

                roles.sort_by(|(a, _), (b, _)| a.rank.cmp(&b.rank));
                roles
            }
        });

        rect()
            .spacing(15.)
            .child(
                StoatButton::new()
                    .corner_radius(12.)
                    .on_press({
                        let mut set_selected_role = set_selected_role.clone();
                        move |_| set_selected_role(SelectedRole::Default)
                    })
                    .child(
                        rect()
                            .padding(13.)
                            .background(theme.md.secondary_container.as_argb_u32())
                            .color(theme.md.on_secondary_container.as_argb_u32())
                            .child(
                                rect()
                                    .horizontal()
                                    .spacing(16.)
                                    .cross_align(Alignment::Center)
                                    .content(Content::Flex)
                                    .child(
                                        rect()
                                            .corner_radius(36.)
                                            .width(Size::px(36.))
                                            .height(Size::px(36.))
                                            .background(theme.md.surface_dim.as_argb_u32())
                                            .color(theme.md.on_surface.as_argb_u32())
                                            .center()
                                            .child(MaterialIcon::new(list()).size(Size::px(22.))),
                                    )
                                    .child(
                                        rect()
                                            .width(Size::flex(1.))
                                            .child(
                                                label()
                                                    .font_size(14.)
                                                    .font_weight(FontWeight::SEMI_BOLD)
                                                    .line_height(1.5)
                                                    .text("Default Permissions"),
                                            )
                                            .child(
                                                label()
                                                    .font_size(12.)
                                                    .line_height(1.5)
                                                    .text("Affects all roles and users"),
                                            ),
                                    )
                                    .child(MaterialIcon::new(chevron_right()).size(Size::px(18.))),
                            ),
                    ),
            )
            .child(
                rect()
                    .spacing(4.)
                    .child(label().font_size(12.).text("Server Roles"))
                    .child(rect().spacing(8.).children(ordered_roles.read().iter().map(
                        |(role, color)| {
                            let mut role_color =
                                rect().background(theme.md.outline_variant.as_argb_u32());

                            if let Some(color) = color {
                                role_color.get_style().background = color.clone();
                            };

                            StoatButton::new()
                                .corner_radius(12.)
                                .on_press({
                                    let id = role.id.clone();
                                    let mut set_selected_role = set_selected_role.clone();

                                    move |_| set_selected_role(SelectedRole::Role(id.clone()))
                                })
                                .child(
                                    rect()
                                        .padding(13.)
                                        .background(theme.md.secondary_container.as_argb_u32())
                                        .color(theme.md.on_secondary_container.as_argb_u32())
                                        .child(
                                            rect()
                                                .horizontal()
                                                .spacing(16.)
                                                .cross_align(Alignment::Center)
                                                .content(Content::Flex)
                                                .child(
                                                    role_color
                                                        .corner_radius(36.)
                                                        .width(Size::px(36.))
                                                        .height(Size::px(36.)),
                                                )
                                                .child(
                                                    label()
                                                        .width(Size::flex(1.))
                                                        .font_size(14.)
                                                        .font_weight(FontWeight::SEMI_BOLD)
                                                        .line_height(1.5)
                                                        .text(role.name.clone()),
                                                )
                                                .child(
                                                    MaterialIcon::new(chevron_right())
                                                        .size(Size::px(18.)),
                                                ),
                                        ),
                                )
                                .into_element()
                        },
                    ))),
            )
    }
}

#[derive(PartialEq)]
pub struct GroupPermissionsChannelSettings {
    pub id: String,
    pub permissions: i64,
}

impl Component for GroupPermissionsChannelSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut permissions = use_initial(|| self.permissions);

        rect()
            .child(PermissionsEditor::new_value(permissions))
            .child(
                rect()
                    .horizontal()
                    .spacing(8.)
                    .font_size(14)
                    .child(
                        StoatButton::new()
                            .color(theme.md.primary.as_argb_u32())
                            .corner_radius(40.)
                            .child(
                                rect()
                                    .height(Size::px(40.))
                                    .padding((0., 16.))
                                    .center()
                                    .child("Reset"),
                            )
                            .on_press(move |_| {
                                permissions.reset();
                            }),
                    )
                    .child(
                        StoatButton::new()
                            .color(theme.md.on_primary.as_argb_u32())
                            .background(theme.md.primary.as_argb_u32())
                            .corner_radius(40.)
                            .on_press({
                                let channel_id = self.id.clone();

                                move |_| {
                                    let channel_id = channel_id.clone();

                                    spawn({
                                        async move {
                                            let field = *permissions.read();

                                            if http()
                                                .set_default_channel_permissions(
                                                    &channel_id,
                                                    &v0::DataDefaultChannelPermissions::Value {
                                                        permissions: field as u64,
                                                    },
                                                )
                                                .await
                                                .is_ok()
                                            {
                                                permissions.apply();
                                            }
                                        }
                                    });
                                }
                            })
                            .child(
                                rect()
                                    .height(Size::px(40.))
                                    .padding((0., 16.))
                                    .center()
                                    .child("Save permissions"),
                            ),
                    ),
            )
    }
}

#[derive(PartialEq)]
pub struct ServerChannelPermissionsSettings {
    pub id: String,
    pub server: String,
    pub selected_role: SelectedRole,
    pub permissions: OverrideField,
}

impl Component for ServerChannelPermissionsSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut permissions = use_initial(|| self.permissions);

        rect()
            .child(PermissionsEditor::new_overrite(permissions))
            .child(
                rect()
                    .horizontal()
                    .spacing(8.)
                    .font_size(14)
                    .child(
                        StoatButton::new()
                            .color(theme.md.primary.as_argb_u32())
                            .corner_radius(40.)
                            .child(
                                rect()
                                    .height(Size::px(40.))
                                    .padding((0., 16.))
                                    .center()
                                    .child("Reset"),
                            )
                            .on_press(move |_| {
                                permissions.reset();
                            }),
                    )
                    .child(
                        StoatButton::new()
                            .color(theme.md.on_primary.as_argb_u32())
                            .background(theme.md.primary.as_argb_u32())
                            .corner_radius(40.)
                            .on_press({
                                let channel_id = self.id.clone();
                                let selected_role = self.selected_role.clone();

                                move |_| {
                                    let channel_id = channel_id.clone();
                                    let selected_role = selected_role.clone();

                                    spawn({
                                        async move {
                                            let field = permissions.read().cloned().into();

                                            match selected_role {
                                                SelectedRole::Default => http()
                                                    .set_default_channel_permissions(
                                                        &channel_id,
                                                        &v0::DataDefaultChannelPermissions::Field {
                                                            permissions: field,
                                                        },
                                                    )
                                                    .await,
                                                SelectedRole::Role(role_id) => {
                                                    http()
                                                        .set_role_channel_permissions(
                                                            &channel_id,
                                                            &role_id,
                                                            &v0::DataSetRolePermissions {
                                                                permissions: field,
                                                            },
                                                        )
                                                        .await
                                                }
                                            }
                                            .unwrap();

                                            permissions.apply();
                                        }
                                    });
                                }
                            })
                            .child(
                                rect()
                                    .height(Size::px(40.))
                                    .padding((0., 16.))
                                    .center()
                                    .child("Save permissions"),
                            ),
                    ),
            )
    }
}
