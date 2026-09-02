use std::ops::Not;

use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel,
    components::{Channel, DMList, Friends, Welcome},
    consume_material_theme, use_config,
};

#[derive(Default, Clone, PartialEq)]
pub enum HomeSelection {
    #[default]
    Welcome,
    Friends,
}

#[derive(PartialEq)]
pub struct Home {}

impl Component for Home {
    fn render(&self) -> impl IntoElement {
        let config = use_config();
        let radio = use_radio(AppChannel::SelectedChannel);
        let theme = consume_material_theme();

        let selection = use_state(HomeSelection::default);
        let channel = radio.slice_current(|state| &state.selected_channel);

        rect()
            .corner_radius(CornerRadius::new(16., 0., 0., 16.))
            .background(theme.md.surface_container_low.as_argb_u32())
            .overflow(Overflow::Clip)
            .direction(Direction::Horizontal)
            .maybe_child(config.read().hide_channel_list.not().then(|| {
                rect()
                    // .spacing(8.)
                    .width(Size::px(248.))
                    .child(DMList { selection })
            }))
            .child(
                if let Some((channel_id, jump_message)) = channel.read().cloned() {
                    let channel = radio.slice(AppChannel::Channels, move |state| {
                        state.channels.get(&channel_id).unwrap()
                    });
                    Channel {
                        channel: channel.into_readable(),
                        server: None,
                        jump_message,
                    }
                    .into_element()
                } else {
                    match selection.read().clone() {
                        HomeSelection::Welcome => Welcome {}.into_element(),
                        HomeSelection::Friends => Friends {}.into_element(),
                    }
                },
            )
    }
}
