use std::{borrow::Cow, collections::HashMap};

use freya::prelude::*;
use stoat_models::v0;

use crate::{
    SizeExt,
    components::{
        MaterialIcon, SingleLineEntry, StoatButton, StoatButtonLayoutThemePartialExt, file_image,
        material::outlined::{clear, delete},
        stoat_scrollbar,
    },
    consume_material_theme, http,
};

#[derive(PartialEq)]
pub struct BanServerSettings {
    pub server: Readable<v0::Server>,
}

#[derive(Debug, Clone, PartialEq)]
enum BannedUser {
    User(v0::BannedUser),
    Id(String),
}

impl BannedUser {
    pub fn id(&self) -> &str {
        match self {
            BannedUser::User(user) => &user.id,
            BannedUser::Id(id) => id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            BannedUser::User(user) => &user.username,
            BannedUser::Id(_) => "Unknown User",
        }
    }

    pub fn formatted_name(&self) -> Cow<'static, str> {
        match self {
            BannedUser::User(user) => format!("{}#{}", user.username, user.discriminator).into(),
            BannedUser::Id(_) => "Unknown User".into(),
        }
    }

    pub fn avatar(&self) -> Option<&v0::File> {
        match self {
            BannedUser::User(banned_user) => banned_user.avatar.as_ref(),
            BannedUser::Id(_) => None,
        }
    }
}

impl Component for BanServerSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let mut bans = use_state(|| Vec::new());

        use_hook({
            let server_id = self.server.read().id.clone();

            move || {
                spawn(async move {
                    if let Ok(resp) = http().fetch_bans(&server_id).await {
                        let mut users = HashMap::new();

                        for user in resp.users {
                            users.insert(user.id.clone(), user);
                        }

                        let mut bans = bans.write();

                        for ban in resp.bans {
                            let user = users.remove(&ban.id.user).map_or_else(
                                || BannedUser::Id(ban.id.user.clone()),
                                BannedUser::User,
                            );

                            bans.push((user, ban.reason));
                        }
                    };
                });
            }
        });

        let mut user_filter = use_state(|| String::new());
        let mut reason_filter = use_state(|| String::new());

        let filtered_bans = use_side_effect_value({
            move || {
                let mut output = Vec::new();

                let user_filter = user_filter.read();
                let reason_filter = reason_filter.read();

                for (user, reason) in bans.read().iter() {
                    if !user_filter.is_empty() && !user.name().starts_with(&*user_filter) {
                        continue;
                    }

                    if !reason_filter.is_empty()
                        && !reason
                            .as_ref()
                            .is_some_and(|r| r.starts_with(&*reason_filter))
                    {
                        continue;
                    }

                    output.push((user.clone(), reason.clone()));
                }

                output
            }
        });

        rect()
            .corner_radius(12.)
            .overflow(Overflow::Clip)
            .border(
                Border::new()
                    .width(1.)
                    .fill(theme.md.outline_variant.as_argb_u32()),
            )
            .child(
                rect()
                    .horizontal()
                    .content(Content::Flex)
                    .width(Size::Fill)
                    .color(theme.md.on_surface_variant.as_argb_u32())
                    .border(
                        Border::new()
                            .width(BorderWidth {
                                top: 0.,
                                right: 0.,
                                bottom: 1.,
                                left: 0.,
                            })
                            .fill(theme.md.outline_variant.as_argb_u32()),
                    )
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .padding(15.)
                            .child(SingleLineEntry::new("User", user_filter)),
                    )
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .padding(15.)
                            .child(SingleLineEntry::new("Reason", reason_filter)),
                    )
                    .child(
                        rect().padding(15.).height(Size::px(82.)).center().child(
                            StoatButton::new()
                                .corner_radius(18.)
                                .enabled(
                                    !user_filter.read().is_empty()
                                        || !reason_filter.read().is_empty(),
                                )
                                .tooltip(label().max_lines(1).font_size(11.).text("Clear Filter"))
                                .on_press(move |_| {
                                    user_filter.write().clear();
                                    reason_filter.write().clear();
                                })
                                .child(
                                    rect().size(Size::px(36.)).center().child(
                                        MaterialIcon::new(clear())
                                            .color(theme.md.on_surface.as_argb_u32())
                                            .size(Size::px(24.)),
                                    ),
                                ),
                        ),
                    ),
            )
            .child(
                VirtualScrollView::new({
                    let server_id = self.server.read().id.clone();

                    move |item, _| {
                        let (user, reason) = &filtered_bans.read()[item.index];

                        rect()
                            .key(user.id())
                            .horizontal()
                            .cross_align(Alignment::Center)
                            .content(Content::Flex)
                            .width(Size::Fill)
                            .border(
                                Border::new()
                                    .width(BorderWidth {
                                        top: 0.,
                                        right: 0.,
                                        bottom: 1.,
                                        left: 0.,
                                    })
                                    .fill(theme.md.outline_variant.as_argb_u32()),
                            )
                            .child(
                                rect()
                                    .horizontal()
                                    .width(Size::flex(1.))
                                    .padding(15.)
                                    .spacing(8.)
                                    .cross_align(Alignment::Center)
                                    .child(if let Some(avatar) = user.avatar() {
                                        file_image(avatar)
                                            .size(Size::px(32.))
                                            .aspect_ratio(AspectRatio::Max)
                                            .image_cover(ImageCover::Center)
                                            .corner_radius(16.)
                                            .overflow(Overflow::Clip)
                                            .into_element()
                                    } else {
                                        rect()
                                            .size(Size::px(32.))
                                            .center()
                                            .child(label().font_size(12.).font_weight(600).text(
                                                user.name().chars().next().unwrap().to_string(),
                                            ))
                                            .into_element()
                                    })
                                    .child(
                                        label()
                                            .font_size(16.)
                                            .max_lines(1)
                                            .text_overflow(TextOverflow::Ellipsis)
                                            .text(user.formatted_name()),
                                    ),
                            )
                            .child(
                                rect()
                                    .horizontal()
                                    .width(Size::flex(1.))
                                    .padding(15.)
                                    .cross_align(Alignment::Center)
                                    .child(
                                        label()
                                            .max_lines(1)
                                            .text_overflow(TextOverflow::Ellipsis)
                                            .text(
                                                reason
                                                    .clone()
                                                    .unwrap_or_else(|| "No reason".to_string()),
                                            ),
                                    ),
                            )
                            .child(
                                rect().padding(15.).child(
                                    StoatButton::new()
                                        .corner_radius(18.)
                                        .tooltip(
                                            label().max_lines(1).font_size(11.).text("Pardon User"),
                                        )
                                        .on_press({
                                            let server_id = server_id.clone();
                                            let id = user.id().to_string();

                                            move |_| {
                                                let server_id = server_id.clone();
                                                let id = id.clone();

                                                spawn(async move {
                                                    if http()
                                                        .unban_member(&server_id, &id)
                                                        .await
                                                        .is_ok()
                                                    {
                                                        let mut bans = bans.write();
                                                        if let Some(idx) = bans
                                                            .iter()
                                                            .position(|(user, _)| user.id() == &id)
                                                        {
                                                            bans.remove(idx);
                                                        };
                                                    };
                                                });
                                            }
                                        })
                                        .child(
                                            rect()
                                                .size(Size::px(36.))
                                                .background(theme.md.primary.as_argb_u32())
                                                .color(theme.md.on_primary.as_argb_u32())
                                                .center()
                                                .child(
                                                    MaterialIcon::new(delete()).size(Size::px(24.)),
                                                ),
                                        ),
                                ),
                            )
                            .into_element()
                    }
                })
                .scrollbar(stoat_scrollbar)
                .length(filtered_bans.read().len())
                .item_size(66.)
                .min_height(Size::Inner)
                .max_height(Size::Fill),
            )
    }
}
