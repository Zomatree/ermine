use std::time::Duration;

use freya::{
    animation::{AnimNum, Ease, OnChange, OnCreation, UseAnimation, use_animation},
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;

use crate::{
    AppChannel, SizeExt, calculate_server_permissions,
    components::{CategoryContextMenu, ChannelButton, MaterialIcon, material::filled::expand_more},
    consume_material_theme, use_config, user_permissions_query,
};

#[derive(PartialEq)]
pub struct Category {
    pub server: Readable<v0::Server>,
    pub category: Readable<v0::Category>,
}

impl Component for Category {
    fn render(&self) -> impl IntoElement {
        let config = use_config();
        let radio = use_radio(AppChannel::Channels);
        let selected_channel =
            radio.slice(AppChannel::SelectedChannel, |state| &state.selected_channel);

        let channels = use_side_effect_value({
            let category = self.category.clone();

            move || {
                category
                    .read()
                    .channels
                    .iter()
                    .filter(|&channel_id| radio.read().channels.contains_key(channel_id))
                    .cloned()
                    .map(|channel_id| {
                        radio
                            .slice(AppChannel::Channels, move |state| {
                                state.channels.get(&channel_id).unwrap()
                            })
                            .into_readable()
                    })
                    .collect::<Vec<Readable<v0::Channel>>>()
            }
        });

        let is_expanded = use_memo({
            let category = self.category.clone();
            move || {
                !config
                    .read()
                    .collapsed_categories
                    .contains(&category.read().id)
            }
        });

        let animation = use_animation(move |conf| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Finish);

            let (start, end) = if *is_expanded.read() {
                (-90., 0.)
            } else {
                (0., -90.)
            };

            AnimNum::new(start, end)
                .ease(Ease::InOut)
                .duration(Duration::from_millis(100))
        });

        rect()
            .key(self.category.read().id.clone())
            .spacing(8.)
            .child(CategoryHeader {
                server: self.server.clone(),
                category: self.category.clone(),
                is_expanded: is_expanded.clone().into_readable(),
                animation,
            })
            .child(
                rect().maybe_child(
                    is_expanded
                        .read()
                        .then(|| {
                            rect().children(channels.read().iter().map(|channel| {
                                rect()
                                    .key(channel.peek().id())
                                    .child(ChannelButton {
                                        channel: channel.clone(),
                                    })
                                    .into_element()
                            }))
                        })
                        .or({
                            let selected = selected_channel.read();

                            if let Some(id) = &*selected
                                && let Some(channel) = channels
                                    .read()
                                    .iter()
                                    .find(|channel| channel.peek().id() == id)
                            {
                                Some(rect().key(channel.peek().id()).child(ChannelButton {
                                    channel: channel.clone(),
                                }))
                            } else {
                                None
                            }
                        }),
                ),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.category.peek().id).into()
    }
}

#[derive(PartialEq)]
pub struct CategoryHeader {
    pub server: Readable<v0::Server>,
    pub category: Readable<v0::Category>,
    pub is_expanded: Readable<bool>,
    pub animation: UseAnimation<AnimNum>,
}

impl Component for CategoryHeader {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);
        let theme = consume_material_theme();
        let mut config = use_config();
        let mut hovering = use_state(|| false);

        use_drop(move || {
            if hovering() {
                Cursor::set(CursorIcon::default());
            }
        });

        rect()
            .color(
                if hovering() {
                    theme.md.on_surface
                } else {
                    theme.md.on_surface_variant
                }
                .as_argb_u32(),
            )
            .width(Size::Fill)
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .on_pointer_enter(move |_| {
                Cursor::set(CursorIcon::Pointer);
            })
            .on_pointer_leave(move |_| {
                Cursor::set(CursorIcon::default());
            })
            .on_press({
                let category_id = self.category.read().id.clone();
                let is_expanded = self.is_expanded.clone();

                move |_| {
                    let collapsed = &mut config.write().collapsed_categories;

                    if *is_expanded.read() {
                        collapsed.insert(category_id.clone());
                    } else {
                        collapsed.remove(&category_id);
                    };
                }
            })
            .on_secondary_down({
                let server = self.server.clone();
                let category_id = self.category.read().id.clone();
                let radio = radio.clone();

                move |e: Event<PressEventData>| {
                    let server = server.read().clone();
                    let category_id = category_id.clone();
                    let radio = radio.clone();

                    spawn(async move {
                        let mut query = user_permissions_query(radio).server(server.clone());

                        let permissions = calculate_server_permissions(&mut query).await;

                        ContextMenu::open_from_event(
                            &e,
                            Menu::new().child(CategoryContextMenu {
                                server_id: server.id.clone(),
                                category_id: category_id.clone(),
                                current_permissions: permissions,
                            }),
                        );
                    });
                }
            })
            .child(
                rect()
                    .padding((10., 4., 0., 12.))
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::Start)
                    .spacing(4.)
                    .child(
                        label()
                            .font_size(13)
                            .max_lines(1)
                            .text_overflow(TextOverflow::Ellipsis)
                            .text(self.category.read().title.clone()),
                    )
                    .child(
                        MaterialIcon::new(expand_more())
                            .size(Size::px(12.))
                            .rotation(self.animation.read().value()),
                    ),
            )
    }
}
