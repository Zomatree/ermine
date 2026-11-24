use std::ops::Not;

use freya::{
    prelude::*,
    radio::{use_radio, use_radio_station},
};
use stoat_models::v0;

use crate::{
    AppChannel, Selection, SizeExt,
    components::{
        Avatar, GroupDMIcon, MaterialIcon, ServerIcon, StoatTooltip,
        material::outlined::{grid_3x3, search, sticky_note_2},
        stoat_label, stoat_paragraph, stoat_scrollbar, use_modals,
    },
    consume_material_theme, set_selection_and_channel,
    theme::Theme,
    use_config,
};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum QueryResultRef<'a> {
    Channel(&'a v0::Channel),
    Server(&'a v0::Server),
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum QueryResult {
    Channel(v0::Channel),
    Server(v0::Server),
}

#[derive(PartialEq)]
pub struct QuickSwitch {}

impl Component for QuickSwitch {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let modals = use_modals();
        let config = use_config();

        let station = use_radio_station();
        let radio = use_radio(AppChannel::PreviousChannels);
        let previous_channels = radio.slice_current(|state| &state.previous_channels);
        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);
        let servers = radio.slice(AppChannel::Servers, |state| &state.servers);
        let users = radio.slice(AppChannel::Users, |state| &state.users);
        let user_id = radio.slice(AppChannel::UserId, |state| state.user_id.as_ref().unwrap());

        let mut selected = use_state(|| 0usize);
        let query = use_state(String::new);

        let results = use_derived({
            let channels = channels.clone();

            move || {
                let mut query = query.read().cloned();

                let channels = channels.read();
                let servers = servers.read();
                let users = users.read();

                let mut results = Vec::new();

                if query.is_empty() {
                    for channel_id in previous_channels.read().iter().rev() {
                        if let Some(channel) = channels.get(channel_id) {
                            results.push(QueryResultRef::Channel(channel));
                        }
                    }
                } else if let Some(query_slice) = query.strip_prefix("@") {
                    query = query_slice.to_string();

                    for channel in channels.values() {
                        if matches!(
                            channel,
                            v0::Channel::DirectMessage { .. } | v0::Channel::Group { .. }
                        ) {
                            results.push(QueryResultRef::Channel(channel));
                        }
                    }
                } else if let Some(query_slice) = query.strip_prefix("#") {
                    query = query_slice.to_string();

                    for channel in channels.values() {
                        if matches!(channel, v0::Channel::TextChannel { .. }) {
                            results.push(QueryResultRef::Channel(channel));
                        }
                    }
                } else if let Some(query_slice) = query.strip_prefix("!") {
                    query = query_slice.to_string();

                    for channel in channels.values() {
                        if matches!(channel, v0::Channel::TextChannel { voice: Some(_), .. }) {
                            results.push(QueryResultRef::Channel(channel));
                        }
                    }
                } else if let Some(query_slice) = query.strip_prefix("*") {
                    query = query_slice.to_string();

                    for server in servers.values() {
                        results.push(QueryResultRef::Server(server));
                    }
                } else {
                    for channel in channels.values() {
                        results.push(QueryResultRef::Channel(channel));
                    }

                    for server in servers.values() {
                        results.push(QueryResultRef::Server(server));
                    }
                };

                if !query.is_empty() {
                    let query = query.to_lowercase();
                    let user_id = user_id.read();

                    results
                        .into_iter()
                        .filter(|result| match result {
                            QueryResultRef::Channel(
                                v0::Channel::Group { name, .. }
                                | v0::Channel::TextChannel { name, .. },
                            ) => name.to_lowercase().starts_with(&query),
                            QueryResultRef::Channel(v0::Channel::DirectMessage {
                                recipients,
                                ..
                            }) => {
                                let recipient_id =
                                    recipients.iter().find(|&id| id != &*user_id).unwrap();
                                let user = users.get(recipient_id).unwrap();

                                user.username.to_lowercase().starts_with(&query)
                                    || user
                                        .display_name
                                        .as_ref()
                                        .is_some_and(|name| name.to_lowercase().starts_with(&query))
                            }
                            QueryResultRef::Channel(v0::Channel::SavedMessages { .. }) => {
                                "saved notes".starts_with(&query)
                            }
                            QueryResultRef::Server(server) => {
                                server.name.to_lowercase().starts_with(&query)
                            }
                        })
                        .map(|result| match result {
                            QueryResultRef::Channel(channel) => {
                                QueryResult::Channel(channel.clone())
                            }
                            QueryResultRef::Server(server) => QueryResult::Server(server.clone()),
                        })
                        .collect::<Vec<_>>()
                } else {
                    results
                        .into_iter()
                        .map(|result| match result {
                            QueryResultRef::Channel(channel) => {
                                QueryResult::Channel(channel.clone())
                            }
                            QueryResultRef::Server(server) => QueryResult::Server(server.clone()),
                        })
                        .collect::<Vec<_>>()
                }
            }
        });

        use_side_effect_with_deps(&*query.read(), move |_| {
            selected.set(0);
        });

        let select_result = EventHandler::new(move |result| {
            match result {
                QueryResult::Channel(
                    v0::Channel::DirectMessage { id, .. }
                    | v0::Channel::Group { id, .. }
                    | v0::Channel::SavedMessages { id, .. },
                ) => set_selection_and_channel(Selection::Home, Some((id, None)), station),
                QueryResult::Channel(v0::Channel::TextChannel { id, server, .. }) => {
                    set_selection_and_channel(Selection::Server(server), Some((id, None)), station)
                }
                QueryResult::Server(server) => {
                    let channel_id = {
                        let channels = channels.read();

                        config
                            .read()
                            .last_channels
                            .get(&server.id)
                            .filter(|&id| channels.contains_key(id))
                            .cloned()
                            .or_else(|| {
                                server
                                    .channels
                                    .iter()
                                    .find(|&id| channels.contains_key(id))
                                    .cloned()
                            })
                    };

                    set_selection_and_channel(
                        Selection::Server(server.id),
                        channel_id.map(|id| (id, None)),
                        station,
                    )
                }
            };

            modals.clone().write().pop_modal();
        });

        let results_values = results.read();

        rect()
            .width(Size::px(480.))
            .corner_radius(28.)
            .background(theme.md.surface_container_high)
            .padding(24.)
            .spacing(8.)
            .child(
                rect().font_size(16.).child(
                    Input::new(query)
                        .auto_focus(true)
                        .height(Size::px(56.))
                        .corner_radius(28.)
                        .width(Size::Fill)
                        .color(theme.md.on_surface)
                        .background(theme.md.surface_container_highest)
                        .border_fill(Color::TRANSPARENT)
                        .focus_border_fill(Color::TRANSPARENT)
                        .focus_background(theme.md.surface_container_highest)
                        .leading(
                            rect().padding((0., 0., 0., 8.)).child(
                                MaterialIcon::new(search())
                                    .color(theme.md.on_surface_variant)
                                    .size(Size::px(20.)),
                            ),
                        )
                        .placeholder("Where would you like to go?")
                        .placeholder_color(theme.md.on_surface_variant)
                        .on_pre_key_down({
                            let select_result = select_result.clone();

                            move |e: Event<KeyboardEventData>| {
                                let results = results.read();
                                let selected_idx = selected();

                                if e.key == Key::Named(NamedKey::ArrowUp) {
                                    if selected_idx == 0 {
                                        selected.set(results.len().saturating_sub(1))
                                    } else {
                                        selected.set(selected_idx - 1);
                                    }

                                    false
                                } else if e.key == Key::Named(NamedKey::ArrowDown) {
                                    if selected_idx == results.len().saturating_sub(1) {
                                        selected.set(0)
                                    } else {
                                        selected.set(selected_idx + 1);
                                    }

                                    false
                                } else if e.key == Key::Named(NamedKey::Enter) {
                                    if let Some(result) = results.get(selected_idx) {
                                        select_result.call(result.clone())
                                    }

                                    false
                                } else if let Key::Character(c) = &e.key
                                    && c == "k"
                                    && e.modifiers.contains(Modifiers::ctrl_or_meta())
                                {
                                    false
                                } else {
                                    true
                                }
                            }
                        }),
                ),
            )
            .maybe_child(query.read().is_empty().then(|| {
                if results_values.is_empty() {
                    rect()
                        .width(Size::Fill)
                        .center()
                        .padding(16.)
                        .child(
                            stoat_label()
                                .font_size(15.)
                                .font_weight(500)
                                .color(theme.md.on_surface_variant)
                                .text("No recent channels"),
                        )
                        .into_element()
                } else {
                    stoat_label()
                        .font_size(12.)
                        .font_weight(600)
                        .color(theme.md.on_surface_variant)
                        .text("PREVIOUS CHANNELS")
                        .into_element()
                }
            }))
            .maybe_child(results_values.is_empty().not().then(|| {
                VirtualScrollView::new(move |item, _| {
                    QuickSwitchEntry {
                        idx: item.index,
                        results,
                        selected,
                        select_result: select_result.clone(),
                    }
                    .into_element()
                })
                .height(Size::px((results_values.len() * 36).min(360) as f32))
                .scrollbar(stoat_scrollbar)
                .item_size(36.)
                .length(results_values.len())
            }))
            .child(
                rect()
                    .spacing(6.)
                    .horizontal()
                    .font_size(12.)
                    .child(
                        stoat_label()
                            .color(theme.md.primary)
                            .font_weight(700)
                            .text("Pro tip:"),
                    )
                    .child(
                        stoat_paragraph()
                            .color(theme.md.on_surface_variant)
                            .font_weight(500)
                            .span("Start searches with ")
                            .child(tip_key("@", "Usernames", &theme))
                            .span(" ")
                            .child(tip_key("#", "Channels", &theme))
                            .span(" ")
                            .child(tip_key("!", "Voice Channels", &theme))
                            .span(" ")
                            .child(tip_key("*", "Servers", &theme))
                            .span(" to narrow results."),
                    ),
            )
    }
}

fn tip_key(text: &'static str, tooltip: &'static str, theme: &Theme) -> impl IntoElement {
    StoatTooltip::new(label().font_size(11.).max_lines(1).text(tooltip))
        .position(AttachedPosition::Top)
        .child(
            rect()
                .padding((1., 6.))
                .background(theme.md.surface_container_highest)
                .color(theme.md.on_surface)
                .corner_radius(8.)
                .font_family("Fira Code")
                .child(stoat_label().text(text)),
        )
}

#[derive(PartialEq)]
struct QuickSwitchEntry {
    pub idx: usize,

    pub results: State<Vec<QueryResult>>,
    pub selected: State<usize>,
    pub select_result: EventHandler<QueryResult>,
}

impl Component for QuickSwitchEntry {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::PreviousChannels);
        let users = radio.slice(AppChannel::Users, |state| &state.users);
        let user_id = radio.slice(AppChannel::UserId, |state| state.user_id.as_ref().unwrap());
        let servers = radio.slice(AppChannel::Servers, |state| &state.servers);

        let result = self.results.read()[self.idx].clone();

        let (icon, name, secondary, server) = match &result {
            QueryResult::Channel(v0::Channel::DirectMessage { recipients, .. }) => {
                let user_id = user_id.read();

                let recipient_id = recipients.iter().find(|&id| id != &*user_id).unwrap();
                let user = users.read().get(recipient_id).unwrap().clone();

                (
                    Avatar::new(user.clone(), None, 24.).into_element(),
                    user.display_name.as_ref().unwrap_or(&user.username).clone(),
                    user.display_name.is_some().then(|| user.username.clone()),
                    None,
                )
            }
            QueryResult::Channel(channel @ v0::Channel::Group { name, .. }) => (
                GroupDMIcon::new(channel.clone())
                    .size(24.)
                    .invert(true)
                    .into_element(),
                name.clone(),
                None,
                None,
            ),
            QueryResult::Channel(v0::Channel::SavedMessages { .. }) => (
                MaterialIcon::new(sticky_note_2())
                    .size(Size::px(24.))
                    .into_element(),
                "Saved Notes".to_string(),
                None,
                None,
            ),
            QueryResult::Channel(v0::Channel::TextChannel {
                id, name, server, ..
            }) => {
                let servers = servers.read();
                let server = servers.get(server).unwrap();
                (
                    MaterialIcon::new(grid_3x3())
                        .size(Size::px(24.))
                        .into_element(),
                    name.clone(),
                    server.categories.as_ref().and_then(|categories| {
                        categories
                            .iter()
                            .filter(|c| &c.id != "default")
                            .find(|c| c.channels.contains(&id))
                            .map(|c| c.title.clone())
                    }),
                    Some(server.name.clone()),
                )
            }
            QueryResult::Server(server) => (
                ServerIcon::new(server.clone(), 24.)
                    .corner_radius(12.)
                    .into_element(),
                server.name.clone(),
                None,
                None,
            ),
        };

        rect()
            .horizontal()
            .content(Content::Flex)
            .padding((6., 12.))
            .corner_radius(12.)
            .maybe(*self.selected.read() == self.idx, |this| {
                this.background(theme.md.surface_container_highest)
            })
            .color(theme.md.on_surface_variant)
            .cursor(CursorIcon::Pointer)
            .spacing(8.)
            .cross_align(Alignment::Center)
            .on_pointer_over({
                let mut selected = self.selected;
                let idx = self.idx;
                move |_| {
                    selected.set_if_modified(idx);
                }
            })
            .on_press({
                let select_result = self.select_result.clone();
                move |_| select_result.call(result.clone())
            })
            .child(icon)
            .child(
                rect()
                    .width(Size::flex(1.))
                    .horizontal()
                    .spacing(8.)
                    .cross_align(Alignment::End)
                    .child(stoat_label().font_size(14.).font_weight(500).text(name))
                    .maybe_child(secondary.map(|text| {
                        stoat_label()
                            .font_size(11.)
                            .font_weight(500)
                            .opacity(0.7)
                            .text(text)
                    })),
            )
            .maybe_child(
                server.map(|text| stoat_label().font_size(13.).font_weight(500).text(text)),
            )
    }
}
