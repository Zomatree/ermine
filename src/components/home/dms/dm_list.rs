use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, SizeExt,
    components::{
        DMButton, HomeSelection, StoatButton, StoatButtonLayoutThemePartialExt,
        material::{
            MaterialIcon,
            outlined::{group, home, sticky_note_2},
        },
    },
    consume_material_theme, http,
    theme::Theme,
};

#[derive(PartialEq)]
pub struct DMList {
    pub selection: State<HomeSelection>,
}

impl Component for DMList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Channels);
        let dm_channel = radio.slice_mut(AppChannel::SelectedChannel, |state| {
            &mut state.selected_channel
        });
        let theme = consume_material_theme();

        let saved_messages = use_memo({
            let radio = radio.clone();

            move || {
                radio
                    .read()
                    .channels
                    .values()
                    .find(|channel| matches!(channel, v0::Channel::SavedMessages { .. }))
                    .cloned()
            }
        });

        let channels = use_memo(move || {
            let mut channels = radio
                .read()
                .channels
                .values()
                .filter_map(|channel| match channel {
                    v0::Channel::DirectMessage { id, .. } | v0::Channel::Group { id, .. } => {
                        let id = id.clone();

                        Some(
                            radio
                                .slice_current(move |state| state.channels.get(&id).unwrap())
                                .into_readable(),
                        )
                    }
                    _ => None,
                })
                .collect::<Vec<Readable<v0::Channel>>>();

            channels.sort_by(|a, b| {
                let (id_a, last_msg_a) = match &*a.read() {
                    v0::Channel::DirectMessage {
                        id,
                        last_message_id,
                        ..
                    }
                    | v0::Channel::Group {
                        id,
                        last_message_id,
                        ..
                    } => (id.clone(), last_message_id.clone()),
                    _ => unreachable!(),
                };

                let (id_b, last_msg_b) = match &*b.read() {
                    v0::Channel::DirectMessage {
                        id,
                        last_message_id,
                        ..
                    }
                    | v0::Channel::Group {
                        id,
                        last_message_id,
                        ..
                    } => (id.clone(), last_message_id.clone()),
                    _ => unreachable!(),
                };

                match (last_msg_a, last_msg_b) {
                    (Some(a), Some(b)) => b.cmp(&a),
                    (Some(a), None) => id_b.cmp(&a),
                    (None, Some(b)) => b.cmp(&id_a),
                    (None, None) => id_b.cmp(&id_a),
                }
            });

            channels
        });

        rect()
            .spacing(4.)
            .padding((0., 0., 0., 8.))
            .child(
                rect()
                    // .margin((8., 8., 0., 8.))
                    .padding((24., 16.0))
                    .font_size(16)
                    .child("Conversations"),
            )
            .child(
                rect()
                    // .padding((0., 8.))
                    .spacing(5.)
                    .child(
                        dmlist_nav_button(
                            home(),
                            "Home",
                            &theme,
                            &*self.selection.read() == &HomeSelection::Welcome
                                && dm_channel.read().is_none(),
                        )
                        .on_press({
                            let mut selection = self.selection.clone();
                            let mut dm_channel = dm_channel.clone();

                            move |_| {
                                dm_channel.set(None);
                                *selection.write() = HomeSelection::Welcome;
                            }
                        }),
                    )
                    .child(
                        dmlist_nav_button(
                            group(),
                            "Friends",
                            &theme,
                            &*self.selection.read() == &HomeSelection::Friends
                                && dm_channel.read().is_none(),
                        )
                        .on_press({
                            let mut selection = self.selection.clone();
                            let mut dm_channel = dm_channel.clone();

                            move |_| {
                                dm_channel.set(None);
                                *selection.write() = HomeSelection::Friends;
                            }
                        }),
                    )
                    .child(
                        dmlist_nav_button(
                            sticky_note_2(),
                            "Saved Notes",
                            &theme,
                            saved_messages.read().as_ref().is_some_and(|c| {
                                dm_channel
                                    .read()
                                    .as_ref()
                                    .is_some_and(|(id, _)| id == c.id())
                            }),
                        )
                        .on_press({
                            let mut radio = radio.clone();
                            let saved_messages = saved_messages.clone();
                            let dm_channel = dm_channel.clone();

                            move |_| {
                                let mut dm_channel = dm_channel.clone();

                                spawn(async move {
                                    let id = if let Some(channel) = &*saved_messages.peek() {
                                        channel.id().to_string()
                                    } else {
                                        let dm = http()
                                            .open_dm(&radio.peek_state().user_id.clone().unwrap())
                                            .await
                                            .unwrap();
                                        let id = dm.id().to_string();

                                        radio
                                            .write_channel(AppChannel::Channels)
                                            .channels
                                            .insert(id.clone(), dm);

                                        id
                                    };

                                    dm_channel.set(Some((id.clone(), None)));
                                });
                            }
                        }),
                    ),
            )
            .child(
                rect()
                    .child(
                        label()
                            .margin((28., 8., 8., 8.))
                            .text("Direct Messages")
                            .font_size(13),
                    )
                    .child(
                        VirtualScrollView::new({
                            let selection = self.selection.clone();
                            move |item, _| {
                                let channel = channels.read()[item.index].clone();

                                rect()
                                    .padding((3., 0.))
                                    .key(channel.read().id())
                                    .child(DMButton { channel })
                                    .into_element()
                            }
                        })
                        .item_size(48.)
                        .length(channels.read().len()),
                    ),
            )
    }
}

pub fn dmlist_nav_button(
    icon: Bytes,
    title: &'static str,
    theme: &Theme,
    selected: bool,
) -> StoatButton {
    StoatButton::new().corner_radius(42.).child(
        rect()
            .horizontal()
            .padding((0., 8., 0., 8.))
            .spacing(8.)
            .height(Size::px(42.))
            .cross_align(Alignment::Center)
            .font_size(15)
            .color(theme.md.outline.as_argb_u32())
            .maybe(selected, |btn| {
                btn.background(theme.md.primary_container.as_argb_u32())
                    .color(theme.md.on_primary_container.as_argb_u32())
            })
            .width(Size::Fill)
            .child(MaterialIcon::new(icon).size(Size::px(24.)))
            .child(title),
    )
}
