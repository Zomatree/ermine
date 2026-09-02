use std::collections::HashMap;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, Selection, SizeExt,
    components::{
        MaterialIcon, StoatButton, StoatButtonLayoutThemePartialExt,
        material::{
            filled::grid_3x3,
            outlined::{chevron_right, message},
        },
    },
    consume_material_theme, get_channel_name, get_channel_server, map_optional_readable,
};

#[derive(PartialEq)]
pub struct MessageMention {
    pub id: String,
    pub channel_id: String,
    pub font_size: f32,
}

impl Component for MessageMention {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Channels);

        let channels = radio.slice_current(|state| &state.channels);
        let channel = map_optional_readable::<HashMap<String, v0::Channel>, v0::Channel>(
            channels.into_readable(),
            {
                let channel_id = self.channel_id.clone();
                move |channels| channels.get(&channel_id)
            },
        );

        let mut selection = radio.slice_mut(AppChannel::Selection, |state| &mut state.selection);
        let mut channel_selection = radio.slice_mut(AppChannel::SelectedChannel, |state| {
            &mut state.selected_channel
        });

        let size = self.font_size * (16. / 14.);

        StoatButton::new()
            .corner_radius(16.)
            .on_press({
                let channel = channel.clone();
                let id = self.id.clone();

                move |_| {
                    let ids = channel.read().map(|channel| {
                        (
                            channel.id().to_string(),
                            get_channel_server(&channel).map(str::to_string),
                        )
                    });

                    if let Some((channel_id, server_id)) = ids {
                        channel_selection.set(Some((channel_id, Some(id.clone()))));

                        selection.set(server_id.map(Selection::Server).unwrap_or(Selection::Home));
                    }
                }
            })
            .child(
                rect()
                    .padding((0., 6., 0., 2.))
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .spacing(4.)
                    .background(theme.md.primary.as_argb_u32())
                    .color(theme.md.on_primary.as_argb_u32())
                    .font_weight(FontWeight::SEMI_BOLD)
                    .child(MaterialIcon::new(grid_3x3()).size(Size::px(size)))
                    .child(if let Some(channel) = channel.read() {
                        rect()
                            .horizontal()
                            .cross_align(Alignment::Center)
                            .spacing(4.)
                            .child(get_channel_name(&radio, &channel))
                            .child(MaterialIcon::new(chevron_right()).size(Size::px(14.)))
                            .child(MaterialIcon::new(message()).size(Size::px(14.)))
                            .into_element()
                    } else {
                        label()
                            .line_height(1.5)
                            .max_lines(1)
                            .font_size(self.font_size)
                            .text("Unknown Channel")
                            .into_element()
                    }),
            )
    }
}
