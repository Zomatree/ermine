use crate::{
    AppChannel, LocalFile, SizeExt,
    components::{
        AnimatedImage, Avatar, MaterialIcon, ModalValue, SingleLineEntry, StoatButton,
        StoatButtonColorsThemePartialExt, StoatButtonLayoutThemePartialExt,
        material::{filled::delete, outlined::edit},
        stoat_label, use_modals,
    },
    consume_material_theme, http,
    types::Tag,
};
use freya::{prelude::*, radio::use_radio};
use rfd::AsyncFileDialog;
use stoat_models::v0;

#[derive(PartialEq)]
pub struct EmojiServerSettings {
    pub server: Readable<v0::Server>,
}

impl Component for EmojiServerSettings {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Emojis);
        let emojis = radio.slice_current(|state| &state.emojis);

        let theme = consume_material_theme();

        let mut selected_image = use_state(|| None);
        let mut emoji_name = use_state(String::new);

        let server_emojis = use_memo({
            let server_id = self.server.peek().id.clone();
            move || {
                let mut server_emojis = emojis
                    .read()
                    .values()
                    .filter(|emoji| match &emoji.parent {
                        v0::EmojiParent::Server { id } if id == &server_id => true,
                        _ => false,
                    })
                    .cloned()
                    .collect::<Vec<_>>();

                server_emojis.sort_by(|a, b| b.id.cmp(&a.id));
                server_emojis
            }
        });

        rect()
            .spacing(15.)
            .child(
                rect()
                    .horizontal()
                    .content(Content::Flex)
                    .spacing(8.)
                    .child(
                        rect()
                            .spacing(8.)
                            .child(
                                stoat_label()
                                    .font_size(12.)
                                    .font_weight(500)
                                    .text("Emoji")
                            )
                            .child(
                                StoatButton::new()
                                    .corner_radius(48.)
                                    .on_press({
                                        move |_| {
                                            spawn(async move {
                                                if let Some(file) = AsyncFileDialog::new()
                                                    .add_filter(
                                                        "Image",
                                                        &["png", "jpeg", "jpg", "webp", "gif", "webm"],
                                                    )
                                                    .pick_file()
                                                    .await
                                                {
                                                    selected_image.set(Some(file.path().to_path_buf()));
                                                };
                                            });
                                        }
                                    })
                                    .child(
                                        rect()
                                            .width(Size::px(96.))
                                            .height(Size::px(96.))
                                            .background(theme.md.surface_dim.as_argb_u32())
                                            .maybe_child(selected_image.read().as_ref().map(|image| {
                                                rect()
                                                    .layer(Layer::Relative(1))
                                                    .width(Size::Fill)
                                                    .height(Size::Fill)
                                                    .child(ImageViewer::new(ImageSource::Path(
                                                        image.clone(),
                                                    )))
                                            })),
                                    )
                            )
                    )
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .spacing(8.)
                            .child(SingleLineEntry::new("Emoji Name", emoji_name).max_length(32))
                            .child(
                                rect()
                                    .spacing(8.)
                                    .horizontal()
                                    .cross_align(Alignment::Center)
                                    .child(
                                        StoatButton::new()
                                            .color(theme.md.on_primary.as_argb_u32())
                                            .background(theme.md.primary.as_argb_u32())
                                            .corner_radius(40.)
                                            .enabled(!emoji_name.read().is_empty())
                                            .on_press({let server_id = self.server.peek().id.clone(); move |_| {
                                                let Some(path) = selected_image.read().cloned() else { return; };
                                                let filename = path.file_name().unwrap().to_string_lossy().to_string();
                                                let Ok(contents) = std::fs::read(path) else { return; };
                                                let server_id = server_id.clone();

                                                spawn(async move {
                                                    if let Ok(response) = http().upload_file(Tag::Emojis.as_str(), LocalFile {
                                                        name: filename,
                                                        body: contents.into(),
                                                    }).await {
                                                        http().create_emoji(&response.id, &v0::DataCreateEmoji {
                                                            name: emoji_name.read().cloned(),
                                                            parent: v0::EmojiParent::Server { id: server_id },
                                                            nsfw: false,
                                                        }).await.unwrap();

                                                        selected_image.set(None);
                                                        emoji_name.set(String::new());
                                                    }
                                                });
                                            } })
                                            .child(
                                                rect()
                                                    .height(Size::px(40.))
                                                    .padding((0., 16.))
                                                    .center()
                                                    .child("Create"),
                                            ),
                                    )
                                    .child(format!(
                                        "{} emoji slots remaining",
                                        100 - server_emojis.read().len()
                                    )),
                            ),
                    ),
            )
            .child(
                rect()
                    .children(server_emojis.read().iter().cloned().map(|emoji| {
                        EmojiServerSettingsEntry { emoji }
                    })),
            )
    }
}

#[derive(PartialEq)]
pub struct EmojiServerSettingsEntry {
    pub emoji: v0::Emoji,
}

impl Component for EmojiServerSettingsEntry {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let radio = use_radio(AppChannel::Users);
        let users = radio.slice_current(|state| &state.users);

        StoatButton::new().child(
            rect()
                .width(Size::Fill)
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(8.)
                .padding((8., 16.))
                .content(Content::Flex)
                .child(
                    rect()
                        .width(Size::px(32.))
                        .height(Size::px(32.))
                        .corner_radius(12.)
                        .overflow(Overflow::Clip)
                        .child(
                            AnimatedImage::new(
                                format!(
                                    "{}/{}/{}",
                                    http().api_config.features.autumn.url,
                                    Tag::Emojis,
                                    &self.emoji.id
                                )
                                .parse::<Url>()
                                .unwrap(),
                            )
                            .sampling_mode(SamplingMode::Trilinear)
                            .image_cover(ImageCover::Center)
                            .aspect_ratio(AspectRatio::Max)
                            .width(Size::Fill)
                            .height(Size::Fill)
                            .into_element(),
                        ),
                )
                .child(
                    rect()
                        .width(Size::flex(1.))
                        .child(
                            label()
                                .font_size(16.)
                                .text(format!(":{}:", &self.emoji.name)),
                        )
                        .child({
                            rect()
                                .font_size(12.)
                                .horizontal()
                                .spacing(4.)
                                .cross_align(Alignment::Center)
                                .with(|this| {
                                    if let Some(user) = users.read().get(&self.emoji.creator_id) {
                                        this.child(Avatar::new(user.clone(), None, 12.)).child(
                                            stoat_label().text(
                                                user.display_name
                                                    .as_ref()
                                                    .unwrap_or(&user.username)
                                                    .clone(),
                                            ),
                                        )
                                    } else {
                                        this.child(stoat_label().text("Unknown Author"))
                                    }
                                })
                        }),
                )
                .child(
                    StoatButton::new()
                        .corner_radius(16.)
                        .on_press({
                            let id = self.emoji.id.clone();

                            move |_| {
                                modals
                                    .write()
                                    .push_modal(ModalValue::EditEmoji { id: id.clone() });
                            }
                        })
                        .child(
                            rect().size(Size::px(36.)).center().child(
                                MaterialIcon::new(edit())
                                    .size(Size::px(24.))
                                    .color(theme.md.primary),
                            ),
                        ),
                )
                .child(
                    StoatButton::new()
                        .corner_radius(16.)
                        .on_press({
                            let id = self.emoji.id.clone();

                            move |_| {
                                modals
                                    .write()
                                    .push_modal(ModalValue::DeleteEmoji { id: id.clone() });
                            }
                        })
                        .child(
                            rect().size(Size::px(36.)).center().child(
                                MaterialIcon::new(delete())
                                    .size(Size::px(24.))
                                    .color(theme.md.primary),
                            ),
                        ),
                ),
        )
    }

    fn render_key(&self) -> DiffKey {
        (&self.emoji.id).into()
    }
}
