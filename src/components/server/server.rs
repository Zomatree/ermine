use std::ops::Not;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, ServerSettingsPage, SizeExt,
    components::{
        Channel, ChannelList, MaterialIcon, ModalValue, StoatButton,
        StoatButtonLayoutThemePartialExt, file_image, material::filled::settings, use_modals,
    },
    consume_material_theme, use_config,
};

#[derive(PartialEq)]
pub struct Server {
    pub server: Readable<v0::Server>,
}

impl Component for Server {
    fn render(&self) -> impl IntoElement {
        let config = use_config();
        let radio = use_radio(AppChannel::SelectedChannel);
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let selected_channel = radio.slice_current(|state| &state.selected_channel);
        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);
        let server_settings = radio.slice_mut(AppChannel::ServerSettingsPage, |state| {
            &mut state.server_settings_page
        });

        let server_header = rect()
            .horizontal()
            .content(Content::Flex)
            .cross_align(Alignment::Center)
            .spacing(8.)
            .child(
                label()
                    .font_size(16)
                    .text(self.server.read().name.clone())
                    .width(Size::flex(1.)),
            )
            .child(
                StoatButton::new()
                    .corner_radius(16.)
                    .on_press({
                        let server = self.server.clone();

                        move |e: Event<PressEventData>| {
                            e.stop_propagation();

                            let id = server.read().id.clone();

                            *server_settings.clone().write() =
                                Some((id, ServerSettingsPage::default()));
                        }
                    })
                    .child(
                        rect()
                            .padding(4.)
                            .child(MaterialIcon::new(settings()).size(Size::px(24.))),
                    ),
            )
            .into_element();

        rect()
            .corner_radius(CornerRadius::new(16., 0., 0., 16.))
            .background(theme.md.surface_container_low.as_argb_u32())
            .overflow(Overflow::Clip)
            .direction(Direction::Horizontal)
            .maybe_child(config.read().hide_channel_list.not().then(|| {
                rect()
                    .spacing(8.)
                    .child(
                        rect()
                            .padding((8., 0., 0., 8.))
                            .on_press({
                                let id = self.server.read().id.clone();
                                move |_| {
                                    modals
                                        .write()
                                        .push_modal(ModalValue::ServerInfo { server: id.clone() })
                                }
                            })
                            .on_pointer_enter(move |_| {
                                Cursor::set(CursorIcon::Pointer);
                            })
                            .on_pointer_leave(move |_| {
                                Cursor::set(CursorIcon::default());
                            })
                            .child(if let Some(banner) = &self.server.read().banner {
                                rect()
                                    .height(Size::px(120.))
                                    .width(Size::Fill)
                                    .corner_radius(16.)
                                    .overflow(Overflow::Clip)
                                    .child(file_image(&banner).aspect_ratio(AspectRatio::Max))
                                    .child(
                                        rect()
                                            .width(Size::Fill)
                                            .position(Position::new_absolute().bottom(0.))
                                            .layer(Layer::Relative(1))
                                            .padding((6., 14.))
                                            .background(
                                                LinearGradient::new()
                                                    .stop((Color::TRANSPARENT, 0.))
                                                    .stop((Color::BLACK, 90.)),
                                            )
                                            .corner_radius(CornerRadius::new_symmetric(0., 16.))
                                            .overflow(Overflow::Clip)
                                            .child(server_header.clone()),
                                    )
                            } else {
                                rect()
                                    .padding((0., 16.))
                                    .height(Size::px(48.))
                                    .center()
                                    .child(server_header)
                            }),
                    )
                    .child(ChannelList {
                        server: self.server.clone(),
                    })
                    .width(Size::px(248.))
            }))
            .child(
                if let Some(channel) = selected_channel.read().clone().and_then(|channel| {
                    if channels.read().contains_key(&channel) {
                        Some(radio.slice(AppChannel::Channels, move |state| {
                            state.channels.get(&channel).unwrap()
                        }))
                    } else {
                        None
                    }
                }) {
                    Channel {
                        channel: channel.into_readable(),
                        server: Some(self.server.clone()),
                    }
                    .into_element()
                } else {
                    "No selected channel".into_element()
                },
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.server.peek().id).into()
    }
}
