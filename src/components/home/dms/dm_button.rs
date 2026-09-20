use std::borrow::Cow;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, NotificationBadge, SizeExt, calculate_channel_permissions,
    components::{
        Avatar, ChannelContextMenu, HomeSelection, StoatButton, StoatButtonLayoutThemePartialExt,
        file_image,
    },
    consume_material_theme, get_unread_badge, user_permissions_query,
};

#[derive(PartialEq)]
pub struct DMButton {
    pub channel: Readable<v0::Channel>,
}

impl Component for DMButton {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let dm_channel = radio.slice_mut(AppChannel::SelectedChannel, |state| {
            &mut state.selected_channel
        });
        let unreads = radio.slice(AppChannel::ChannelUnreads, |state| &state.channel_unreads);

        let user_id = radio.read().user_id.clone().unwrap();
        let theme = consume_material_theme();

        let unread_badge = use_memo({
            let channel = self.channel.clone();

            move || {
                let channel = channel.read();
                let unread = unreads.read().get(channel.id()).cloned()?;
                get_unread_badge(&channel, &unread)
            }
        });

        let selected = dm_channel
            .read()
            .as_ref()
            .is_some_and(|(id, _)| id == self.channel.read().id());

        let color = if selected {
            theme.md.on_primary_container
        } else if unread_badge.read().is_some() {
            theme.md.on_surface
        } else {
            theme.md.outline
        }
        .as_argb_u32();

        rect()
            .on_secondary_down({
                let channel = self.channel.clone();
                let radio = radio.clone();

                move |_| {
                    let channel = channel.read().clone();
                    let radio = radio.clone();

                    spawn(async move {
                        let mut query = user_permissions_query(radio).channel(channel.clone());

                        let permissions = calculate_channel_permissions(&mut query).await;

                        ContextMenu::open_from_down(Menu::new().child(ChannelContextMenu {
                            channel_id: channel.id().to_string(),
                            current_permissions: permissions,
                        }));
                    });
                }
            })
            .child(
                StoatButton::new()
                    .corner_radius(42.)
                    .child(
                        rect()
                            .spacing(8.)
                            .height(Size::px(42.))
                            .horizontal()
                            .content(Content::Flex)
                            .cross_align(Alignment::Center)
                            .color(color)
                            .maybe(selected, |btn| {
                                btn.background(theme.md.primary_container.as_argb_u32())
                            })
                            .child(
                                rect().width(Size::flex(1.)).padding((0., 8.)).child(
                                    match self.channel.read().clone() {
                                        v0::Channel::DirectMessage { recipients, .. } => {
                                            let other = recipients
                                                .iter()
                                                .find(|&id| id != &user_id)
                                                .unwrap()
                                                .clone();

                                            let user = radio
                                                .slice(AppChannel::Users, move |state| {
                                                    state.users.get(&other).unwrap()
                                                });

                                            DMDirectMessageButton {
                                                user: user.into_readable(),
                                            }
                                            .into_element()
                                        }
                                        v0::Channel::Group { .. } => DMGroupButton {
                                            channel: self.channel.clone(),
                                        }
                                        .into_element(),
                                        _ => unreachable!(),
                                    },
                                ),
                            )
                            .maybe_child(unread_badge.read().filter(|_| !selected).map(|badge| {
                                rect().padding((0., 8.)).child(
                                match badge {
                                    NotificationBadge::Mentions(count) => rect()
                                        .corner_radius(14.)
                                        .size(Size::px(14.))
                                        .center()
                                        .background(theme.md.error.as_argb_u32())
                                        .color(theme.md.on_error.as_argb_u32())
                                        .color(0xff690005)
                                        .font_size(8.)
                                        .child(if count <= 9 {
                                            count.to_string()
                                        } else {
                                            "+".to_string()
                                        }),
                                    NotificationBadge::Unread => rect()
                                        .corner_radius(7.)
                                        .size(Size::px(7.))
                                        .background(theme.md.on_surface.as_argb_u32())
                                        .margin((0., 3.5)),
                                })
                            })),
                    )
                    .on_press({
                        let channel = self.channel.clone();
                        let mut dm_channel = dm_channel.clone();

                        move |_| {
                            let id = channel.read().id().to_string();

                            dm_channel.set(Some((id.clone(), None)));
                        }
                    }),
            )
    }
}

#[derive(PartialEq)]
pub struct DMDirectMessageButton {
    pub user: Readable<v0::User>,
}

impl Component for DMDirectMessageButton {
    fn render(&self) -> impl IntoElement {
        let user = use_memo({
            let user = self.user.clone();
            move || user.read().clone()
        });

        rect()
            .horizontal()
            .height(Size::Fill)
            .cross_align(Alignment::Center)
            .spacing(8.)
            .child(Avatar::new(self.user.clone(), None, 32.).presence(true))
            .child(
                rect()
                    .child(
                        label()
                            .text({
                                let user = user.read();
                                user.display_name.as_ref().unwrap_or(&user.username).clone()
                            })
                            .font_size(15)
                            .max_lines(1)
                            .text_overflow(TextOverflow::Ellipsis),
                    )
                    .maybe_child(
                        user.read()
                            .status
                            .as_ref()
                            .and_then(|status| {
                                status
                                    .text
                                    .as_ref()
                                    .map(|text| Cow::Owned(text.clone()))
                                    .or(status.presence.as_ref().map(|presence| match presence {
                                        v0::Presence::Online => Cow::Borrowed("Online"),
                                        v0::Presence::Idle => Cow::Borrowed("Idle"),
                                        v0::Presence::Focus => Cow::Borrowed("Focus"),
                                        v0::Presence::Busy => Cow::Borrowed("Busy"),
                                        v0::Presence::Invisible => Cow::Borrowed("Invisible"),
                                    }))
                            })
                            .map(|text| {
                                label()
                                    .text(text)
                                    .font_size(11)
                                    .max_lines(1)
                                    .text_overflow(TextOverflow::Ellipsis)
                            }),
                    ),
            )
    }
}

#[derive(PartialEq)]
pub struct GroupDMIcon {
    channel: Readable<v0::Channel>,
    size: f32,
    invert: bool,
}

impl GroupDMIcon {
    pub fn new(channel: Readable<v0::Channel>) -> Self {
        Self {
            channel,
            size: 32.,
            invert: false,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;

        self
    }

    pub fn invert(mut self, invert: bool) -> Self {
        self.invert = invert;

        self
    }
}

impl Component for GroupDMIcon {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let (name, icon) = match &*self.channel.read() {
            v0::Channel::Group { name, icon, .. } => (name.clone(), icon.clone()),
            _ => unreachable!(),
        };

        rect()
            .size(Size::px(self.size))
            .corner_radius((12. / 32.) * self.size)
            .overflow(Overflow::Clip)
            .child(match icon {
                Some(icon) => file_image(&icon)
                    .aspect_ratio(AspectRatio::Max)
                    .image_cover(ImageCover::Center)
                    .expanded()
                    .into_element(),
                None => {
                    let initials = name
                        .trim()
                        .split_whitespace()
                        .filter_map(|run| run.chars().next())
                        .take(2)
                        .collect::<String>();

                    rect()
                        .background(
                            if self.invert {
                                theme.md.surface_container
                            } else {
                                theme.md.primary
                            }
                            .as_argb_u32(),
                        )
                        .width(Size::Fill)
                        .height(Size::Fill)
                        .center()
                        .font_size((12. / 32.) * self.size)
                        .child(initials)
                        .color(
                            if self.invert {
                                theme.md.on_surface
                            } else {
                                theme.md.on_primary
                            }
                            .as_argb_u32(),
                        )
                        .into_element()
                }
            })
    }
}

#[derive(PartialEq)]
pub struct DMGroupButton {
    pub channel: Readable<v0::Channel>,
}

impl Component for DMGroupButton {
    fn render(&self) -> impl IntoElement {
        let (name, users) = match &*self.channel.read() {
            v0::Channel::Group {
                name, recipients, ..
            } => (name.clone(), recipients.len()),
            _ => unreachable!(),
        };

        rect()
            .horizontal()
            .spacing(8.)
            .child(GroupDMIcon::new(self.channel.clone()))
            .child(
                rect()
                    .child(label().text(name).font_size(15))
                    .child(label().text(format!("{users} Members")).font_size(11)),
            )
    }
}
