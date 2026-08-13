use std::hash::Hash;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, ChannelSettingsPage, Config, NotificationBadge, SizeExt,
    calculate_channel_permissions,
    components::{
        ChannelContextMenu, MaterialIcon, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt, StoatTooltip,
        material::{
            filled::{grid_3x3, person_add_alt_1, settings},
            outlined::headset_mic,
        },
    },
    consume_material_theme, get_unread_badge, is_channel_muted, user_permissions_query,
};

#[derive(PartialEq)]
pub struct ChannelButton {
    pub channel: Readable<v0::Channel>,
}

impl Component for ChannelButton {
    fn render(&self) -> impl IntoElement {
        let mut config = use_consume::<State<Config>>();
        let mut radio = use_radio(AppChannel::SelectedChannel);
        let theme = consume_material_theme();
        let selected = radio.slice_current(|state| &state.selected_channel);
        let unreads = radio.slice(AppChannel::ChannelUnreads, |state| &state.channel_unreads);
        let mutes = radio.slice(AppChannel::Settings("notifications"), |state| {
            &state.settings.notifications
        });
        let channel_settings = radio.slice_mut(AppChannel::ChannelSettingsPage, |state| {
            &mut state.channel_settings_page
        });

        let mut hovering = use_state(|| false);

        let channel = use_memo({
            let channel = self.channel.clone();
            move || channel.read().clone()
        });

        let unread = use_memo({
            let channel = channel.clone();
            move || unreads.read().get(channel.read().id()).cloned()
        });

        let muted = use_memo({
            let channel = channel.clone();
            let mutes = mutes.into_readable();
            move || is_channel_muted(channel.read().id(), mutes.clone())
        });

        let unread_badge = use_memo({
            let channel = channel.clone();
            let muted = muted.clone();

            move || {
                if *muted.read() {
                    return None;
                };

                let unread = unread.read().clone()?;

                get_unread_badge(&channel.read(), &unread)
            }
        });

        let selected = use_memo(move || selected.read().as_deref() == Some(channel.read().id()));

        rect()
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .on_secondary_down({
                let channel = self.channel.clone();

                move |_| {
                    let channel = channel.read().clone();

                    spawn(async move {
                        let mut query = user_permissions_query(radio).channel(channel.clone());

                        let permissions = calculate_channel_permissions(&mut query).await;

                        ContextMenu::open_from_down(
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
                    .color(theme.md.outline.as_argb_u32())
                    .maybe(*selected.read(), |btn| {
                        btn.background(theme.md.primary_container.as_argb_u32())
                            .color(theme.md.on_primary_container.as_argb_u32())
                    })
                    .on_press({
                        let channel = channel.clone();

                        move |_| {
                            let channel_id = channel.read().id().to_string();

                            radio
                                .write_channel(AppChannel::SelectedChannel)
                                .selected_channel = Some(channel_id.clone());

                            let server_id = match &*channel.read() {
                                v0::Channel::TextChannel { server, .. } => Some(server.clone()),
                                _ => None,
                            };

                            if let Some(server_id) = server_id {
                                config.write().last_channels.insert(server_id, channel_id);
                            };
                        }
                    })
                    .child(
                        rect()
                            .horizontal()
                            .content(Content::Flex)
                            .padding((0., 8., 0., 8.))
                            .spacing(8.)
                            .height(Size::px(42.))
                            .cross_align(Alignment::Center)
                            .overflow(Overflow::Clip)
                            .font_size(15)
                            .width(Size::Fill)
                            .child(
                                MaterialIcon::new(
                                    if matches!(
                                        &*channel.read(),
                                        v0::Channel::TextChannel { voice: Some(_), .. }
                                    ) {
                                        headset_mic()
                                    } else {
                                        grid_3x3()
                                    },
                                )
                                .size(Size::px(24.)),
                            )
                            .child(
                                label()
                                    .text(channel.read().name().unwrap().to_string())
                                    .text_overflow(TextOverflow::Ellipsis)
                                    .width(Size::flex(1.))
                                    .max_lines(1),
                            )
                            .maybe(unread_badge.read().is_some(), |this| {
                                this.color(theme.md.on_surface.as_argb_u32())
                            })
                            .maybe_child(
                                unread_badge
                                    .read()
                                    .filter(|_| !hovering() && !selected())
                                    .map(|badge| match badge {
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
                                    }),
                            )
                            .maybe_child(hovering().then(|| {
                                rect()
                                    .horizontal()
                                    .spacing(4.)
                                    .child(
                                        StoatTooltip::new(
                                            label()
                                                .font_size(11.)
                                                .max_lines(1)
                                                .text("Create Invite"),
                                        )
                                        .position(AttachedPosition::Top)
                                        .child(
                                            MaterialIcon::new(person_add_alt_1())
                                                .size(Size::px(16.))
                                                .on_press(|e: Event<PressEventData>| {
                                                    e.stop_propagation();
                                                }),
                                        ),
                                    )
                                    .child(
                                        StoatTooltip::new(
                                            label()
                                                .font_size(11.)
                                                .max_lines(1)
                                                .text("Edit Channel"),
                                        )
                                        .position(AttachedPosition::Top)
                                        .child(
                                            MaterialIcon::new(settings())
                                                .width(Size::px(16.))
                                                .height(Size::px(16.))
                                                .on_press({
                                                    let id = self.channel.peek().id().to_string();

                                                    move |e: Event<PressEventData>| {
                                                        e.stop_propagation();

                                                        *channel_settings.clone().write() = Some((
                                                            id.clone(),
                                                            ChannelSettingsPage::default(),
                                                        ));
                                                    }
                                                }),
                                        ),
                                    )
                            })),
                    ),
            )
    }

    fn render_key(&self) -> DiffKey {
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::default();
        self.channel.peek().id().hash(&mut hasher);
        DiffKey::U64(hasher.finish())
    }
}
