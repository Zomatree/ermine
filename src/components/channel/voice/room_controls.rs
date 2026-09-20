use std::sync::Arc;

use freya::{prelude::*, radio::use_radio};
use livekit::{
    PlatformAudio, Room, RtcAudioSource,
    options::TrackPublishOptions,
    prelude::LocalParticipant,
    track::{LocalAudioTrack, LocalTrack, LocalVideoTrack, TrackKind, TrackSource},
    webrtc::{
        desktop_capturer::{DesktopCaptureSourceType, DesktopCapturer, DesktopCapturerOptions},
        prelude::RtcVideoSource,
        video_source::native::NativeVideoSource,
    },
};

use crate::{
    AppChannel, SizeExt,
    components::{
        MaterialIcon, StoatButton, StoatButtonLayoutThemePartialExt,
        material::{
            filled::{
                call_end, camera_alt as filled_camera_alt, headphones, headset_off, mic, mic_off,
                screen_share as filled_screen_share,
            },
            outlined::{camera_alt, screen_share},
        },
    },
    consume_material_theme,
};

pub struct RoomControls {
    pub room: Arc<Room>,
    pub audio: PlatformAudio,
    pub local_participant: State<LocalParticipant>,
}

impl PartialEq for RoomControls {
    fn eq(&self, other: &Self) -> bool {
        self.room.name() == other.room.name()
    }
}

impl Component for RoomControls {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::CurrentRoom);

        let is_muted = use_memo({
            let local_participant = self.local_participant.clone();
            move || {
                local_participant
                    .read()
                    .track_publications()
                    .values()
                    .all(|track| {
                        track.kind() == TrackKind::Audio
                            && track.is_muted()
                            && track.source() != TrackSource::ScreenshareAudio
                    })
            }
        });

        let is_deafend = use_memo(|| false);
        let is_camera = use_memo(|| false);
        let is_screenshare = use_memo(|| false);

        let room_state = radio.slice_mut_current(|state| &mut state.current_room);

        rect()
            .horizontal()
            .padding(8.)
            .corner_radius(48.)
            .spacing(8.)
            .background(theme.md.surface_container.as_argb_u32())
            .child(
                StoatButton::new()
                    .corner_radius(20.)
                    .on_press({
                        let local_participant = self.local_participant.clone();
                        let audio = self.audio.clone();

                        move |_| {
                            let local_participant = local_participant.read().cloned();
                            let audio = audio.clone();

                            spawn(async move {
                                if is_muted() {
                                    let track = LocalAudioTrack::create_audio_track(
                                        "microphone",
                                        RtcAudioSource::Device,
                                    );
                                    local_participant
                                        .publish_track(
                                            LocalTrack::Audio(track),
                                            TrackPublishOptions {
                                                source: TrackSource::Microphone,
                                                ..Default::default()
                                            },
                                        )
                                        .await
                                        .unwrap();
                                } else {
                                    if let Some(track) = local_participant
                                        .track_publications()
                                        .values()
                                        .find(|track| {
                                            track.kind() == TrackKind::Audio
                                                && !track.is_muted()
                                                && track.source() != TrackSource::ScreenshareAudio
                                        })
                                    {
                                        local_participant
                                            .unpublish_track(&track.sid())
                                            .await
                                            .unwrap();

                                        audio.stop_recording().unwrap();
                                    };
                                };
                            });
                        }
                    })
                    .child({
                        let is_muted = is_muted();

                        rect()
                            .background(if is_muted {
                                theme.md.secondary_container.as_argb_u32()
                            } else {
                                theme.md.primary.as_argb_u32()
                            })
                            .color(if is_muted {
                                theme.md.on_secondary_container.as_argb_u32()
                            } else {
                                theme.md.on_primary.as_argb_u32()
                            })
                            .width(Size::px(40.))
                            .height(Size::px(40.))
                            .center()
                            .child(
                                MaterialIcon::new(if is_muted { mic_off() } else { mic() })
                                    .size(Size::px(24.)),
                            )
                    }),
            )
            .child(StoatButton::new().corner_radius(20.).child({
                let is_deafend = is_deafend();

                rect()
                    .background(if is_deafend {
                        theme.md.secondary_container.as_argb_u32()
                    } else {
                        theme.md.primary.as_argb_u32()
                    })
                    .color(if is_deafend {
                        theme.md.on_secondary_container.as_argb_u32()
                    } else {
                        theme.md.on_primary.as_argb_u32()
                    })
                    .width(Size::px(40.))
                    .height(Size::px(40.))
                    .center()
                    .child(
                        MaterialIcon::new(if is_deafend {
                            headset_off()
                        } else {
                            headphones()
                        })
                        .size(Size::px(24.)),
                    )
            }))
            .child(StoatButton::new().corner_radius(20.).child({
                let is_camera = is_camera();

                rect()
                    .background(if !is_camera {
                        theme.md.secondary_container.as_argb_u32()
                    } else {
                        theme.md.primary.as_argb_u32()
                    })
                    .color(if !is_camera {
                        theme.md.on_secondary_container.as_argb_u32()
                    } else {
                        theme.md.on_primary.as_argb_u32()
                    })
                    .width(Size::px(40.))
                    .height(Size::px(40.))
                    .center()
                    .child(
                        MaterialIcon::new(if !is_camera {
                            camera_alt()
                        } else {
                            filled_camera_alt()
                        })
                        .size(Size::px(24.)),
                    )
            }))
            .child(
                StoatButton::new()
                    .corner_radius(20.)
                    // .on_press({
                    //     let room = self.room.clone();
                    //     let audio = self.audio.clone();
                    //     move |_| {
                    //         let mut options =
                    //             DesktopCapturerOptions::new(DesktopCaptureSourceType::Window);
                    //         #[cfg(target_os = "macos")]
                    //         {
                    //             options.set_sck_system_picker(true);
                    //         };

                    //         let mut capturer = DesktopCapturer::new(options).unwrap();
                    //         capturer.start_capture(capturer.get_source_list().first().cloned(), move |r| { println!("{:?}", r.unwrap().data().len()) });
                    //         for source in capturer.get_source_list() {
                    //             println!("{:?} {:?}", source.id(), source.title())
                    //         }
                    //         capturer.capture_frame();
                    //         // NativeVideoSource::new(resolution, is_screencast)
                    //         // LocalVideoTrack::create_video_track(name, source)
                    //         // room.local_participant().publish_track(LocalTrack::Video(()), options)
                    //     }
                    // })
                    .child({
                        let is_screenshare = is_screenshare();

                        rect()
                            .background(if !is_screenshare {
                                theme.md.secondary_container.as_argb_u32()
                            } else {
                                theme.md.primary.as_argb_u32()
                            })
                            .color(if !is_screenshare {
                                theme.md.on_secondary_container.as_argb_u32()
                            } else {
                                theme.md.on_primary.as_argb_u32()
                            })
                            .width(Size::px(40.))
                            .height(Size::px(40.))
                            .center()
                            .child(
                                MaterialIcon::new(if !is_screenshare {
                                    screen_share()
                                } else {
                                    filled_screen_share()
                                })
                                .size(Size::px(24.)),
                            )
                    }),
            )
            .child(
                StoatButton::new()
                    .corner_radius(20.)
                    .on_press({
                        let room = self.room.clone();

                        move |_| {
                            let room = room.clone();
                            let mut room_state = room_state.clone();

                            spawn_forever(async move {
                                room_state.set(None);
                                room.close().await.unwrap()
                            });
                        }
                    })
                    .child(
                        rect()
                            .background(theme.md.error.as_argb_u32())
                            .color(theme.md.on_error.as_argb_u32())
                            .width(Size::px(56.))
                            .height(Size::px(40.))
                            .center()
                            .child(
                                MaterialIcon::new(call_end())
                                    .size(Size::px(24.))
                                    .margin((2., 0., 0., 0.)),
                            ),
                    ),
            )
    }
}
