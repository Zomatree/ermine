use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use crate::{Config, http};
use freya::prelude::*;
use futures::channel::oneshot;
use livekit::{
    PlatformAudio, Room as LivekitRoom, RoomEvent, RoomOptions, RtcAudioSource,
    id::ParticipantIdentity,
    options::{TrackPublishOptions, VideoEncoding},
    participant::{LocalParticipant, RemoteParticipant},
    track::{LocalAudioTrack, LocalTrack, LocalVideoTrack, TrackKind, TrackSource},
    webrtc::{
        desktop_capturer::{
            CaptureError, DesktopCaptureSourceType, DesktopCapturer, DesktopCapturerOptions,
        },
        native::yuv_helper::nv12_to_i420,
        prelude::{
            I420Buffer, RtcVideoSource, VideoBuffer, VideoFrame, VideoResolution, VideoRotation,
        },
        video_source::native::NativeVideoSource,
    },
};
use stoat_models::v0;
use yuv::{
    BufferStoreMut, YuvConversionMode, YuvPackedImage, YuvPlanarImageMut, YuvRange,
    YuvStandardMatrix, bgra_to_yuv420, rgb_to_yuv420, rgba_to_yuv420, yuyv422_to_yuv420,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CaptureEvent {
    Terminate,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VoiceConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Clone, Copy)]
pub struct Room {
    config: State<Config>,
    status: State<VoiceConnectionStatus>,
    room: State<Option<Arc<LivekitRoom>>>,
    audio: State<Option<PlatformAudio>>,
    screenshare_tx: State<Option<mpsc::Sender<CaptureEvent>>>,
}

impl Room {
    pub fn new() -> Self {
        Self {
            config: consume_context(),
            status: State::create(VoiceConnectionStatus::Disconnected),
            room: State::create(None),
            audio: State::create(None),
            screenshare_tx: State::create(None),
        }
    }

    pub async fn connect(&self, channel_id: &str) {
        let mut status = self.status;

        status.set(VoiceConnectionStatus::Connecting);

        let http = http();

        if let Ok(resp) = http
            .join_call(
                &channel_id,
                &v0::DataJoinCall {
                    node: Some(
                        http.api_config
                            .features
                            .livekit
                            .nodes
                            .first()
                            .unwrap()
                            .name
                            .clone(),
                    ),
                    force_disconnect: Some(true),
                    recipients: None,
                },
            )
            .await
        {
            self.inner_connect(&resp.url, &resp.token).await
        } else {
            status.set(VoiceConnectionStatus::Disconnected);
        }
    }

    pub async fn inner_connect(&self, url: &str, token: &str) {
        let audio = PlatformAudio::new().unwrap();

        audio
            .set_playout_device(&audio.playout_devices().next().unwrap().id)
            .unwrap();
        audio
            .set_recording_device(&audio.recording_devices().next().unwrap().id)
            .unwrap();
        let mut options = RoomOptions::default();
        options.auto_subscribe = false;

        let (room, mut events) = LivekitRoom::connect(url, token, options).await.unwrap();
        let room = Arc::new(room);

        let config = self.config;
        spawn({
            let mut room_state = self.room;
            let mut connection_status = self.status;
            let mut audio_state = self.audio;

            async move {
                while let Some(event) = events.recv().await {
                    room_state.try_write();
                    log::debug!("Room Event: {event:?}");

                    match event {
                        RoomEvent::Connected {
                            participants_with_tracks,
                        } => {
                            let is_deafend = config.read().audio.deafened;
                            connection_status.set(VoiceConnectionStatus::Connected);
                            room_state.set(Some(room.clone()));
                            audio_state.set(Some(audio.clone()));

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
                                (publication.kind() == TrackKind::Video)
                                    || !config.read().audio.deafened,
                            );
                        }
                        RoomEvent::Disconnected { reason } => {
                            log::debug!("Disconnecting from room ({reason:?})");
                            connection_status.set(VoiceConnectionStatus::Disconnected);
                            room_state.set(None);
                            audio_state.set(None);

                            let _ = audio.stop_recording();
                        }
                        _ => {}
                    };
                }
            }
        });
    }

    pub fn read(&self) -> Option<Arc<LivekitRoom>> {
        self.room.read().cloned()
    }

    pub fn local_participant(&self) -> Option<LocalParticipant> {
        self.room.read().as_ref().map(|r| r.local_participant())
    }

    pub fn remote_participants(&self) -> HashMap<ParticipantIdentity, RemoteParticipant> {
        self.room
            .read()
            .as_ref()
            .map_or_default(|r| r.remote_participants())
    }

    pub fn is_deafened(&self) -> bool {
        consume_context::<State<Config>>().read().audio.deafened
    }

    pub fn toggle_deafen(&self) {
        let is_deafend = {
            let mut config = self.config;
            let mut cfg = config.write();
            cfg.audio.deafened = !cfg.audio.deafened;

            cfg.audio.deafened
        };

        for participant in self.remote_participants().values() {
            for track in participant.track_publications().values() {
                if track.kind() == TrackKind::Audio {
                    track.set_subscribed(!is_deafend);
                }
            }
        }
    }

    pub fn is_muted(&self) -> bool {
        !self.local_participant().is_some_and(|p| {
            p.track_publications().values().any(|track| {
                track.kind() == TrackKind::Audio
                    && !track.is_muted()
                    && track.source() != TrackSource::ScreenshareAudio
            })
        })
    }

    pub async fn toggle_mute(&self) {
        if let Some(local_participant) = self.local_participant() {
            if self.is_muted() {
                let track =
                    LocalAudioTrack::create_audio_track("microphone", RtcAudioSource::Device);

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

                self.audio
                    .read()
                    .as_ref()
                    .map(|audio| audio.start_recording().unwrap());
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

                    self.audio
                        .read()
                        .as_ref()
                        .map(|audio| audio.stop_recording().unwrap());
                };
            };
        };
    }

    pub fn is_screensharing(&self) -> bool {
        self.screenshare_tx.read().is_some()
    }

    pub fn start_screensharing(&self) {
        let Some(room) = self.room.read().cloned() else {
            return;
        };

        let (tx, rx) = mpsc::channel();
        self.screenshare_tx.clone().set(Some(tx.clone()));

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        let source_type = DesktopCaptureSourceType::Generic;
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        let source_type = DesktopCaptureSourceType::Window;

        let mut capture_options = DesktopCapturerOptions::new(source_type);

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

            move |r| {
                match r {
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
                            frame_buffer.buffer = I420Buffer::new(width as u32, height as u32);
                        }

                        let (y_stride, u_stride, v_stride) = frame_buffer.buffer.strides();
                        let (y_plane, u_plane, v_plane) = frame_buffer.buffer.data_mut();

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
                    }
                    Err(CaptureError::Temporary) => {}
                    Err(CaptureError::Permanent) => {
                        _ = tx.send(CaptureEvent::Terminate);
                    }
                };

                if let Some(source) = &*source.lock().unwrap() {
                    source.capture_frame(&frame_buffer);
                }
            }
        });

        let track_id = Arc::new(Mutex::new(None));

        spawn_forever({
            let track_id = track_id.clone();
            let room = room.clone();
            let mut tx = self.screenshare_tx;

            async move {
                thread(move || {
                    loop {
                        match rx.recv_timeout(Duration::from_millis(32)) {
                            Ok(CaptureEvent::Terminate) => {
                                log::info!("Capture thread received terminate message");
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

                tx.set(None);

                if let Some(track_id) = track_id.lock().unwrap().take() {
                    let _ = room
                        .local_participant()
                        .unpublish_track(&track_id)
                        .await
                        .unwrap();
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

                    let native_source =
                        NativeVideoSource::new(VideoResolution { width, height }, true);

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

    pub fn stop_screensharing(&self) {
        if let Some(tx) = self.screenshare_tx.clone().take() {
            tx.send(CaptureEvent::Terminate).unwrap();
        };
    }

    pub fn toggle_screensharing(&self) {
        let is_screensharing = self.is_screensharing();
        self.stop_screensharing();

        if !is_screensharing {
            self.start_screensharing();
        }
    }

    pub fn start_camera(&self) {
        let Some(room) = self.room.read().cloned() else {
            return;
        };

        let devices = cameras::devices().unwrap();
        let Some(device) = devices.first() else {
            return;
        };

        let camera = cameras::open(
            device,
            cameras::StreamConfig {
                resolution: cameras::Resolution {
                    width: 1080,
                    height: 720,
                },
                framerate: 30,
                pixel_format: cameras::PixelFormat::Yuyv,
            },
        )
        .unwrap();

        let frame = cameras::next_frame(&camera, Duration::from_secs(2)).unwrap();

        spawn_forever({
            let room = room.clone();

            async move {
                let width1 = frame.width as f32;
                let height1 = frame.height as f32;
                let scale = (1080. / width1 as f32).min(720. / height1 as f32).min(1.);

                let width = (width1 * scale) as u32;
                let height = (height1 * scale) as u32;
                let native_source =
                    NativeVideoSource::new(VideoResolution { width, height }, false);

                let track = LocalVideoTrack::create_video_track(
                    "camera",
                    RtcVideoSource::Native(native_source.clone()),
                );

                let mut options = TrackPublishOptions::default();
                options.video_encoding = Some(VideoEncoding {
                    max_bitrate: 1_700_000,
                    max_framerate: 30.,
                });
                options.source = TrackSource::Camera;

                let publication = room
                    .local_participant()
                    .publish_track(LocalTrack::Video(track), options)
                    .await
                    .unwrap();

                let mut buffer = VideoFrame::new(
                    VideoRotation::VideoRotation0,
                    I420Buffer::new(frame.width, frame.height),
                );

                thread(move || {
                    loop {
                        match cameras::next_frame(&camera, Duration::from_millis(32)) {
                            Ok(frame) => {
                                let (y_stride, u_stride, v_stride) = buffer.buffer.strides();
                                let (y, u, v) = buffer.buffer.data_mut();
                                let planar_image = &mut YuvPlanarImageMut {
                                    y_plane: BufferStoreMut::Borrowed(y),
                                    y_stride,
                                    u_plane: BufferStoreMut::Borrowed(u),
                                    u_stride,
                                    v_plane: BufferStoreMut::Borrowed(v),
                                    v_stride,
                                    width: frame.width,
                                    height: frame.height,
                                };

                                match frame.pixel_format {
                                    cameras::PixelFormat::Bgra8 => bgra_to_yuv420(
                                        planar_image,
                                        &frame.plane_primary,
                                        frame.stride,
                                        YuvRange::Limited,
                                        YuvStandardMatrix::Bt601,
                                        YuvConversionMode::Fast,
                                    )
                                    .unwrap(),
                                    cameras::PixelFormat::Rgb8 => rgb_to_yuv420(
                                        planar_image,
                                        &frame.plane_primary,
                                        frame.stride,
                                        YuvRange::Limited,
                                        YuvStandardMatrix::Bt601,
                                        YuvConversionMode::Fast,
                                    )
                                    .unwrap(),
                                    cameras::PixelFormat::Rgba8 => rgba_to_yuv420(
                                        planar_image,
                                        &frame.plane_primary,
                                        frame.stride,
                                        YuvRange::Limited,
                                        YuvStandardMatrix::Bt601,
                                        YuvConversionMode::Fast,
                                    )
                                    .unwrap(),
                                    cameras::PixelFormat::Yuyv => yuyv422_to_yuv420(
                                        planar_image,
                                        &YuvPackedImage {
                                            yuy: &frame.plane_primary,
                                            yuy_stride: frame.stride,
                                            width: frame.width,
                                            height: frame.height,
                                        },
                                    )
                                    .unwrap(),
                                    cameras::PixelFormat::Nv12 => nv12_to_i420(
                                        &frame.plane_primary,
                                        frame.stride,
                                        &frame.plane_secondary,
                                        frame.stride,
                                        y,
                                        y_stride,
                                        u,
                                        u_stride,
                                        v,
                                        v_stride,
                                        frame.width as i32,
                                        frame.height as i32,
                                    ),
                                    cameras::PixelFormat::Mjpeg => todo!(),
                                    _ => todo!(),
                                };

                                native_source.capture_frame(&buffer);
                            }
                            Err(e) => {
                                log::error!("{e:?}");
                            }
                        }
                    }
                })
                .await;

                let _ = room
                    .local_participant()
                    .unpublish_track(&publication.sid())
                    .await;
            }
        });
    }

    pub fn status(&self) -> VoiceConnectionStatus {
        (self.status)()
    }

    pub async fn close(&self) {
        if let Some(room) = self.read() {
            self.stop_screensharing();
            _ = room.close().await;
        }
    }
}
