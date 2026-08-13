use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;
use stoat_permissions::{DataPermissionsValue, Override};

use crate::{
    AppChannel, SelectedRole, ServerSettingsPage, SizeExt, Tag,
    components::{
        MaterialIcon, ModalValue, PermissionsEditor, SingleLineEntry, StoatButton,
        StoatButtonColorsThemePartialExt, StoatButtonLayoutThemePartialExt, StoatColorPicker,
        checkbox::StoatCheckbox,
        file_image,
        material::{
            filled::{chevron_right, clear, list},
            outlined::group_add,
        },
        use_modals,
    },
    consume_material_theme, http, parse_fill, prompt_image_upload, use_initial,
};

#[derive(PartialEq)]
pub struct RoleServerSettings {
    pub server: Readable<v0::Server>,
    pub selected_role: Option<SelectedRole>,
}

impl Component for RoleServerSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();
        let mut radio = use_radio(AppChannel::ServerSettingsPage);

        let set_selected_role = {
            let server = self.server.read().id.clone();
            move |role| {
                radio.write().server_settings_page =
                    Some((server.clone(), ServerSettingsPage::Roles(Some(role))))
            }
        };

        let ordered_roles = use_memo({
            let server = self.server.clone();
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

        match self.selected_role.clone() {
            Some(SelectedRole::Default) => DefaultRoleServerSettings {
                server: self.server.clone(),
            }
            .into_element(),
            Some(SelectedRole::Role(id)) => SelectedRoleServerSettings {
                server: self.server.clone(),
                role_id: id,
            }
            .into_element(),
            None => rect()
                .spacing(15.)
                .child(
                    rect()
                        .spacing(4.)
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
                                                        .background(
                                                            theme.md.surface_dim.as_argb_u32(),
                                                        )
                                                        .color(theme.md.on_surface.as_argb_u32())
                                                        .center()
                                                        .child(
                                                            MaterialIcon::new(list())
                                                                .size(Size::px(22.)),
                                                        ),
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
                                                                .text(
                                                                    "Affects all roles and users",
                                                                ),
                                                        ),
                                                )
                                                .child(
                                                    MaterialIcon::new(chevron_right())
                                                        .size(Size::px(18.)),
                                                ),
                                        ),
                                ),
                        )
                        .child(
                            StoatButton::new()
                                .corner_radius(12.)
                                .on_press({
                                    let id = self.server.read().id.clone();
                                    move |_| {
                                        modals.write().push_modal(ModalValue::CreateRole {
                                            server: id.clone(),
                                        })
                                    }
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
                                                        .background(
                                                            theme.md.surface_dim.as_argb_u32(),
                                                        )
                                                        .color(theme.md.on_surface.as_argb_u32())
                                                        .center()
                                                        .child(
                                                            MaterialIcon::new(group_add())
                                                                .size(Size::px(22.)),
                                                        ),
                                                )
                                                .child(
                                                    rect()
                                                        .width(Size::flex(1.))
                                                        .child(
                                                            label()
                                                                .font_size(14.)
                                                                .font_weight(FontWeight::SEMI_BOLD)
                                                                .line_height(1.5)
                                                                .text("Create Role"),
                                                        )
                                                        .child(
                                                            label()
                                                                .font_size(12.)
                                                                .line_height(1.5)
                                                                .text("Create a new role"),
                                                        ),
                                                )
                                                .child(
                                                    MaterialIcon::new(chevron_right())
                                                        .width(Size::px(18.))
                                                        .height(Size::px(18.)),
                                                ),
                                        ),
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
                                                            .width(Size::px(18.))
                                                            .height(Size::px(18.)),
                                                    ),
                                            ),
                                    )
                                    .into_element()
                            },
                        ))),
                )
                .into_element(),
        }
    }
}

#[derive(PartialEq)]
pub struct SelectedRoleServerSettings {
    pub server: Readable<v0::Server>,
    pub role_id: String,
}

impl Component for SelectedRoleServerSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let server = self.server.read();
        let current_role = server.roles.get(&self.role_id).unwrap();

        let mut name = use_initial(move || current_role.name.clone());

        let mut color = use_initial(move || {
            current_role
                .colour
                .as_deref()
                .and_then(parse_fill)
                .and_then(|f| {
                    if let Fill::Color(c) = f {
                        Some(c)
                    } else {
                        None
                    }
                })
        });

        let mut hoist = use_initial(move || current_role.hoist);

        let mut overrides = use_initial(move || current_role.permissions);

        let mut error = use_state(|| None);

        let edit_role = {
            let server_id = server.id.clone();
            let role_id = self.role_id.clone();

            move |payload| {
                let server_id = server_id.clone();
                let role_id = role_id.clone();

                async move {
                    match http().edit_role(&server_id, &role_id, &payload).await {
                        Ok(role) => Some(role),
                        Err(e) => {
                            error.set(Some(e));
                            None
                        }
                    }
                }
            }
        };

        let remove_field = {
            let edit_role = edit_role.clone();

            move |field| {
                let edit_role = edit_role.clone();

                async move {
                    edit_role(v0::DataEditRole {
                        name: None,
                        colour: None,
                        hoist: None,
                        rank: None,
                        icon: None,
                        remove: vec![field],
                    })
                    .await
                }
            }
        };

        rect()
            .spacing(15.)
            .child(SingleLineEntry::new("Role Name", name))
            .child(rect().horizontal().child(StoatColorPicker::new(
                color.read().cloned().unwrap_or(Color::WHITE),
                move |c| color.set(Some(c)),
            )))
            .child(label().text("Role Icon").font_size(12.))
            .child(
                rect()
                    .horizontal()
                    .spacing(15.)
                    .cross_align(Alignment::Center)
                    .child(
                        StoatButton::new()
                            .corner_radius(48.)
                            .on_press({
                                let edit_role = edit_role.clone();

                                move |_| {
                                    let edit_role = edit_role.clone();

                                    spawn(async move {
                                        if let Some(id) = prompt_image_upload(Tag::Icons).await {
                                            edit_role(v0::DataEditRole {
                                                name: None,
                                                colour: None,
                                                hoist: None,
                                                rank: None,
                                                icon: Some(id),
                                                remove: Vec::new(),
                                            })
                                            .await;
                                        };
                                    });
                                }
                            })
                            .child(
                                rect()
                                    .width(Size::px(96.))
                                    .height(Size::px(96.))
                                    .background(theme.md.surface_dim.as_argb_u32())
                                    .maybe_child(current_role.icon.as_ref().map(|icon| {
                                        rect()
                                            .layer(Layer::Relative(1))
                                            .width(Size::Fill)
                                            .height(Size::Fill)
                                            .child(file_image(icon))
                                    })),
                            ),
                    )
                    .child(
                        StoatButton::new()
                            .corner_radius(16.)
                            .on_press({
                                let remove_field = remove_field.clone();

                                move |_| {
                                    let remove_field = remove_field.clone();

                                    spawn(async move {
                                        remove_field(v0::FieldsRole::Icon).await;
                                    });
                                }
                            })
                            .child(
                                rect().size(Size::px(36.)).center().child(
                                    MaterialIcon::new(clear())
                                        .size(Size::px(24.))
                                        .color(theme.md.primary.as_argb_u32()),
                                ),
                            ),
                    ),
            )
            .child(label().text("Hoist Role").font_size(12.))
            .child(rect().child(StoatCheckbox::new(hoist).child("Display this role above others")))
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
                                name.reset();
                                color.reset();
                                hoist.reset();
                            }),
                    )
                    .child(
                        StoatButton::new()
                            .color(theme.md.on_primary.as_argb_u32())
                            .background(theme.md.primary.as_argb_u32())
                            .corner_radius(40.)
                            .on_press({
                                let edit_role = edit_role.clone();

                                move |_| {
                                    let edit_role = edit_role.clone();

                                    spawn({
                                        async move {
                                            let payload = v0::DataEditRole {
                                                name: name.get_if_different(),
                                                colour: color
                                                    .get_if_different()
                                                    .flatten()
                                                    .map(|c| c.to_hex_string()),
                                                hoist: hoist.get_if_different(),
                                                icon: None,
                                                rank: None,
                                                remove: Vec::new(),
                                            };

                                            if edit_role(payload).await.is_some() {
                                                name.apply();
                                                color.apply();
                                                hoist.apply();
                                            };
                                        }
                                    });
                                }
                            })
                            .child(
                                rect()
                                    .height(Size::px(40.))
                                    .padding((0., 16.))
                                    .center()
                                    .child("Save"),
                            ),
                    ),
            )
            .child(
                rect()
                    .child(PermissionsEditor::new_overrite(overrides))
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
                                        overrides.reset();
                                    }),
                            )
                            .child(
                                StoatButton::new()
                                    .color(theme.md.on_primary.as_argb_u32())
                                    .background(theme.md.primary.as_argb_u32())
                                    .corner_radius(40.)
                                    .on_press({
                                        let server_id = server.id.clone();
                                        let role_id = self.role_id.clone();

                                        move |_| {
                                            let server_id = server_id.clone();
                                            let role_id = role_id.clone();

                                            spawn({
                                                async move {
                                                    let field = *overrides.read();

                                                    if http()
                                                        .set_role_server_permissions(
                                                            &server_id,
                                                            &role_id,
                                                            &v0::DataSetServerRolePermission {
                                                                permissions: Override {
                                                                    allow: field.a as u64,
                                                                    deny: field.d as u64,
                                                                },
                                                            },
                                                        )
                                                        .await
                                                        .is_ok()
                                                    {
                                                        overrides.apply();
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
                    ),
            )
    }
}

#[derive(PartialEq)]
struct DefaultRoleServerSettings {
    server: Readable<v0::Server>,
}

impl Component for DefaultRoleServerSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let server = self.server.read();

        let mut permissions = use_initial(|| server.default_permissions);

        rect().child(
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
                                    let server_id = server.id.clone();

                                    move |_| {
                                        let server_id = server_id.clone();

                                        spawn({
                                            async move {
                                                let value = *permissions.read();

                                                if http()
                                                    .set_default_server_permissions(
                                                        &server_id,
                                                        &DataPermissionsValue {
                                                            permissions: value as u64,
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
                ),
        )
    }
}
