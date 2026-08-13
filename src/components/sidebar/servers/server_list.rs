use std::collections::HashMap;

use freya::{prelude::*, radio::use_radio};
use indexmap::IndexSet;
use serde_json::to_value;

use crate::{
    AppChannel, OrderingSettings, Selection, SettingsPage, SizeExt,
    components::{
        CurrentUserButton, HomeButton, ModalValue, Reorder, ServerListButton, StoatButton,
        StoatButtonLayoutThemePartialExt, StoatTooltip,
        material::{
            MaterialIcon,
            filled::{add, explore, settings},
        },
        use_modals,
    },
    consume_material_theme, http,
};

#[derive(PartialEq)]
pub struct ServerList {}

impl Component for ServerList {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::Servers);
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let order_settings = radio.slice_mut(AppChannel::Settings("ordering"), |state| {
            &mut state.settings.ordering
        });

        let servers = radio.slice(AppChannel::Servers, |state| &state.servers);
        let members = radio.slice(AppChannel::Members, |state| &state.members);
        let user_id = radio.slice(AppChannel::UserId, |state| state.user_id.as_ref().unwrap());

        let order = use_side_effect_value({
            let servers = servers.clone();
            let order_settings = order_settings.clone();

            move || {
                let mut order = IndexSet::new();

                let server_map = servers.read();
                let order_settings = order_settings
                    .read()
                    .as_ref()
                    .and_then(|o| o.servers.as_ref())
                    .cloned();

                if let Some(ordering) = order_settings {
                    for id in ordering.clone() {
                        if server_map.contains_key(&id) {
                            order.insert(id);
                        };
                    }

                    for id in server_map.keys() {
                        if !ordering.contains(id) {
                            order.insert(id.clone());
                        }
                    }
                } else {
                    let mut join_dates = server_map
                        .keys()
                        .cloned()
                        .map(|id| {
                            let joined_at = members
                                .read()
                                .get(&id)
                                .unwrap()
                                .get(&*user_id.read())
                                .unwrap()
                                .joined_at
                                .clone();

                            (id, joined_at)
                        })
                        .collect::<Vec<_>>();

                    join_dates.sort_by(|(_, a), (_, b)| a.cmp(b));

                    order.extend(join_dates.into_iter().map(|(id, _)| id));
                };

                order
            }
        });

        let servers = use_side_effect_value({
            let servers = servers.clone();

            move || {
                let servers = servers.read();
                order
                    .read()
                    .iter()
                    .flat_map(|id| servers.get(id).cloned())
                    .collect::<Vec<_>>()
            }
        });

        use_side_effect_with_deps(&*servers.read(), {
            move |servers| {
                let value = OrderingSettings {
                    servers: Some(servers.iter().map(|server| server.id.clone()).collect()),
                };

                *order_settings.clone().write() = Some(value.clone());

                let mut settings = HashMap::new();
                settings.insert("ordering".to_string(), to_value(value).unwrap());

                spawn(async move {
                    http().set_settings(&settings).await.unwrap();
                });
            }
        });

        rect()
            .child(
                ScrollView::new()
                    .child(
                        rect()
                            .width(Size::fill())
                            .cross_align(Alignment::Center)
                            .child(HomeButton {})
                            .child(CurrentUserButton {})
                            .child(
                                rect()
                                    .height(Size::px(1.))
                                    .width(Size::px(32.))
                                    .margin((6., 0.))
                                    .background(theme.md.outline_variant.as_argb_u32()),
                            )
                            .child(
                                Reorder::new(servers, move |server| {
                                    ServerListButton {
                                        server: server.clone().into_readable(),
                                    }
                                    .into_element()
                                })
                                .width(Size::fill())
                                .cross_align(Alignment::Center)
                            )
                            .child(
                                StoatTooltip::new(
                                    label()
                                        .max_lines(1)
                                        .font_size(11.)
                                        .text("Create or join a server"),
                                )
                                .position(AttachedPosition::Right)
                                .child(
                                    rect()
                                        .width(Size::px(56.))
                                        .height(Size::px(56.))
                                        .center()
                                        .child(
                                            StoatButton::new()
                                                .corner_radius(42.)
                                                .child(
                                                    rect()
                                                        .center()
                                                        .width(Size::px(42.0))
                                                        .height(Size::px(42.0))
                                                        .background(
                                                            theme
                                                                .md
                                                                .surface_container_low
                                                                .as_argb_u32(),
                                                        )
                                                        .child(
                                                            MaterialIcon::new(add())
                                                                .size(Size::px(32.0)),
                                                        ),
                                                )
                                                .on_press(move |_| {
                                                    modals
                                                        .write()
                                                        .push_modal(ModalValue::CreateJoinServer);
                                                }),
                                        ),
                                ),
                            )
                            .child(
                                StoatTooltip::new(
                                    label()
                                        .max_lines(1)
                                        .font_size(11.)
                                        .text("Find new servers to join"),
                                )
                                .position(AttachedPosition::Right)
                                .child(
                                    rect()
                                        .width(Size::px(56.))
                                        .height(Size::px(56.))
                                        .center()
                                        .child(
                                            StoatButton::new()
                                                .corner_radius(42.)
                                                .child(
                                                    rect()
                                                        .center()
                                                        .width(Size::px(42.0))
                                                        .height(Size::px(42.0))
                                                        .background(
                                                            theme
                                                                .md
                                                                .surface_container_low
                                                                .as_argb_u32(),
                                                        )
                                                        .child(
                                                            MaterialIcon::new(explore())
                                                                .width(Size::px(32.0))
                                                                .height(Size::px(32.0)),
                                                        ),
                                                )
                                                .on_press(move |_| {
                                                    radio
                                                        .write_channel(AppChannel::Selection)
                                                        .selection = Selection::Discover;
                                                }),
                                        ),
                                ),
                            ),
                    )
                    .show_scrollbar(false)
                    .width(Size::px(56.))
                    .height(Size::func(|size| Some(size.parent - 56.))),
            )
            .child(
                StoatTooltip::new(label().max_lines(1).font_size(11.).text("Settings"))
                    .position(AttachedPosition::Right)
                    .child(
                        rect()
                            .width(Size::px(56.))
                            .height(Size::px(56.))
                            .center()
                            .child(
                                StoatButton::new()
                                    .corner_radius(42.)
                                    .child(
                                        rect()
                                            .center()
                                            .width(Size::px(42.0))
                                            .height(Size::px(42.0))
                                            .overflow(Overflow::Clip)
                                            .background(
                                                theme.md.surface_container_low.as_argb_u32(),
                                            )
                                            .child(
                                                MaterialIcon::new(settings()).size(Size::px(32.0)),
                                            ),
                                    )
                                    .on_press(move |_| {
                                        radio
                                            .write_channel(AppChannel::SettingsPage)
                                            .settings_page = Some(SettingsPage::default());
                                    }),
                            ),
                    ),
            )
    }
}
