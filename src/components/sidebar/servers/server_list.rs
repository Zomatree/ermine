use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, NotificationBadge, Selection, SettingsPage, SizeExt,
    components::{
        Avatar, CurrentUserButton, GroupDMIcon, HomeButton, ModalValue, ReorderServers,
        StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip,
        material::{
            MaterialIcon,
            filled::{add, explore, settings},
        },
        stoat_scrollbar, use_modals,
    },
    consume_material_theme, get_unread_badge,
};

#[derive(PartialEq)]
pub struct ServerList {}

impl Component for ServerList {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let radio = use_radio(AppChannel::Servers);

        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);
        let unreads = radio.slice(AppChannel::ChannelUnreads, |state| &state.channel_unreads);
        let users = radio.slice(AppChannel::Users, |state| &state.users);
        let mut settings_page =
            radio.slice_mut(AppChannel::SettingsPage, |state| &mut state.settings_page);
        let mut selection = radio.slice_mut(AppChannel::Selection, |state| &mut state.selection);
        let selected_channel = radio.slice_mut(AppChannel::SelectedChannel, |state| {
            &mut state.selected_channel
        });

        let user_id = radio.read().user_id.clone().unwrap();

        let unread_channels = use_side_effect_value({
            let channels = channels.clone();

            move || {
                let unreads = unreads.read();
                let mut unread_channels = Vec::new();

                for channel in channels.read().values() {
                    if let v0::Channel::DirectMessage {
                        id,
                        last_message_id,
                        ..
                    }
                    | v0::Channel::Group {
                        id,
                        last_message_id,
                        ..
                    } = channel
                    {
                        if let Some(unread) = unreads.get(id)
                            && let Some(badge) = get_unread_badge(&channel, unread)
                        {
                            unread_channels.push((
                                channel.clone(),
                                last_message_id.as_ref().unwrap_or(&id).clone(),
                                badge,
                            ));
                        }
                    }
                }

                unread_channels.sort_by(|a, b| b.1.cmp(&a.1));
                unread_channels
                    .into_iter()
                    .take(9)
                    .map(|(c, _, badge)| (c, badge))
                    .collect::<Vec<_>>()
            }
        });

        rect()
            .child(
                ScrollView::new()
                    .scrollbar(stoat_scrollbar)
                    .child(
                        rect()
                            .width(Size::fill())
                            .cross_align(Alignment::Center)
                            .child(HomeButton {})
                            .child(CurrentUserButton {})
                            .children(unread_channels.read().iter().cloned().map(
                                |(channel, badge)| {
                                    let (title, icon) = match &channel {
                                        v0::Channel::DirectMessage { recipients, .. } => {
                                            let other = recipients
                                                .iter()
                                                .find(|&id| id != &user_id)
                                                .unwrap();

                                            let user = users.read().get(other).cloned().unwrap();

                                            (
                                                user.display_name
                                                    .as_ref()
                                                    .unwrap_or(&user.username)
                                                    .clone(),
                                                Avatar::new(user, None, 42.).into_element(),
                                            )
                                        }
                                        v0::Channel::Group { name, .. } => (
                                            name.clone(),
                                            GroupDMIcon::new(channel.clone())
                                                .size(42.)
                                                .invert(true)
                                                .into_element(),
                                        ),
                                        _ => unreachable!(),
                                    };

                                    StoatTooltip::new(
                                        label().max_lines(1).text(title).font_size(11.),
                                    )
                                    .position(AttachedPosition::Right)
                                    .child(
                                        rect().size(Size::px(56.)).center().child(
                                            rect()
                                                .size(Size::px(42.))
                                                .child(
                                                    StoatButton::new()
                                                        .corner_radius(
                                                            if matches!(
                                                                &channel,
                                                                v0::Channel::DirectMessage { .. }
                                                            ) {
                                                                21.
                                                            } else {
                                                                15.75
                                                            },
                                                        )
                                                        .on_press({
                                                            let mut selection = selection.clone();
                                                            let mut selected_channel =
                                                                selected_channel.clone();

                                                            let channel_id =
                                                                channel.id().to_string();

                                                            move |_| {
                                                                *selection.write() =
                                                                    Selection::Home;
                                                                *selected_channel.write() = Some((
                                                                    channel_id.clone(),
                                                                    None,
                                                                ));
                                                            }
                                                        })
                                                        .child(
                                                            rect().size(Size::px(42.)).child(icon),
                                                        ),
                                                )
                                                .child(
                                                    rect()
                                                        .position(
                                                            Position::new_absolute()
                                                                .right(0.)
                                                                .top(0.),
                                                        )
                                                        .layer(Layer::Relative(10))
                                                        .width(Size::px(13.))
                                                        .height(Size::px(13.))
                                                        .corner_radius(13.)
                                                        .center()
                                                        .with(|this| match badge {
                                                            NotificationBadge::Unread => {
                                                                this.background(theme.md.on_surface)
                                                            }
                                                            crate::NotificationBadge::Mentions(
                                                                count,
                                                            ) => this
                                                                .background(
                                                                    theme.md.error.as_argb_u32(),
                                                                )
                                                                .color(
                                                                    theme.md.on_error.as_argb_u32(),
                                                                )
                                                                .font_size(8.)
                                                                .font_weight(FontWeight::SEMI_BOLD)
                                                                .child(if count <= 9 {
                                                                    count.to_string()
                                                                } else {
                                                                    "+".to_string()
                                                                }),
                                                        }),
                                                ),
                                        ),
                                    )
                                },
                            ))
                            .child(
                                rect()
                                    .height(Size::px(1.))
                                    .width(Size::px(32.))
                                    .margin((6., 0.))
                                    .background(theme.md.outline_variant.as_argb_u32()),
                            )
                            .child(ReorderServers {})
                            .child(
                                StoatTooltip::new(
                                    label()
                                        .max_lines(1)
                                        .font_size(11.)
                                        .text("Create or join a server"),
                                )
                                .position(AttachedPosition::Right)
                                .child(
                                    rect()
                                        .width(Size::px(56.))
                                        .height(Size::px(56.))
                                        .center()
                                        .child(
                                            StoatButton::new()
                                                .corner_radius(42.)
                                                .child(
                                                    rect()
                                                        .center()
                                                        .width(Size::px(42.0))
                                                        .height(Size::px(42.0))
                                                        .background(
                                                            theme
                                                                .md
                                                                .surface_container_low
                                                                .as_argb_u32(),
                                                        )
                                                        .child(
                                                            MaterialIcon::new(add())
                                                                .size(Size::px(32.0)),
                                                        ),
                                                )
                                                .on_press(move |_| {
                                                    modals
                                                        .write()
                                                        .push_modal(ModalValue::CreateJoinServer);
                                                }),
                                        ),
                                ),
                            )
                            .child(
                                StoatTooltip::new(
                                    label()
                                        .max_lines(1)
                                        .font_size(11.)
                                        .text("Find new servers to join"),
                                )
                                .position(AttachedPosition::Right)
                                .child(
                                    rect()
                                        .width(Size::px(56.))
                                        .height(Size::px(56.))
                                        .center()
                                        .child(
                                            StoatButton::new()
                                                .corner_radius(42.)
                                                .child(
                                                    rect()
                                                        .center()
                                                        .width(Size::px(42.0))
                                                        .height(Size::px(42.0))
                                                        .background(
                                                            theme
                                                                .md
                                                                .surface_container_low
                                                                .as_argb_u32(),
                                                        )
                                                        .child(
                                                            MaterialIcon::new(explore())
                                                                .width(Size::px(32.0))
                                                                .height(Size::px(32.0)),
                                                        ),
                                                )
                                                .on_press(move |_| {
                                                    *selection.write() = Selection::Discover;
                                                }),
                                        ),
                                ),
                            ),
                    )
                    .show_scrollbar(false)
                    .width(Size::px(56.))
                    .height(Size::func(|size| Some(size.parent - 56.))),
            )
            .child(
                StoatTooltip::new(label().max_lines(1).font_size(11.).text("Settings"))
                    .position(AttachedPosition::Right)
                    .child(
                        rect()
                            .width(Size::px(56.))
                            .height(Size::px(56.))
                            .center()
                            .child(
                                StoatButton::new()
                                    .corner_radius(42.)
                                    .child(
                                        rect()
                                            .center()
                                            .width(Size::px(42.0))
                                            .height(Size::px(42.0))
                                            .overflow(Overflow::Clip)
                                            .background(
                                                theme.md.surface_container_low.as_argb_u32(),
                                            )
                                            .child(
                                                MaterialIcon::new(settings()).size(Size::px(32.0)),
                                            ),
                                    )
                                    .on_press(move |_| {
                                        *settings_page.write() = Some(SettingsPage::default());
                                    }),
                            ),
                    ),
            )
    }
}
