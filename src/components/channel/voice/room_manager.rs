use std::{
    sync::Arc,
};

use bytes::BytesMut;
use freya::{elements::image::ImageHandle, prelude::*, radio::use_radio};
use freya_engine::prelude::AlphaType;
use futures::StreamExt;
use livekit::{
    PlatformAudio, Room,
    prelude::{Participant, RemoteParticipant},
    track::{RemoteTrack, RemoteVideoTrack, TrackKind, TrackSource},
    webrtc::{prelude::VideoBuffer, video_stream::native::NativeVideoStream},
};
use stoat_models::v0;

use crate::{
    AppChannel, SizeExt,
    components::{Avatar, MaterialIcon, RoomControls, material::filled::mic_off},
    consume_material_theme,
};

pub struct RoomManager {
    pub room: Arc<Room>,
    pub audio: PlatformAudio,
}

impl PartialEq for RoomManager {
    fn eq(&self, other: &Self) -> bool {
        self.room.name() == other.room.name()
    }
}

impl Component for RoomManager {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Channels);

        let channel = radio
            .slice_current({
                let room = self.room.clone();

                move |state| state.channels.get(&room.name()).unwrap()
            })
            .into_readable();

        let server = use_hook(|| {
            if let v0::Channel::TextChannel { server, .. } = &*channel.read() {
                let server = server.clone();

                Some(
                    radio
                        .slice(AppChannel::Servers, move |state| {
                            state.servers.get(&server).unwrap()
                        })
                        .into_readable(),
                )
            } else {
                None
            }
        });

        let mut local_participant = use_state(|| self.room.local_participant());
        let mut remote_participants = use_state(|| self.room.remote_participants());

        use_hook({
            let room = self.room.clone();

            move || {
                let mut sub = room.subscribe();

                spawn({
                    async move {
                        while let Some(_event) = sub.recv().await {
                            local_participant.set(room.local_participant());
                            remote_participants.set(room.remote_participants());
                        }
                    }
                });
            }
        });

        rect()
            .content(Content::Flex)
            .spacing(8.)
            .child(
                rect()
                    .corner_radius(16.)
                    .overflow(Overflow::Clip)
                    .height(Size::flex(1.))
                    .child(
                        ScrollView::new().child(
                            rect()
                                .horizontal()
                                .content(Content::wrap_spacing(8.))
                                .spacing(8.)
                                .child(RoomUserCard {
                                    participant: Participant::Local(
                                        local_participant.read().cloned(),
                                    ),
                                    channel: channel.clone(),
                                    server: server.clone(),
                                })
                                .children(
                                    remote_participants
                                        .read()
                                        .values()
                                        .map(|p| {
                                            let mut cards = Vec::new();

                                            cards.push(
                                                RoomUserCard {
                                                    participant: Participant::Remote(p.clone()),
                                                    channel: channel.clone(),
                                                    server: server.clone(),
                                                }
                                                .into_element(),
                                            );

                                            for track_pub in p.track_publications().values() {
                                                if track_pub.is_subscribed()
                                                    && track_pub.kind() == TrackKind::Video
                                                    && let Some(RemoteTrack::Video(track)) =
                                                        track_pub.track()
                                                {
                                                    cards.push(
                                                        RoomVideoCard {
                                                            participant: p.clone(),
                                                            track,
                                                            channel: channel.clone(),
                                                            server: server.clone(),
                                                        }
                                                        .into_element(),
                                                    );
                                                }
                                            }

                                            cards
                                        })
                                        .flatten(),
                                ),
                        ),
                    ),
            )
            .child(rect().width(Size::Fill).center().child(RoomControls {
                room: self.room.clone(),
                audio: self.audio.clone(),
                local_participant,
            }))
    }
}

struct RoomUserCard {
    pub participant: Participant,
    pub channel: Readable<v0::Channel>,
    pub server: Option<Readable<v0::Server>>,
}

impl PartialEq for RoomUserCard {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Component for RoomUserCard {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Users);
        let users = radio.slice(AppChannel::Users, |state| &state.users);
        let members = radio.slice(AppChannel::Members, |state| &state.members);

        let user = users
            .read()
            .get(self.participant.identity().as_str())
            .cloned();

        use_hook(|| if user.is_none() {});

        let member = use_hook::<Option<Readable<v0::Member>>>(|| {
            if let Some(server) = &self.server {
                let server_id = server.read().id.clone();
                let user_id = self.participant.identity().0.clone();

                if members
                    .read()
                    .get(&server_id)
                    .is_some_and(|members| members.contains_key(&user_id))
                {
                    Some(
                        radio
                            .slice(AppChannel::Members, move |state| {
                                state
                                    .members
                                    .get(&server_id)
                                    .unwrap()
                                    .get(&user_id)
                                    .unwrap()
                            })
                            .into_readable(),
                    )
                } else {
                    None
                }
            } else {
                None
            }
        });

        let user =
            user.unwrap_or_else(|| serde_json::from_str(&self.participant.metadata()).unwrap());

        let is_muted = self.participant.track_publications().values().all(|track| {
            track.kind() == TrackKind::Audio
                && track.is_muted()
                && track.source() != TrackSource::ScreenshareAudio
        });

        let display_name = member
            .as_ref()
            .and_then(|member| member.read().nickname.clone())
            .unwrap_or_else(|| user.display_name.as_ref().unwrap_or(&user.username).clone());

        rect()
            .width(Size::px(384.))
            .height(Size::px(216.))
            .corner_radius(16.)
            .background(0x22000000)
            .border(
                Border::new()
                    .alignment(BorderAlignment::Inner)
                    .width(3.)
                    .fill(if self.participant.is_speaking() {
                        theme.md.primary.as_argb_u32().into()
                    } else {
                        Color::TRANSPARENT
                    }),
            )
            .child(
                rect()
                    .width(Size::Fill)
                    .height(Size::Fill)
                    .center()
                    .child(Avatar::new(user.clone().into(), member.clone(), 48.)),
            )
            .child(
                rect()
                    .color(theme.md.on_surface.as_argb_u32())
                    .layer(10)
                    .position(Position::new_absolute().bottom(0.))
                    .width(Size::Fill)
                    .padding((8., 15.))
                    .horizontal()
                    .main_align(Alignment::SpaceBetween)
                    .child(display_name)
                    .child(rect().maybe_child(
                        is_muted.then(|| MaterialIcon::new(mic_off()).size(Size::px(16.))),
                    )),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.participant.identity().0).into()
    }
}

pub struct RoomVideoCard {
    pub participant: RemoteParticipant,
    pub track: RemoteVideoTrack,
    pub channel: Readable<v0::Channel>,
    pub server: Option<Readable<v0::Server>>,
}

impl PartialEq for RoomVideoCard {
    fn eq(&self, other: &Self) -> bool {
        self.participant.identity() == other.participant.identity()
            && self.track.sid() == other.track.sid()
    }
}

impl Component for RoomVideoCard {
    fn render(&self) -> impl IntoElement {
        let mut buffer = use_state(BytesMut::new);
        let mut handle = use_state(|| None);

        use_hook(|| {
            let mut stream = NativeVideoStream::new(self.track.rtc_track());
            spawn(async move {
                while let Some(frame) = stream.next().await {
                    // let now = SystemTime::now();

                    // if now.duration_since(last_run).unwrap().as_secs_f64() > 1. / 15. {
                    //     last_run = now;
                    // } else {
                    //     continue;
                    // }

                    let buf = frame.buffer.as_i420().unwrap();
                    let (stride_y, stride_u, stride_v) = buf.strides();

                    let (y, u, v) = buf.data();

                    let mut rgba_buf = buffer.write();

                    let y_size = buf.width() * buf.height();

                    rgba_buf.resize((y_size * 4) as usize, 0);
                    // livekit::webrtc::native::yuv_helper::i420_to_abgr(
                    //     y,
                    //     stride_y,
                    //     u,
                    //     stride_u,
                    //     v,
                    //     stride_v,
                    //     &mut rgba_buf,
                    //     buf.width() * 4,
                    //     buf.width() as i32,
                    //     buf.height() as i32,
                    // );

                    yuv::yuv420_to_rgba(
                        &yuv::YuvPlanarImage {
                            y_plane: y,
                            y_stride: stride_y,
                            u_plane: u,
                            u_stride: stride_u,
                            v_plane: v,
                            v_stride: stride_v,
                            width: buf.width(),
                            height: buf.height(),
                        },
                        &mut rgba_buf,
                        buf.width() * 4,
                        yuv::YuvRange::Limited,
                        yuv::YuvStandardMatrix::Bt601,
                    )
                    .unwrap();

                    let bytes = Bytes::copy_from_slice(&rgba_buf);
                    let h =
                        ImageHandle::from_rgba(buf.width(), buf.height(), bytes, AlphaType::Opaque);
                    handle.set(h);
                }
            });
        });

        rect()
            .corner_radius(16.)
            .padding(8.)
            .background(0x22000000)
            .maybe_child(handle.read().cloned().map(|handle| {
                image(handle)
                    .aspect_ratio(AspectRatio::Fit)
                    .corner_radius(8.)
            }))
    }
}
