use crate::{
    LocalFile,
    components::{
        Dropdown, SingleLineEntry, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt, image,
    },
    http,
    types::Tag,
    use_initial, use_material_theme,
};
use freya::{icons::lucide::x, prelude::*};
use rfd::AsyncFileDialog;
use stoat_models::v0;

#[derive(PartialEq)]
pub struct OverviewChannelSettings {
    pub channel: Readable<v0::Channel>,
}

impl Component for OverviewChannelSettings {
    fn render(&self) -> impl IntoElement {
        let theme = use_material_theme();
        let mut error = use_state(|| None);

        let edit_channel = {
            let channel_id = self.channel.read().id().to_string();

            move |payload| {
                let channel_id = channel_id.clone();
                async move {
                    match http().edit_channel(&channel_id, &payload).await {
                        Ok(channel) => Some(channel),
                        Err(e) => {
                            error.set(Some(e));
                            None
                        }
                    }
                }
            }
        };

        let remove_field = {
            let edit_channel = edit_channel.clone();

            move |field| {
                let edit_channel = edit_channel.clone();

                async move {
                    edit_channel(v0::DataEditChannel {
                        name: None,
                        description: None,
                        owner: None,
                        icon: None,
                        nsfw: None,
                        archived: None,
                        voice: None,
                        slowmode: None,
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

        let current_channel = self.channel.read();

        let mut channel_name = use_initial(|| {
            let (v0::Channel::TextChannel { name, .. } | v0::Channel::Group { name, .. }) =
                &*current_channel
            else {
                unreachable!()
            };

            name.clone()
        });

        let mut channel_description = use_initial(|| {
            let (v0::Channel::TextChannel { description, .. }
            | v0::Channel::Group { description, .. }) = &*current_channel
            else {
                unreachable!()
            };

            description.clone().unwrap_or_default()
        });

        let mut channel_slowmode = use_initial(|| {
            if let v0::Channel::TextChannel { slowmode, .. } = &*current_channel {
                slowmode.unwrap_or_default()
            } else {
                0
            }
        });

        let icon = match &*current_channel {
            v0::Channel::TextChannel { icon, .. } | v0::Channel::Group { icon, .. } => icon.clone(),
            _ => None,
        };

        let is_text_channel = matches!(&*current_channel, v0::Channel::TextChannel { .. });

        rect()
            .spacing(8.)
            .child(label().text("Channel Icon").font_size(12.))
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
                                let edit_server = edit_channel.clone();

                                move |_| {
                                    let prompt_image_upload = prompt_image_upload.clone();
                                    let edit_server = edit_server.clone();

                                    spawn(async move {
                                        if let Some(id) = prompt_image_upload(Tag::Icons).await {
                                            edit_server(v0::DataEditChannel {
                                                name: None,
                                                description: None,
                                                owner: None,
                                                icon: Some(id),
                                                nsfw: None,
                                                archived: None,
                                                voice: None,
                                                slowmode: None,
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
                                    .maybe_child(icon.as_ref().map(|icon| {
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
                                        remove_field(v0::FieldsChannel::Icon).await;
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
            .child(SingleLineEntry::new("Channel Name", channel_name).margin((0., 0., 24., 0.)))
            .child(
                SingleLineEntry::new("Channel Description", channel_description)
                    .placeholder("This channel is about...")
                    .margin((0., 0., 24., 0.)),
            )
            .maybe_child(is_text_channel.then(|| label().text("Channel Slowmode").font_size(12.)))
            .maybe_child(is_text_channel.then(|| {
                Dropdown::new(
                    channel_slowmode,
                    vec![
                        0,
                        5,
                        10,
                        30,
                        60,
                        60 * 5,
                        60 * 10,
                        60 * 30,
                        60 * 60,
                        60 * 60 * 2,
                        60 * 60 * 6,
                    ],
                    |duration| label().text(format!("{duration} seconds")).into_element(),
                )
            }))
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
                                channel_name.reset();
                                channel_description.reset();
                                channel_slowmode.reset();
                            }),
                    )
                    .child(
                        StoatButton::new()
                            .color(theme.md.on_primary.as_argb_u32())
                            .background(theme.md.primary.as_argb_u32())
                            .corner_radius(40.)
                            .on_press({
                                move |_| {
                                    let edit_channel = edit_channel.clone();

                                    spawn({
                                        async move {
                                            let payload = v0::DataEditChannel {
                                                name: channel_name.get_if_different(),
                                                description: channel_description.get_if_different(),
                                                icon: None,
                                                owner: None,
                                                nsfw: None,
                                                archived: None,
                                                voice: None,
                                                slowmode: channel_slowmode.get_if_different(),
                                                remove: Vec::new(),
                                            };

                                            if let Some(new_channel) = edit_channel(payload).await {
                                                match new_channel {
                                                    v0::Channel::Group {
                                                        name,
                                                        description,
                                                        ..
                                                    } => {
                                                        channel_name.set_new(name);

                                                        if let Some(description) = description {
                                                            channel_description
                                                                .set_new(description);
                                                        }
                                                    }
                                                    v0::Channel::TextChannel {
                                                        name,
                                                        description,
                                                        slowmode,
                                                        ..
                                                    } => {
                                                        channel_name.set_new(name);

                                                        if let Some(description) = description {
                                                            channel_description
                                                                .set_new(description);
                                                        }

                                                        if let Some(slowmode) = slowmode {
                                                            channel_slowmode.set_new(slowmode);
                                                        }
                                                    }
                                                    _ => {}
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
