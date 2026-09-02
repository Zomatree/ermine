use std::collections::HashMap;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, Selection, SizeExt,
    components::{StoatButton, StoatButtonLayoutThemePartialExt, material::filled::grid_3x3},
    consume_material_theme, get_channel_server, map_optional_readable,
};

#[derive(PartialEq)]
pub struct ChannelMention {
    pub id: String,
    pub font_size: f32,
}

impl Component for ChannelMention {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Channels);

        let channels = radio.slice_current(|state| &state.channels);
        let channel = map_optional_readable::<HashMap<String, v0::Channel>, v0::Channel>(
            channels.into_readable(),
            {
                let id = self.id.clone();
                move |channels| channels.get(&id)
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

                move |_| {
                    let ids = channel.read().map(|channel| {
                        (
                            channel.id().to_string(),
                            get_channel_server(&channel).map(str::to_string),
                        )
                    });

                    if let Some((channel_id, server_id)) = ids {
                        channel_selection.set(Some((channel_id, None)));

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
                    .child(SvgViewer::new(grid_3x3()).size(Size::px(size)))
                    .child(
                        label()
                            .line_height(1.5)
                            .max_lines(1)
                            .font_size(self.font_size)
                            .text(
                                channel
                                    .read()
                                    .map(|c| c.name().unwrap_or("<TODO>").to_string())
                                    .unwrap_or_else(|| "Unknown Channel".to_string()),
                            ),
                    ),
            )
    }
}
