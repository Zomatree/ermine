use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use freya::{prelude::*, radio::use_radio};
use futures::channel::oneshot;
use livekit::{
    PlatformAudio, Room, RoomEvent, RtcAudioSource,
    options::{TrackPublishOptions, VideoEncoding},
    prelude::LocalParticipant,
    track::{LocalAudioTrack, LocalTrack, LocalVideoTrack, TrackKind, TrackSource},
    webrtc::{
        desktop_capturer::{DesktopCaptureSourceType, DesktopCapturer, DesktopCapturerOptions},
        prelude::{
            I420Buffer, RtcVideoSource, VideoBuffer, VideoFrame, VideoResolution, VideoRotation,
        },
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

#[derive(Debug, Clone)]
enum CaptureEvent {
    Terminate,
}

impl Component for RoomControls {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::CurrentRoom);

        let is_muted = use_memo({
            let local_participant = self.local_participant.clone();
            move || {
                !local_participant
                    .read()
                    .track_publications()
                    .values()
                    .any(|track| {
                        track.kind() == TrackKind::Audio
                            && !track.is_muted()
                            && track.source() != TrackSource::ScreenshareAudio
                    })
            }
        });

        let mut is_deafend = use_state(|| false);
        let is_camera = use_memo(|| false);
        let mut is_screenshare = use_state(|| None::<mpsc::Sender<CaptureEvent>>);

        use_hook(|| {
            {
                let is_deafend = is_deafend();

                for participant in self.room.remote_participants().values() {
                    for track in participant.track_publications().values() {
                        if track.kind() == TrackKind::Audio {
                            track.set_subscribed(track.kind() == TrackKind::Video || !is_deafend);
                        }
                    }
                }
            }

            let mut events = self.room.subscribe();

            spawn(async move {
                while let Some(event) = events.recv().await {
                    match event {
                        RoomEvent::Connected {
                            participants_with_tracks,
                        } => {
                            let is_deafend = is_deafend();

                            for (_, tracks) in participants_with_tracks {
                                for track in tracks {
                                    track.set_subscribed(
                                        track.kind() == TrackKind::Video || !is_deafend,
                                    );
                                }
                            }
                        }
                        RoomEvent::TrackPublished { publication, .. } => {
                            publication.set_subscribed(
                                (publication.kind() == TrackKind::Video) || !is_deafend(),
                            );
                        }
                        _ => {}
                    };
                }
            });
        });

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
            .child(
                StoatButton::new()
                    .corner_radius(20.)
                    .on_press({
                        let room = self.room.clone();

                        move |_| {
                            let is_deafend = is_deafend.toggled();

                            for participant in room.remote_participants().values() {
                                for track in participant.track_publications().values() {
                                    if track.kind() == TrackKind::Audio {
                                        track.set_subscribed(!is_deafend);
                                    }
                                }
                            }
                        }
                    })
                    .child({
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
                    }),
            )
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
                    .on_press({
                        let room = self.room.clone();

                        move |_| {
                            if let Some(tx) = is_screenshare.take() {
                                tx.send(CaptureEvent::Terminate).unwrap();
                                return;
                            };

                            let (tx, rx) = mpsc::channel();
                            is_screenshare.set(Some(tx));

                            #[cfg(any(target_os = "macos", target_os = "linux"))]
                            let source_type = DesktopCaptureSourceType::Generic;
                            #[cfg(not(any(target_os = "macos", target_os = "linux")))]
                            let source_type = DesktopCaptureSourceType::Window;

                            let mut capture_options =
                                DesktopCapturerOptions::new(source_type);

                            capture_options.set_include_cursor(true);
                            #[cfg(target_os = "macos")]
                            {
                                capture_options.set_sck_system_picker(true);
                            };
                            let mut capturer = DesktopCapturer::new(capture_options).unwrap();

                            let (res_tx, res_rx) = oneshot::channel();
                            let mut res_tx = Some(res_tx);
                            let source = Arc::new(Mutex::new(None::<NativeVideoSource>));

                            capturer.start_capture(capturer.get_source_list().first().cloned(), {
                                let source = source.clone();

                                let mut frame_buffer = VideoFrame {
                                    rotation: VideoRotation::VideoRotation0,
                                    timestamp_us: 0,
                                    frame_metadata: None,
                                    buffer: I420Buffer::new(1, 1),
                                };

                                move |r| match r {
                                    Ok(frame) => {
                                        if let Some(res_tx) = res_tx.take() {
                                            res_tx
                                                .send(VideoResolution {
                                                    width: frame.width() as u32,
                                                    height: frame.height() as u32,
                                                })
                                                .unwrap();
                                        };

                                        let width = frame.width();
                                        let height = frame.height();
                                        let stride = frame.stride();
                                        let data = frame.data();

                                        let buffer_width = frame_buffer.buffer.width() as i32;
                                        let buffer_height = frame_buffer.buffer.height() as i32;
                                        if buffer_width != width || buffer_height != height {
                                            frame_buffer.buffer =
                                                I420Buffer::new(width as u32, height as u32);
                                        }

                                        let (y_stride, u_stride, v_stride) =
                                            frame_buffer.buffer.strides();
                                        let (y_plane, u_plane, v_plane) =
                                            frame_buffer.buffer.data_mut();

                                        yuv::bgra_to_yuv420(
                                            &mut yuv::YuvPlanarImageMut {
                                                y_plane: yuv::BufferStoreMut::Borrowed(y_plane),
                                                y_stride,
                                                u_plane: yuv::BufferStoreMut::Borrowed(u_plane),
                                                u_stride,
                                                v_plane: yuv::BufferStoreMut::Borrowed(v_plane),
                                                v_stride,
                                                width: width as u32,
                                                height: height as u32,
                                            },
                                            data,
                                            stride,
                                            yuv::YuvRange::Limited,
                                            yuv::YuvStandardMatrix::Bt601,
                                            yuv::YuvConversionMode::Fast,
                                        )
                                        .unwrap();

                                        if let Some(source) = &*source.lock().unwrap() {
                                            source.capture_frame(&frame_buffer);
                                        }
                                    }
                                    Err(_) => {},
                                }
                            });

                            let track_id = Arc::new(Mutex::new(None));

                            spawn_forever({
                                let track_id = track_id.clone();
                                let room = room.clone();

                                async move {
                                    thread(move || {
                                        loop {
                                            match rx.recv_timeout(Duration::from_millis(32)) {
                                                Ok(CaptureEvent::Terminate) => {
                                                    log::info!(
                                                        "Capture thread received terminate message"
                                                    );
                                                    break;
                                                }
                                                Err(mpsc::RecvTimeoutError::Timeout) => {
                                                    capturer.capture_frame();
                                                }
                                                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                                            }
                                        }
                                    })
                                    .await;

                                    if let Some(track_id) = track_id.lock().unwrap().take() {
                                        let _ = room.local_participant()
                                            .unpublish_track(&track_id)
                                            .await;
                                    };
                                }
                            });

                            spawn_forever({
                                let room = room.clone();

                                async move {
                                    if let Ok(res) = res_rx.await {
                                        let width1 = res.width as f32;
                                        let height1 = res.height as f32;
                                        let scale = (1080. / width1 as f32).min(720. / height1 as f32).min(1.);

                                        let width = (width1 * scale) as u32;
                                        let height = (height1 * scale) as u32;

                                        let native_source = NativeVideoSource::new(VideoResolution { width, height }, true);

                                        let track = LocalVideoTrack::create_video_track(
                                            "screenshare",
                                            RtcVideoSource::Native(native_source.clone()),
                                        );
                                        let mut options = TrackPublishOptions::default();
                                        options.video_encoding = Some(VideoEncoding {
                                            max_bitrate: 1_700_000,
                                            max_framerate: 30.,
                                        });
                                        options.source = TrackSource::Screenshare;
                                        let publication = room
                                            .local_participant()
                                            .publish_track(LocalTrack::Video(track), options)
                                            .await
                                            .unwrap();

                                        *track_id.lock().unwrap() = Some(publication.sid());
                                        *source.lock().unwrap() = Some(native_source.clone());
                                    };
                                }
                            });
                        }
                    })
                    .child({
                        let is_screenshare = is_screenshare.read().is_some();

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
