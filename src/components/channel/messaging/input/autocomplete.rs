use freya::{
    icons::lucide::hash,
    prelude::*,
    radio::use_radio,
    text_edit::{TextEditor, UseEditable},
};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{AutocompleteType, Avatar, StoatButton},
    parse_fill, use_material_theme,
};

#[derive(PartialEq, Eq, Clone, Debug)]
enum AutocompleteEntry {
    User(String),
    Channel(String),
    Emoji(String),
    Role(String),
}

#[derive(PartialEq)]
pub struct Autocomplete {
    pub autocomplete: AutocompleteType,
    pub query: String,
    pub editable: UseEditable,
    pub channel: Readable<v0::Channel>,
}

impl Component for Autocomplete {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);
        let servers = radio.slice_current(|state| &state.servers);
        let users = radio.slice(AppChannel::Users, |state| &state.users);
        let members = radio.slice(AppChannel::Members, |state| &state.members);
        let emojis = radio.slice(AppChannel::Emojis, |state| &state.emojis);
        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);

        let server = if let v0::Channel::TextChannel { server, .. } = &*self.channel.read() {
            Some(server.clone())
        } else {
            None
        };

        let theme = use_material_theme();
        let mut area = use_state(Area::default);

        let entries = use_hook({
            let members = members.clone();
            let servers = servers.clone();
            let emojis = emojis.clone();
            let server = server.clone();

            move || {
                let autocomplete = self.autocomplete;
                let channel = self.channel.clone();

                Effect::create_value(move || {
                    let mut entries = Vec::new();

                    match autocomplete {
                        AutocompleteType::User => match &*channel.read() {
                            v0::Channel::SavedMessages { user, .. } => {
                                entries.push(AutocompleteEntry::User(user.clone()));
                            }
                            v0::Channel::DirectMessage { recipients, .. }
                            | v0::Channel::Group { recipients, .. } => {
                                for user in recipients {
                                    entries.push(AutocompleteEntry::User(user.clone()));
                                }
                            }
                            v0::Channel::TextChannel { server, .. } => {
                                entries.extend(
                                    members
                                        .read()
                                        .get(server)
                                        .unwrap()
                                        .values()
                                        .map(|m| AutocompleteEntry::User(m.id.user.clone())),
                                );
                            }
                        },
                        AutocompleteType::Channel => {
                            let servers = servers.read();
                            let server = servers.get(server.as_ref().unwrap()).unwrap();

                            for channel in &server.channels {
                                entries.push(AutocompleteEntry::Channel(channel.clone()));
                            }
                        }
                        AutocompleteType::Emoji => {
                            let emojis = emojis.read();
                            let server = server.as_ref().unwrap();

                            for emoji in emojis.values() {
                                if let v0::EmojiParent::Server { id } = &emoji.parent
                                    && id == server
                                {
                                    entries.push(AutocompleteEntry::Emoji(emoji.id.clone()));
                                }
                            }
                        }
                        AutocompleteType::Role => {
                            let servers = servers.read();
                            let server = servers.get(server.as_ref().unwrap()).unwrap();

                            for role in server.roles.keys() {
                                entries.push(AutocompleteEntry::Role(role.clone()))
                            }
                        }
                    };

                    entries
                })
            }
        });

        let query = use_reactive(&self.query);

        let filtered = use_memo({
            let server = server.clone();
            let users = users.clone();
            let channels = channels.clone();
            let servers = servers.clone();

            move || {
                let query = query.read().to_lowercase();

                entries
                    .read()
                    .iter()
                    .filter(|entry| match entry {
                        AutocompleteEntry::User(user) => {
                            let users = users.read();
                            let user = users.get(user).unwrap();

                            user.username.to_lowercase().starts_with(&*query)
                                || user
                                    .display_name
                                    .as_ref()
                                    .is_some_and(|name| name.to_lowercase().starts_with(&*query))
                        }
                        AutocompleteEntry::Channel(channel) => {
                            let channels = channels.read();
                            let channel = channels.get(channel).unwrap();

                            channel
                                .name()
                                .is_some_and(|name| name.to_lowercase().starts_with(&*query))
                        }
                        AutocompleteEntry::Emoji(_) => todo!(),
                        AutocompleteEntry::Role(role) => {
                            let servers = servers.read();
                            let server = servers.get(server.as_ref().unwrap()).unwrap();
                            let role = server.roles.get(role).unwrap();

                            role.name.to_lowercase().starts_with(&*query)
                        }
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            }
        });

        rect()
            .height(Size::px((filtered.read().len() * 32 + 16).min(176) as f32))
            .padding((8., 0.))
            .width(Size::Fill)
            .background(theme.md.primary_container.as_argb_u32())
            .opacity(
                if area.read().height() != 0. && filtered.read().len() != 0 {
                    1.
                } else {
                    0.
                },
            )
            .color(theme.md.on_primary_container.as_argb_u32())
            .corner_radius(16.)
            .overflow(Overflow::Clip)
            .content(Content::Flex)
            .content(Content::Flex)
            .font_size(14)
            .position(Position::new_absolute().top(-(area.read().height() + 8.)))
            .layer(Layer::Overlay)
            .on_sized(move |e: Event<SizedEventData>| area.set(e.area))
            .child(
                VirtualScrollView::new({
                    let mut editable = self.editable;
                    let query = self.query.clone();
                    let server = server.clone();

                    move |idx, _| {
                        let filtered = filtered.read();

                        let entry = &filtered[idx];

                        StoatButton::new()
                            .on_press({
                                let entry = entry.clone();
                                let query = query.clone();

                                move |_| {
                                    let text = match &entry {
                                        AutocompleteEntry::User(id) => format!("<@{id}> "),
                                        AutocompleteEntry::Channel(id) => format!("<#{id}> "),
                                        AutocompleteEntry::Emoji(_) => todo!(),
                                        AutocompleteEntry::Role(id) => format!("<%{id}> "),
                                    };

                                    let mut editor = editable.editor_mut().write();
                                    let end_pos = editor.selection().end();
                                    let start_pos = end_pos - query.len() - 1;

                                    editor.remove(start_pos..end_pos);
                                    editor.insert(&text, start_pos);
                                    editor.selection_mut().move_to(start_pos + text.len());
                                    editor.selection_mut().set_as_cursor();
                                }
                            })
                            .child(match entry {
                                AutocompleteEntry::User(user) => {
                                    let users = users.read();
                                    let user = users.get(user).unwrap();

                                    rect()
                                        .horizontal()
                                        .cross_align(Alignment::Center)
                                        .padding((4., 16.))
                                        .spacing(8.)
                                        .width(Size::Fill)
                                        .child(Avatar::new(user.clone().into_readable(), None, 24.))
                                        .child(label().font_size(14.).text(user.username.clone()))
                                }
                                AutocompleteEntry::Channel(channel) => {
                                    let channels = channels.read();
                                    let channel = channels.get(channel).unwrap();

                                    rect()
                                        .horizontal()
                                        .cross_align(Alignment::Center)
                                        .padding((4., 16.))
                                        .spacing(8.)
                                        .width(Size::Fill)
                                        .child(
                                            svg(hash()).width(Size::px(24.)).height(Size::px(24.)),
                                        )
                                        .child(
                                            label()
                                                .font_size(14.)
                                                .text(channel.name().unwrap().to_string()),
                                        )
                                }
                                AutocompleteEntry::Emoji(_) => todo!(),
                                AutocompleteEntry::Role(role) => {
                                    let servers = servers.read();
                                    let server = servers.get(server.as_ref().unwrap()).unwrap();

                                    let role = server.roles.get(role).unwrap();

                                    let mut color = rect()
                                        .margin(6.)
                                        .corner_radius(6.)
                                        .width(Size::px(12.))
                                        .height(Size::px(12.));

                                    color.get_style().background = role
                                        .colour
                                        .as_deref()
                                        .and_then(parse_fill)
                                        .unwrap_or_else(|| Fill::Color(theme.md.surface_container_highest.as_argb_u32().into()));

                                    rect()
                                        .horizontal()
                                        .cross_align(Alignment::Center)
                                        .padding((4., 16.))
                                        .spacing(8.)
                                        .width(Size::Fill)
                                        .child(color)
                                        .child(label().font_size(14.).text(role.name.clone()))
                                }
                            })
                            .into_element()
                    }
                })
                .length(filtered.read().len())
                .item_size(32.),
            )
    }
}
