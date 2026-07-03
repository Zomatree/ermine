use std::sync::Arc;

use freya::{icons::lucide::mic_off, prelude::*, radio::use_radio};
use livekit::{
    PlatformAudio, Room,
    prelude::Participant,
    track::{TrackKind, TrackSource},
};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Avatar, RoomControls},
    use_material_theme,
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
                                .children(remote_participants.read().values().map(|p| {
                                    RoomUserCard {
                                        participant: Participant::Remote(p.clone()),
                                        channel: channel.clone(),
                                        server: server.clone(),
                                    }
                                    .into_element()
                                })),
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
        let theme = use_material_theme();
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
                        is_muted.then(|| SvgViewer::new(mic_off()).width(Size::px(16.)).height(Size::px(16.))),
                    )),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.participant.identity().0).into()
    }
}
