use std::borrow::Cow;

use crate::{
    AppChannel, LocalFile,
    components::{
        SingleLineEntry, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt, image,
    },
    http,
    types::Tag,
    use_initial, use_material_theme,
};
use freya::{icons::lucide::x, prelude::*, radio::use_radio};
use rfd::AsyncFileDialog;
use stoat_models::v0;

#[derive(PartialEq)]
pub struct OverviewServerSettings {
    pub server: Readable<v0::Server>,
}

impl Component for OverviewServerSettings {
    fn render(&self) -> impl IntoElement {
        let theme = use_material_theme();
        let mut error = use_state(|| None);

        let edit_server = {
            let server_id = self.server.read().id.clone();

            move |payload| {
                let server_id = server_id.clone();
                async move {
                    match http().edit_server(&server_id, &payload).await {
                        Ok(server) => Some(server),
                        Err(e) => {
                            error.set(Some(e));
                            None
                        }
                    }
                }
            }
        };

        let remove_field = {
            let edit_server = edit_server.clone();

            move |field| {
                let edit_server = edit_server.clone();

                async move {
                    edit_server(v0::DataEditServer {
                        name: None,
                        description: None,
                        icon: None,
                        banner: None,
                        categories: None,
                        system_messages: None,
                        flags: None,
                        discoverable: None,
                        analytics: None,
                        owner: None,
                        remove: vec![field],
                    })
                    .await
                }
            }
        };

        let prompt_image_upload = {
            move |tag: Tag| async move {
                if let Some(file) = AsyncFileDialog::new().pick_file().await {
                    let contents = file.read().await.into();
                    let filename = file.file_name();

                    if let Ok(response) = http()
                        .upload_file(
                            tag.as_str(),
                            LocalFile {
                                name: filename,
                                body: contents,
                            },
                        )
                        .await
                    {
                        return Some(response.id);
                    };
                };

                None
            }
        };

        let current_server = self.server.read();

        let mut server_name = use_initial(|| current_server.name.clone());
        let mut server_description =
            use_initial(|| current_server.description.clone().unwrap_or_default());
        let mut user_joined = use_initial(|| {
            current_server
                .system_messages
                .as_ref()
                .and_then(|s| s.user_joined.clone())
        });
        let mut user_left = use_initial(|| {
            current_server
                .system_messages
                .as_ref()
                .and_then(|s| s.user_left.clone())
        });
        let mut user_kicked = use_initial(|| {
            current_server
                .system_messages
                .as_ref()
                .and_then(|s| s.user_kicked.clone())
        });
        let mut user_banned = use_initial(|| {
            current_server
                .system_messages
                .as_ref()
                .and_then(|s| s.user_banned.clone())
        });

        rect()
            .spacing(8.)
            .child(label().text("Server Icon").font_size(12.))
            .child(
                rect()
                    .horizontal()
                    .spacing(15.)
                    .cross_align(Alignment::Center)
                    .child(
                        StoatButton::new()
                            .corner_radius(48.)
                            .on_press({
                                let prompt_image_upload = prompt_image_upload.clone();
                                let edit_server = edit_server.clone();

                                move |_| {
                                    let prompt_image_upload = prompt_image_upload.clone();
                                    let edit_server = edit_server.clone();

                                    spawn(async move {
                                        if let Some(id) = prompt_image_upload(Tag::Icons).await {
                                            edit_server(v0::DataEditServer {
                                                name: None,
                                                description: None,
                                                icon: Some(id),
                                                banner: None,
                                                categories: None,
                                                system_messages: None,
                                                flags: None,
                                                discoverable: None,
                                                analytics: None,
                                                owner: None,
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
                                    .maybe_child(current_server.icon.as_ref().map(|icon| {
                                        rect()
                                            .layer(Layer::Relative(1))
                                            .width(Size::Fill)
                                            .height(Size::Fill)
                                            .child(image(icon))
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
                                        remove_field(v0::FieldsServer::Icon).await;
                                    });
                                }
                            })
                            .child(
                                rect()
                                    .width(Size::px(36.))
                                    .height(Size::px(36.))
                                    .center()
                                    .child(
                                        SvgViewer::new(x())
                                            .width(Size::px(24.))
                                            .height(Size::px(24.))
                                            .color(theme.md.primary.as_argb_u32()),
                                    ),
                            ),
                    ),
            )
            .child(label().text("Server Banner").font_size(12.))
            .child(
                rect()
                    .horizontal()
                    .spacing(15.)
                    .cross_align(Alignment::Center)
                    .child(
                        StoatButton::new()
                            .corner_radius(16.)
                            .on_press({
                                let prompt_image_upload = prompt_image_upload.clone();
                                let edit_server = edit_server.clone();

                                move |_| {
                                    let prompt_image_upload = prompt_image_upload.clone();
                                    let edit_server = edit_server.clone();

                                    spawn(async move {
                                        if let Some(id) = prompt_image_upload(Tag::Banners).await {
                                            edit_server(v0::DataEditServer {
                                                name: None,
                                                description: None,
                                                icon: None,
                                                banner: Some(id),
                                                categories: None,
                                                system_messages: None,
                                                flags: None,
                                                discoverable: None,
                                                analytics: None,
                                                owner: None,
                                                remove: Vec::new(),
                                            })
                                            .await;
                                        };
                                    });
                                }
                            })
                            .child(
                                rect()
                                    .width(Size::px((96. / 100.) * 232.))
                                    .height(Size::px(96.))
                                    .background(theme.md.surface_dim.as_argb_u32())
                                    .maybe_child(current_server.banner.as_ref().map(|icon| {
                                        rect()
                                            .layer(Layer::Relative(1))
                                            .width(Size::Fill)
                                            .height(Size::Fill)
                                            .child(image(icon).aspect_ratio(AspectRatio::Max))
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
                                        remove_field(v0::FieldsServer::Banner).await;
                                    });
                                }
                            })
                            .child(
                                rect()
                                    .width(Size::px(36.))
                                    .height(Size::px(36.))
                                    .center()
                                    .child(
                                        SvgViewer::new(x())
                                            .width(Size::px(24.))
                                            .height(Size::px(24.))
                                            .color(theme.md.primary.as_argb_u32()),
                                    ),
                            ),
                    ),
            )
            .child(SingleLineEntry::new("Server Name", server_name).margin((0., 0., 24., 0.)))
            .child(
                SingleLineEntry::new("Server Description", server_description)
                    .placeholder("This server is about...")
                    .margin((0., 0., 24., 0.)),
            )
            .child(label().text("System message channels").font_size(14.))
            .child(label().text("User Joined").font_size(12.))
            .child(SystemMessagesChannelSelector::new(
                self.server.clone(),
                user_joined,
            ))
            .child(label().text("User Left").font_size(12.))
            .child(SystemMessagesChannelSelector::new(
                self.server.clone(),
                user_left,
            ))
            .child(label().text("User Kicked").font_size(12.))
            .child(SystemMessagesChannelSelector::new(
                self.server.clone(),
                user_kicked,
            ))
            .child(label().text("User Banned").font_size(12.))
            .child(SystemMessagesChannelSelector::new(
                self.server.clone(),
                user_banned,
            ))
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
                                server_name.reset();
                                server_description.reset();
                                user_joined.reset();
                                user_left.reset();
                                user_kicked.reset();
                                user_banned.reset();
                            }),
                    )
                    .child(
                        StoatButton::new()
                            .color(theme.md.on_primary.as_argb_u32())
                            .background(theme.md.primary.as_argb_u32())
                            .corner_radius(40.)
                            .on_press({
                                let server = self.server.clone();

                                move |_| {
                                    let server = server.clone();
                                    let edit_server = edit_server.clone();

                                    spawn({
                                        async move {
                                            let current = server.read().clone();
                                            let current_system_messages = current
                                                .system_messages
                                                .unwrap_or_else(|| v0::SystemMessageChannels {
                                                    user_joined: None,
                                                    user_left: None,
                                                    user_kicked: None,
                                                    user_banned: None,
                                                });

                                            let system_messages = v0::SystemMessageChannels {
                                                user_joined: user_joined
                                                    .get_if_different()
                                                    .unwrap_or_else(|| {
                                                        current_system_messages.user_joined.clone()
                                                    }),
                                                user_left: user_left
                                                    .get_if_different()
                                                    .unwrap_or_else(|| {
                                                        current_system_messages.user_left.clone()
                                                    }),
                                                user_kicked: user_kicked
                                                    .get_if_different()
                                                    .unwrap_or_else(|| {
                                                        current_system_messages.user_kicked.clone()
                                                    }),
                                                user_banned: user_banned
                                                    .get_if_different()
                                                    .unwrap_or_else(|| {
                                                        current_system_messages.user_banned.clone()
                                                    }),
                                            };

                                            let payload = v0::DataEditServer {
                                                name: server_name.get_if_different(),
                                                description: server_description.get_if_different(),
                                                icon: None,
                                                banner: None,
                                                categories: None,
                                                system_messages: if current_system_messages
                                                    != system_messages
                                                {
                                                    Some(system_messages)
                                                } else {
                                                    None
                                                },
                                                flags: None,
                                                discoverable: None,
                                                analytics: None,
                                                owner: None,
                                                remove: Vec::new(),
                                            };

                                            if let Some(new_server) = edit_server(payload).await {
                                                server_name.set_new(new_server.name);
                                                server_description.set_new(
                                                    new_server.description.unwrap_or_default(),
                                                );

                                                if let Some(system_messages) =
                                                    new_server.system_messages
                                                {
                                                    user_joined
                                                        .set_new(system_messages.user_joined);
                                                    user_left.set_new(system_messages.user_left);
                                                    user_kicked
                                                        .set_new(system_messages.user_kicked);
                                                    user_banned
                                                        .set_new(system_messages.user_banned);
                                                }
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
    }
}

#[derive(PartialEq)]
struct SystemMessagesChannelSelector {
    server: Readable<v0::Server>,
    value: Writable<Option<String>>,
}

impl SystemMessagesChannelSelector {
    pub fn new(
        server: impl Into<Readable<v0::Server>>,
        value: impl Into<Writable<Option<String>>>,
    ) -> Self {
        Self {
            server: server.into(),
            value: value.into(),
        }
    }
}

impl Component for SystemMessagesChannelSelector {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Channels);
        let channels = radio.slice_current(|state| &state.channels);
        let theme = use_material_theme();

        let current_value = &*self.value.read();

        Select::new()
            .width(Size::Fill)
            .background_button(theme.md.surface_container_highest.as_argb_u32())
            .hover_background(theme.md.surface_container_highest.as_argb_u32())
            .select_background(theme.md.surface_container.as_argb_u32())
            .selected_item(
                rect()
                    .padding((8., 6.))
                    .width(Size::Fill)
                    .child(label().text(
                        if let Some(id) = &*current_value
                            && let Some(channel) = channels.read().get(id)
                        {
                            Cow::Owned(channel.name().unwrap().to_string())
                        } else {
                            Cow::Borrowed("Disabled")
                        },
                    )),
            )
            .child(
                MenuItem::new()
                    .selected(current_value.is_none())
                    .background(theme.md.surface_container.as_argb_u32())
                    .select_background(Color::lerp(
                        theme.md.primary.as_argb_u32().into(),
                        Color::TRANSPARENT,
                        0.12,
                    ))
                    .on_press({
                        let mut value = self.value.clone();
                        move |_| value.set(None)
                    })
                    .child("Disabled")
                    .into_element(),
            )
            .children(
                self.server
                    .read()
                    .channels
                    .iter()
                    .filter_map(|id| {
                        channels
                            .read()
                            .get(id)
                            .map(|c| (c.id().to_string(), c.name().unwrap().to_string()))
                    })
                    .map(move |(id, name)| {
                        MenuItem::new()
                            .selected(current_value.as_ref().is_some_and(|c| c == &id))
                            .background(theme.md.surface_container.as_argb_u32())
                            .select_background(theme.md.primary.as_u32() | (0xb9 << 24))
                            .on_press({
                                let mut value = self.value.clone();
                                move |_| value.set(Some(id.clone()))
                            })
                            .child(name)
                            .into_element()
                    }),
            )
    }
}
