use std::time::{Duration, SystemTime};

use freya::{
    animation::{AnimColor, AnimatedValue, Ease, OnChange, OnCreation, use_animation},
    prelude::*,
};
use jiff::{Timestamp, tz::TimeZone};
use stoat_models::v0;

use crate::{
    components::{MessageContent, MessageHover, MessageModel, StoatTooltip},
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct TrailingMessage {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
}

impl Component for TrailingMessage {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let hovering = try_consume_context::<MessageHover>().map(|hover| hover.0);

        let hover_color = use_animation(move |conf| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Nothing);

            let theme = consume_material_theme();

            let anim = AnimColor::new(theme.md.outline.as_u32(), theme.md.outline.as_argb_u32())
                .duration(Duration::from_secs_f32(0.1))
                .ease(Ease::InOut);

            if hovering.is_some_and(|s| s()) {
                anim
            } else {
                anim.into_reversed()
            }
        });

        rect()
            .horizontal()
            .spacing(8.)
            .child({
                let row = rect()
                    .width(Size::px(54.))
                    .horizontal()
                    .font_size(11)
                    .padding((4., 0., 0., 0.))
                    .main_align(Alignment::End);

                let datetime = Timestamp::try_from(
                    ulid::Ulid::from_string(&self.message.message.id)
                        .unwrap()
                        .datetime(),
                )
                .unwrap()
                .to_zoned(TimeZone::system());

                let hover_color = hover_color.read().value();

                if let Some(ts) = self.message.message.edited {
                    let edited = Timestamp::try_from(SystemTime::from(ts))
                        .unwrap()
                        .to_zoned(TimeZone::system());

                    row.color(theme.md.outline.as_argb_u32()).child(
                        StoatTooltip::new(
                            rect()
                                .spacing(8.)
                                .font_size(11)
                                .font_weight(500)
                                .child(
                                    label().max_lines(1).text(format!(
                                        "Sent {}",
                                        datetime.strftime("%d/%m/%Y %H:%M"),
                                    )),
                                )
                                .child(
                                    label().max_lines(1).text(format!(
                                        "Edited {}",
                                        edited.strftime("%d/%m/%Y %H:%M")
                                    )),
                                ),
                        )
                        .position(AttachedPosition::Top)
                        .child(label().text("(edited)")),
                    )
                } else if hovering.is_some_and(|s| s()) || hover_color != Color::TRANSPARENT {
                    row.color(hover_color).child(
                        StoatTooltip::new(
                            label()
                                .font_size(11)
                                .max_lines(1)
                                .font_weight(500)
                                .text(format!("Sent {}", datetime.strftime("%d/%m/%Y %H:%M"))),
                        )
                        .position(AttachedPosition::Top)
                        .child(label().text(datetime.strftime("%H:%M").to_string())),
                    )
                } else {
                    row
                }
            })
            .child(MessageContent {
                channel: self.channel.clone(),
                message: self.message.clone(),
            })
    }
}
