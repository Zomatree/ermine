use std::collections::HashMap;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, ChannelSettingsPage, SizeExt, Tag,
    components::{
        MaterialIcon, ModalValue, ServerIcon, SingleLineEntry, StoatButton,
        StoatButtonColorsThemePartialExt, StoatButtonLayoutThemePartialExt, file_image,
        material::{
            filled::{chevron_right, clear},
            outlined::{content_copy, delete, link, webhook},
        },
        use_modals,
    },
    consume_material_theme, http, prompt_image_upload, use_initial,
};

#[derive(PartialEq)]
pub struct WebhookChannelSettings {
    pub channel: Readable<v0::Channel>,
    pub selected_webhook: Option<(String, String)>,
}

impl Component for WebhookChannelSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();
        let mut radio = use_radio(AppChannel::ChannelSettingsPage);

        let set_selected_webhook = {
            let channel = self.channel.read().id().to_string();

            move |webhook| {
                radio.write().channel_settings_page = Some((
                    channel.clone(),
                    ChannelSettingsPage::Webhooks(Some(webhook)),
                ))
            }
        };

        let mut webhooks = use_state(|| HashMap::new());

        use_hook(|| {
            let channel_id = self.channel.read().id().to_string();

            spawn(async move {
                if let Ok(response) = http().fetch_webhooks(&channel_id).await {
                    webhooks.with_mut(|mut webhooks| {
                        for webhook in response {
                            webhooks.insert(webhook.id.clone(), webhook);
                        }
                    });
                }
            });
        });

        match self
            .selected_webhook
            .as_ref()
            .and_then(|(id, _)| webhooks.read().get(id).cloned())
        {
            Some(webhook) => SelectedWebhookChannelSettings {
                channel: self.channel.clone(),
                webhooks,
                webhook,
            }
            .into_element(),
            None => {
                rect()
                    .spacing(15.)
                    .child(
                        rect().spacing(4.).child(
                            StoatButton::new()
                                .corner_radius(12.)
                                .on_press({
                                    let id = self.channel.read().id().to_string();
                                    move |_| {
                                        let mut set_selected_webhook = set_selected_webhook.clone();

                                        modals.write().push_modal(ModalValue::CreateWebhook {
                                            channel: id.clone(),
                                            callback: EventHandler::new(move |webhook: v0::Webhook| {
                                                set_selected_webhook((webhook.id.clone(), webhook.name.clone()));
                                                webhooks.write().insert(webhook.id.clone(), webhook);
                                            })
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
                                                            MaterialIcon::new(webhook())
                                                                .size(Size::px(22.)),
                                                        ),
                                                )
                                                .child(
                                                    rect().width(Size::flex(1.)).child(
                                                        label()
                                                            .font_size(14.)
                                                            .font_weight(500)
                                                            .line_height(1.5)
                                                            .text("Create Webhook"),
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
                            .spacing(8.)
                            .children(webhooks.read().values().cloned().map(|webhook| {
                                WebhookButton {
                                    channel: self.channel.clone(),
                                    webhook,
                                }
                            })),
                    )
                    .into_element()
            }
        }
    }
}

#[derive(PartialEq)]
pub struct WebhookButton {
    pub channel: Readable<v0::Channel>,
    pub webhook: v0::Webhook,
}

impl Component for WebhookButton {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let mut radio = use_radio(AppChannel::ChannelSettingsPage);

        let set_selected_webhook = {
            let channel = self.channel.read().id().to_string();

            move |webhook| {
                radio.write().channel_settings_page = Some((
                    channel.clone(),
                    ChannelSettingsPage::Webhooks(Some(webhook)),
                ))
            }
        };

        StoatButton::new()
            .corner_radius(12.)
            .on_press({
                let webhook = self.webhook.clone();
                let mut set_selected_webhook = set_selected_webhook.clone();

                move |_| set_selected_webhook((webhook.id.clone(), webhook.name.clone()))
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
                                    .size(Size::px(36.))
                                    .corner_radius(18.)
                                    .center()
                                    .background(theme.md.surface_dim.as_argb_u32())
                                    .child(
                                        ServerIcon::from_values(
                                            self.webhook.avatar.clone(),
                                            self.webhook.name.clone(),
                                            24.,
                                        )
                                        .corner_radius(12.),
                                    ),
                            )
                            .child(
                                rect()
                                    .width(Size::flex(1.))
                                    .spacing(2.)
                                    .child(
                                        label()
                                            .font_size(14.)
                                            .font_weight(500)
                                            .line_height(1.5)
                                            .text(self.webhook.name.clone()),
                                    )
                                    .child(
                                        label()
                                            .font_size(12.)
                                            .font_weight(500)
                                            .line_height(1.5)
                                            .text(self.webhook.id.clone()),
                                    ),
                            )
                            .child(
                                MaterialIcon::new(chevron_right())
                                    .width(Size::px(18.))
                                    .height(Size::px(18.)),
                            ),
                    ),
            )
    }
}

#[derive(PartialEq)]
pub struct SelectedWebhookChannelSettings {
    pub channel: Readable<v0::Channel>,
    pub webhooks: State<HashMap<String, v0::Webhook>>,
    pub webhook: v0::Webhook,
}

impl Component for SelectedWebhookChannelSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let radio = use_radio(AppChannel::ChannelSettingsPage);
        let channel_settings = radio.slice_mut_current(|state| &mut state.channel_settings_page);

        let mut name = use_initial(move || self.webhook.name.clone());
        let mut error = use_state(|| None);

        let edit_webhook = {
            let webhook_id = self.webhook.id.clone();
            let mut webhooks = self.webhooks;

            move |payload| {
                let webhook_id = webhook_id.clone();

                async move {
                    match http().edit_webhook(&webhook_id, &payload).await {
                        Ok(webhook) => {
                            webhooks.write().insert(webhook.id.clone(), webhook.clone());
                            Some(webhook)
                        },
                        Err(e) => {
                            error.set(Some(e));
                            None
                        }
                    }
                }
            }
        };

        let remove_field = {
            let edit_webhook = edit_webhook.clone();

            move |field| {
                let edit_webhook = edit_webhook.clone();

                async move {
                    edit_webhook(v0::DataEditWebhook {
                        name: None,
                        avatar: None,
                        permissions: None,
                        remove: vec![field],
                    })
                    .await
                }
            }
        };

        rect().spacing(32.).child(
            rect()
                .spacing(15.)
                .child(label().text("Webhook Icon").font_size(12.))
                .child(
                    rect()
                        .horizontal()
                        .spacing(15.)
                        .cross_align(Alignment::Center)
                        .child(
                            StoatButton::new()
                                .corner_radius(48.)
                                .on_press({
                                    let edit_webhook = edit_webhook.clone();

                                    move |_| {
                                        let edit_webhook = edit_webhook.clone();

                                        spawn(async move {
                                            if let Some(id) =
                                                prompt_image_upload(Tag::Avatars).await
                                            {
                                                edit_webhook(v0::DataEditWebhook {
                                                    name: None,
                                                    avatar: Some(id),
                                                    permissions: None,
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
                                        .maybe_child(self.webhook.avatar.as_ref().map(|icon| {
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
                                .enabled(self.webhook.avatar.is_some())
                                .color(theme.md.primary.as_argb_u32())
                                .on_press({
                                    let remove_field = remove_field.clone();

                                    move |_| {
                                        let remove_field = remove_field.clone();

                                        spawn(async move {
                                            remove_field(v0::FieldsWebhook::Avatar).await;
                                        });
                                    }
                                })
                                .child(
                                    rect().size(Size::px(36.)).center().child(
                                        MaterialIcon::new(clear())
                                            .size(Size::px(24.))
                                    ),
                                ),
                        ),
                )
                .child(SingleLineEntry::new("Webhook Name", name))
                .child(
                    rect()
                        .horizontal()
                        .spacing(8.)
                        .font_size(14)
                        .child(
                            StoatButton::new()
                                .color(theme.md.primary.as_argb_u32())
                                .corner_radius(40.)
                                .enabled(name.is_different())
                                .child(
                                    rect()
                                        .height(Size::px(40.))
                                        .padding((0., 16.))
                                        .center()
                                        .child("Reset"),
                                )
                                .on_press(move |_| {
                                    name.reset();
                                }),
                        )
                        .child(
                            StoatButton::new()
                                .color(theme.md.on_primary.as_argb_u32())
                                .background(theme.md.primary.as_argb_u32())
                                .corner_radius(40.)
                                .on_press({
                                    let edit_webhook = edit_webhook.clone();

                                    move |_| {
                                        let edit_webhook = edit_webhook.clone();

                                        spawn({
                                            async move {
                                                let payload = v0::DataEditWebhook {
                                                    name: name.get_if_different(),
                                                    avatar: None,
                                                    permissions: None,
                                                    remove: Vec::new(),
                                                };

                                                if edit_webhook(payload).await.is_some() {
                                                    name.apply();
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
                        .spacing(8.)
                        .child(
                            StoatButton::new()
                                .corner_radius(12.)
                                .on_press({
                                    let id = self.webhook.id.clone();
                                    let token = self.webhook.token.clone().unwrap();

                                    move |_| {
                                        Clipboard::set(format!(
                                            "https://stoat.chat/api/webhooks/{id}/{token}"
                                        ))
                                        .unwrap();
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
                                                            MaterialIcon::new(link())
                                                                .size(Size::px(22.)),
                                                        ),
                                                )
                                                .child(
                                                    label()
                                                        .width(Size::flex(1.))
                                                        .font_size(14.)
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .line_height(1.5)
                                                        .text("Copy Webhook URL"),
                                                )
                                                .child(
                                                    MaterialIcon::new(content_copy())
                                                        .size(Size::px(18.)),
                                                ),
                                        ),
                                ),
                        )
                        .child(
                            StoatButton::new()
                                .corner_radius(12.)
                                .on_press({
                                    let id = self.webhook.id.clone();
                                    let mut webhooks = self.webhooks;

                                    move |_| {
                                        modals.write().push_modal(ModalValue::DeleteWebhook {
                                            id: id.clone(),
                                            callback: EventHandler::new({
                                                let mut channel_settings = channel_settings.clone();
                                                let id = id.clone();

                                                move |_| {
                                                    webhooks.write().remove(&id);
                                                    if let Some((_, page)) =
                                                        &mut *channel_settings.write()
                                                    {
                                                        page.go_back();
                                                    }
                                                }
                                            }),
                                        });
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
                                                        .color(theme.md.error.as_argb_u32())
                                                        .center()
                                                        .child(
                                                            MaterialIcon::new(delete())
                                                                .size(Size::px(22.)),
                                                        ),
                                                )
                                                .child(
                                                    label()
                                                        .width(Size::flex(1.))
                                                        .font_size(14.)
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .line_height(1.5)
                                                        .text("Delete Bot"),
                                                )
                                                .child(
                                                    MaterialIcon::new(chevron_right())
                                                        .size(Size::px(18.)),
                                                ),
                                        ),
                                ),
                        ),
                ),
        )
    }
}
