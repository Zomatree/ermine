use std::collections::HashMap;

use freya::prelude::*;
use livekit::{AudioProcessingType, PlatformAudio, PlayoutDeviceId, RecordingDeviceId};

use crate::{
    components::{Dropdown, checkbox::StoatCheckbox},
    use_changed,
};

#[derive(PartialEq)]
pub struct VoiceSettings {}

impl Component for VoiceSettings {
    fn render(&self) -> impl IntoElement {
        let audio = use_hook(|| PlatformAudio::new().unwrap());

        let recording_devices = audio
            .recording_devices()
            .map(|d| (d.id.clone(), d))
            .collect::<HashMap<_, _>>();
        let audio_input = use_state(|| RecordingDeviceId::from_unchecked_guid("default"));

        let playout_devices = audio
            .playout_devices()
            .map(|d| (d.id.clone(), d))
            .collect::<HashMap<_, _>>();
        let audio_output = use_state(|| PlayoutDeviceId::from_unchecked_guid("default"));

        let noise_suppression = use_state(|| audio.active_ns_type() != AudioProcessingType::None);
        let echo_cancellation = use_state(|| audio.active_aec_type() != AudioProcessingType::None);
        let automatic_gain_control =
            use_state(|| audio.active_agc_type() != AudioProcessingType::None);

        use_changed(noise_suppression, {
            let audio = audio.clone();
            move |&enabled| {
                audio.set_noise_suppression(enabled).unwrap();
            }
        });
        use_changed(echo_cancellation, {
            let audio = audio.clone();
            move |&enabled| {
                audio.set_echo_cancellation(enabled).unwrap();
            }
        });
        use_changed(automatic_gain_control, {
            let audio = audio.clone();
            move |&enabled| {
                audio.set_auto_gain_control(enabled).unwrap();
            }
        });

        rect()
            .spacing(15.)
            .child(
                rect()
                    .spacing(8.)
                    .child(Dropdown::new(
                        "Audio Input",
                        audio_input.into_writable(),
                        recording_devices.keys().cloned().collect(),
                        move |id| {
                            recording_devices
                                .get(id)
                                .unwrap()
                                .name
                                .clone()
                                .into_element()
                        },
                    ))
                    .child(Dropdown::new(
                        "Audio Output",
                        audio_output.into_writable(),
                        playout_devices.keys().cloned().collect(),
                        move |id| playout_devices.get(id).unwrap().name.clone().into_element(),
                    )),
            )
            .child(
                rect()
                    .spacing(8.)
                    .child(StoatCheckbox::new(noise_suppression).child("Noise Suppression"))
                    .child(StoatCheckbox::new(echo_cancellation).child("Echo Cancellation"))
                    .child(
                        StoatCheckbox::new(automatic_gain_control).child("Automatic Gain Control"),
                    ),
            )
    }
}
