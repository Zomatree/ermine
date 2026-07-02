use std::{borrow::Cow, fmt::Debug, mem, sync::Arc};

use freya::{
    icons::lucide::{at_sign, hash, notebook_text, phone_call, pin, users_round},
    prelude::*,
    radio::use_radio,
};
use indexmap::IndexMap;
// use livekit::{PlatformAudio, Room, RoomOptions};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{
        AttachmentController, ChannelMessages, HideSidebarHeader, MemberList, MessageAttachmentsPreview, MessageInput, MessageReplyPreview, ModalValue, ReplyController, StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip, Textbox, use_modals
    },
    use_config, use_material_theme,
};

#[derive(PartialEq)]
pub struct Channel {
    pub channel: Readable<v0::Channel>,
    pub server: Option<Readable<v0::Server>>,
}

impl Component for Channel {
    fn render(&self) -> impl IntoElement {
        let mut config = use_config();
        let radio = use_radio(AppChannel::UserId);
        // let current_room =
        //     radio.slice_mut(AppChannel::CurrentRoom, |state| &mut state.current_room);
        let theme = use_material_theme();
        let mut modals = use_modals();

        let replies = ReplyController(use_state(Vec::new));
        let attachments = AttachmentController(use_state(IndexMap::new));

        let hide_members_list = config.read().hide_members_list;

        let search = use_state(String::new);

        let channel = self.channel.read().clone();

        let channel_name = match &channel {
            v0::Channel::DirectMessage { recipients, .. } => {
                let user_id = radio.peek_state().user_id.clone().unwrap();

                let other = recipients
                    .iter()
                    .find(|&id| id != &*user_id)
                    .unwrap()
                    .clone();

                let user = radio.slice(AppChannel::Users, move |state| {
                    state.users.get(&other).unwrap()
                });

                Cow::Owned(user.read().username.clone())
            }
            v0::Channel::Group { name, .. } | v0::Channel::TextChannel { name, .. } => {
                Cow::Owned(name.clone())
            }
            v0::Channel::SavedMessages { .. } => Cow::Borrowed("Saved Messages"),
        };

        let channel_description = if let v0::Channel::TextChannel { description, .. }
        | v0::Channel::Group { description, .. } = &channel
        {
            description.clone().filter(|d| !d.is_empty())
        } else {
            None
        };

        rect()
            .child(
                rect()
                    .horizontal()
                    .height(Size::px(48.))
                    .padding((0., 24., 0., 8.))
                    .margin((8., 0.))
                    .spacing(10.)
                    .cross_align(Alignment::Center)
                    .content(Content::Flex)
                    .child(HideSidebarHeader {
                        icon: match &channel {
                            v0::Channel::DirectMessage { .. } => at_sign(),
                            v0::Channel::SavedMessages { .. } => notebook_text(),
                            _ => hash(),
                        },
                    })
                    .child(label().text(channel_name).font_size(16).max_lines(1))
                    .maybe_child(channel_description.is_some().then(|| {
                        rect()
                            .height(Size::px(20.))
                            .width(Size::px(1.))
                            .margin((0., 5.))
                            .background(theme.md.outline_variant.as_argb_u32())
                    }))
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .maybe_child(channel_description.map(|description| {
                                label()
                                    .max_lines(1)
                                    .text_overflow(TextOverflow::Ellipsis)
                                    .font_size(14.)
                                    .text(description)
                                    .on_press({
                                        let id = channel.id().to_string();

                                        move |_| {
                                            modals.write().push_modal(
                                                ModalValue::ChannelDescription {
                                                    channel: id.clone(),
                                                },
                                            )
                                        }
                                    })
                            })),
                    )
                    // .maybe_child(
                    //     matches!(
                    //         &channel,
                    //         v0::Channel::Group { .. }
                    //             | v0::Channel::DirectMessage { .. }
                    //             | v0::Channel::TextChannel { voice: Some(_), .. }
                    //     )
                    //     .then(|| {
                    //         StoatTooltip::new(
                    //             label()
                    //                 .font_size(11.)
                    //                 .max_lines(1)
                    //                 .text("Join voice channel"),
                    //         )
                    //         .position(AttachedPosition::Bottom)
                    //         .child(
                    //             StoatButton::new()
                    //                 .corner_radius(40.)
                    //                 .on_press({
                    //                     let current_room = current_room.clone();
                    //                     move |_| {
                    //                         spawn({
                    //                             let id = channel.id().to_string();
                    //                             let current_room = current_room.clone();
                    //                             async move {
                    //                                 let http = http();
                    //                                 if let Ok(resp) = http
                    //                                     .join_call(
                    //                                         &id,
                    //                                         &v0::DataJoinCall {
                    //                                             node: Some(
                    //                                                 http.api_config
                    //                                                     .features
                    //                                                     .livekit
                    //                                                     .nodes
                    //                                                     .first()
                    //                                                     .unwrap()
                    //                                                     .name
                    //                                                     .clone(),
                    //                                             ),
                    //                                             force_disconnect: Some(true),
                    //                                             recipients: None,
                    //                                         },
                    //                                     )
                    //                                     .await
                    //                                 {
                    //                                     let audio = PlatformAudio::new().unwrap();
                    //                                     println!(
                    //                                         "{:?}",
                    //                                         audio
                    //                                             .recording_devices()
                    //                                             .collect::<Vec<_>>()
                    //                                     );
                    //                                     println!(
                    //                                         "{:?}",
                    //                                         audio
                    //                                             .playout_devices()
                    //                                             .collect::<Vec<_>>()
                    //                                     );
                    //                                     audio
                    //                                         .set_playout_device(
                    //                                             &audio
                    //                                                 .playout_devices()
                    //                                                 .next()
                    //                                                 .unwrap()
                    //                                                 .id,
                    //                                         )
                    //                                         .unwrap();
                    //                                     audio
                    //                                         .set_recording_device(
                    //                                             &audio
                    //                                                 .recording_devices()
                    //                                                 .next()
                    //                                                 .unwrap()
                    //                                                 .id,
                    //                                         )
                    //                                         .unwrap();
                    //                                     let (room, _) = Room::connect(
                    //                                         &resp.url,
                    //                                         &resp.token,
                    //                                         RoomOptions::default(),
                    //                                     )
                    //                                     .await
                    //                                     .unwrap();
                    //                                     *current_room.clone().write() =
                    //                                         Some((Arc::new(room), audio));
                    //                                 };
                    //                             }
                    //                         });
                    //                     }
                    //                 })
                    //                 .child(
                    //                     rect()
                    //                         .horizontal()
                    //                         .height(Size::px(40.))
                    //                         .padding((0., 8.))
                    //                         .center()
                    //                         .color(theme.md.on_surface_variant.as_argb_u32())
                    //                         .child(
                    //                             svg(phone_call())
                    //                                 .width(Size::px(24.))
                    //                                 .height(Size::px(24.)),
                    //                         ),
                    //                 ),
                    //         )
                    //     }),
                    // )
                    .child(
                        StoatTooltip::new(
                            label()
                                .font_size(11.)
                                .max_lines(1)
                                .text("View pinned messages"),
                        )
                        .position(AttachedPosition::Bottom)
                        .child(
                            StoatButton::new()
                                .corner_radius(40.)
                                .on_press(move |_| {})
                                .child(
                                    rect()
                                        .horizontal()
                                        .height(Size::px(40.))
                                        .padding((0., 8.))
                                        .center()
                                        .color(theme.md.on_surface_variant.as_argb_u32())
                                        .child(
                                            svg(pin()).width(Size::px(24.)).height(Size::px(24.)),
                                        ),
                                ),
                        ),
                    )
                    .child(
                        StoatTooltip::new(label().font_size(11.).max_lines(1).text("View members"))
                            .position(AttachedPosition::Bottom)
                            .child(
                                StoatButton::new()
                                    .corner_radius(40.)
                                    .on_press(move |_| {
                                        config.write().hide_members_list = !hide_members_list
                                    })
                                    .child(
                                        rect()
                                            .horizontal()
                                            .height(Size::px(40.))
                                            .padding((0., 8.))
                                            .center()
                                            .color(theme.md.on_surface_variant.as_argb_u32())
                                            .child(
                                                svg(users_round())
                                                    .width(Size::px(24.))
                                                    .height(Size::px(24.)),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        rect()
                            .child(
                                Input::new(search)
                                    .placeholder("Search messages...")
                                    .placeholder_color(theme.md.outline.as_argb_u32())
                                    .border_fill(Color::TRANSPARENT)
                                    .inner_margin((10., 16.))
                                    .corner_radius(40.)
                                    .width(Size::Fill)
                                    .background(theme.md.surface_container_high.as_argb_u32()),
                            )
                            .max_width(Size::px(240.)),
                    ),
            )
            .child(
                rect()
                    .height(Size::Fill)
                    .horizontal()
                    .content(Content::Flex)
                    .child(
                        rect()
                            .margin((0., 8., 8., 8.))
                            .content(Content::Flex)
                            .height(Size::Fill)
                            .width(Size::flex(1.))
                            .spacing(8.)
                            // .maybe_child(current_room.read().cloned().map(|(room, audio)| {
                            //     rect()
                            //         .height(Size::flex(4.))
                            //         .width(Size::flex(1.))
                            //         .content(Content::Flex)
                            //         .corner_radius(28.)
                            //         .background(theme.md.secondary_container.as_argb_u32())
                            //         .padding(8.)
                            //         .child(RoomManager { room: room, audio })
                            // }))
                            .child(
                                rect()
                                    .height(Size::flex(6.))
                                    .content(Content::Flex)
                                    .corner_radius(28.)
                                    .background(theme.md.surface_container_lowest.as_argb_u32())
                                    .overflow(Overflow::Clip)
                                    .child(
                                        rect()
                                            .height(Size::flex(1.))
                                            .main_align(Alignment::End)
                                            .child(ChannelMessages {
                                                replies,
                                                channel: self.channel.clone(),
                                                server: self.server.clone(),
                                            }),
                                    )
                                    .child(
                                        MessageInput {
                                            channel: self.channel.clone(),
                                            replies,
                                            attachments,
                                        }
                                    ),
                            ),
                    )
                    .maybe_child(self.server.as_ref().filter(|_| !hide_members_list).map(
                        |server| {
                            rect()
                                .child(MemberList {
                                    server: server.clone(),
                                })
                                .width(Size::px(248.))
                                .padding((0., 8., 0., 0.))
                        },
                    )),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.channel.peek().id().to_string()).into()
    }
}
