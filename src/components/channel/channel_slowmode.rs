use std::time::Duration;

use freya::{prelude::*, radio::use_radio};
use jiff::{Timestamp, Unit};
use stoat_models::v0;
use stoat_permissions::PermissionValue;
use tokio::time::sleep;

use crate::{
    AppChannel, SizeExt, calculate_channel_permissions,
    components::{MaterialIcon, material::outlined::schedule},
    consume_material_theme, user_permissions_query,
};

#[derive(PartialEq)]
pub struct ChannelSlowmode {
    pub channel: Readable<v0::Channel>,
}

impl Component for ChannelSlowmode {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Slowmodes);
        let slowmodes = radio.slice_current(|state| &state.slowmodes);
        let permissions = use_state(|| PermissionValue::from_raw(0));

        use_side_effect({
            let radio = radio.clone();
            let channel = self.channel.clone();

            move || {
                let radio = radio.clone();
                let channel = channel.clone();

                spawn(async move {
                    let mut query =
                        user_permissions_query(radio.clone()).channel(channel.read().clone());

                    let value = calculate_channel_permissions(&mut query).await;
                    permissions.clone().set(value);
                });
            }
        });

        let channel_slowmode = use_side_effect_value({
            let channel = self.channel.clone();
            move || {
                if let v0::Channel::TextChannel {
                    slowmode: Some(slowmode),
                    ..
                } = &*channel.read()
                {
                    Some(*slowmode)
                } else {
                    None
                }
            }
        });

        let mut now = use_state(Timestamp::now);

        use_side_effect_with_deps(
            &slowmodes.read().get(self.channel.read().id()).cloned(),
            move |slowmode| {
                if let Some(slowmode) = *slowmode {
                    spawn(async move {
                        for _ in 0..=slowmode.duration {
                            now.set(Timestamp::now());
                            sleep(Duration::from_millis(1000)).await;
                        }
                    });
                };
            },
        );

        if let Some(channel_slowmode) = *channel_slowmode.read() {
            let current_slowmode = slowmodes.read().get(self.channel.read().id()).cloned();
            let now = now();

            rect()
                .color(theme.md.outline.as_argb_u32())
                .font_size(12.)
                .cross_align(Alignment::Center)
                .horizontal()
                .spacing(4.)
                .child(MaterialIcon::new(schedule()).size(Size::px(16.)))
                .child(
                    if permissions().has_channel_permission(
                        stoat_permissions::ChannelPermission::BypassSlowmode,
                    ) {
                        "Slowmode Immune".into_element()
                    } else if let Some(current_slowmode) = current_slowmode
                        && let time_left =
                            (now.until(current_slowmode.finished_at)
                                .unwrap()
                                .total(Unit::Second)
                                .unwrap()
                                .ceil() as u64)
                        && time_left > 0
                    {
                        format!("{time_left}s").into_element()
                    } else {
                        format!("Slowmode is enabled, {channel_slowmode}s").into_element()
                    },
                )
        } else {
            rect()
        }
    }
}
