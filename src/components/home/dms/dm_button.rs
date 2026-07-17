use std::borrow::Cow;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, calculate_channel_permissions,
    components::{
        Avatar, ChannelContextMenu, HomeSelection, StoatButton, StoatButtonLayoutThemePartialExt,
        file_image,
    },
    consume_material_theme, user_permissions_query,
};

#[derive(PartialEq)]
pub struct DMButton {
    pub channel: Readable<v0::Channel>,
    pub selection: State<HomeSelection>,
}

impl Component for DMButton {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let user_id = radio.read().user_id.clone().unwrap();
        let theme = consume_material_theme();

        rect()
            .on_secondary_down({
                let channel = self.channel.clone();
                let radio = radio.clone();

                move |e| {
                    let channel = channel.read().clone();
                    let radio = radio.clone();

                    spawn(async move {
                        let mut query = user_permissions_query(radio).channel(channel.clone());

                        let permissions = calculate_channel_permissions(&mut query).await;

                        ContextMenu::open_from_event(
                            &e,
                            Menu::new().child(ChannelContextMenu {
                                channel_id: channel.id().to_string(),
                                current_permissions: permissions,
                            }),
                        );
                    });
                }
            })
            .child(
                StoatButton::new()
                    .corner_radius(42.)
                    .child(
                        rect()
                            .padding((0., 8., 0., 8.))
                            .spacing(8.)
                            .height(Size::px(42.))
                            .width(Size::Fill)
                            .main_align(Alignment::Center)
                            .color(theme.md.outline.as_argb_u32())
                            .maybe(
                                self.selection.read().channel_id()
                                    == Some(self.channel.read().id()),
                                |btn| {
                                    btn.background(theme.md.primary_container.as_argb_u32())
                                        .color(theme.md.on_primary_container.as_argb_u32())
                                },
                            )
                            .child(match self.channel.read().clone() {
                                v0::Channel::DirectMessage { recipients, .. } => {
                                    let other = recipients
                                        .iter()
                                        .find(|&id| id != &user_id)
                                        .unwrap()
                                        .clone();

                                    let user = radio.slice(AppChannel::Users, move |state| {
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
                            }),
                    )
                    .on_press({
                        let channel = self.channel.clone();
                        let mut selection = self.selection.clone();

                        move |_| {
                            let id = channel.read().id().to_string();

                            *selection.write() = HomeSelection::DM(id);
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
pub struct DMGroupButton {
    pub channel: Readable<v0::Channel>,
}

impl Component for DMGroupButton {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let (name, icon, users) = match &*self.channel.read() {
            v0::Channel::Group {
                name,
                icon,
                recipients,
                ..
            } => (name.clone(), icon.clone(), recipients.len()),
            _ => unreachable!(),
        };

        rect()
            .horizontal()
            .spacing(8.)
            .child(
                rect()
                    .width(Size::px(32.))
                    .height(Size::px(32.))
                    .corner_radius(12.)
                    .overflow(Overflow::Clip)
                    .child(match icon {
                        Some(icon) => file_image(&icon).into_element(),
                        None => {
                            let initials = name
                                .trim()
                                .split_whitespace()
                                .filter_map(|run| run.chars().next())
                                .take(2)
                                .collect::<String>();

                            rect()
                                .background(theme.md.primary.as_argb_u32())
                                .width(Size::Fill)
                                .height(Size::Fill)
                                .center()
                                .font_size(12)
                                .child(initials)
                                .color(theme.md.on_primary.as_argb_u32())
                                .into_element()
                        }
                    }),
            )
            .child(
                rect()
                    .child(label().text(name).font_size(15))
                    .child(label().text(format!("{users} Members")).font_size(11)),
            )
    }
}
