use freya::{
    prelude::*,
    radio::use_radio,
    text_edit::{TextEditor, UseEditable},
};
use stoat_models::v0;
use stoat_permissions::{ChannelPermission, PermissionValue};

use crate::{
    AppChannel, SizeExt, calculate_channel_permissions,
    components::{
        AutocompleteType, Avatar, Emoji, StoatButton, StoatButtonColorsThemePartialExt,
        material::{MaterialIcon, filled::grid_3x3, outlined::alternate_email},
    },
    consume_material_theme, get_unicode_emojis, parse_fill, user_permissions_query,
};

#[derive(PartialEq, Eq, Clone, Debug)]
enum AutocompleteRoleEntry {
    Custom(String),
    Everyone,
    Online,
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum AutocompleteEntry {
    User(String),
    Channel(String),
    Emoji { value: String, name: String },
    Role(AutocompleteRoleEntry),
}

#[derive(PartialEq)]
pub struct Autocomplete {
    pub autocomplete: AutocompleteType,
    pub visible: State<bool>,
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

        let a11y_id = use_a11y();

        let server = if let v0::Channel::TextChannel { server, .. } = &*self.channel.read() {
            Some(server.clone())
        } else {
            None
        };

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

        let theme = consume_material_theme();

        let mut area = use_state(Area::default);

        let mut selected_idx = use_state(|| 0usize);

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
                            let unicode_emojis = get_unicode_emojis();

                            for (emoji_name, emoji) in unicode_emojis.iter() {
                                entries.push(AutocompleteEntry::Emoji {
                                    name: emoji_name.clone(),
                                    value: emoji.clone(),
                                });
                            }

                            let emojis = emojis.read();

                            for emoji in emojis.values() {
                                entries.push(AutocompleteEntry::Emoji {
                                    name: emoji.name.clone(),
                                    value: emoji.id.clone(),
                                });
                            }
                        }
                        AutocompleteType::Role => {
                            let servers = servers.read();
                            let server = servers.get(server.as_ref().unwrap()).unwrap();

                            for role in server.roles.keys() {
                                entries.push(AutocompleteEntry::Role(
                                    AutocompleteRoleEntry::Custom(role.clone()),
                                ))
                            }

                            if permissions()
                                .has_channel_permission(ChannelPermission::MentionEveryone)
                            {
                                entries.extend([
                                    AutocompleteEntry::Role(AutocompleteRoleEntry::Everyone),
                                    AutocompleteEntry::Role(AutocompleteRoleEntry::Online),
                                ]);
                            }
                        }
                    };

                    entries
                })
            }
        });

        let query = use_reactive(&self.query);

        use_side_effect({
            let mut visible = self.visible;
            move || {
                query.read();
                selected_idx.set(0);
                visible.set(true);
            }
        });

        let filtered = use_side_effect_value({
            let server = server.clone();
            let users = users.clone();
            let channels = channels.clone();
            let servers = servers.clone();

            move || {
                let query = query.read().to_lowercase();

                let mut filtered = entries
                    .read()
                    .iter()
                    .filter_map(|entry| match entry {
                        AutocompleteEntry::User(user) => {
                            let users = users.read();
                            let user = users.get(user).unwrap();

                            if let Some(name) = user.display_name.clone()
                                && name.to_lowercase().starts_with(&query)
                            {
                                Some((name, entry.clone()))
                            } else if user.username.to_lowercase().starts_with(&query) {
                                Some((user.username.clone(), entry.clone()))
                            } else {
                                None
                            }
                        }
                        AutocompleteEntry::Channel(channel) => {
                            let channels = channels.read();
                            let channel = channels.get(channel).unwrap();

                            if let Some(name) = channel.name()
                                && name.to_lowercase().starts_with(&query)
                            {
                                Some((name.to_string(), entry.clone()))
                            } else {
                                None
                            }
                        }
                        AutocompleteEntry::Emoji { name, .. } => {
                            if name.starts_with(&query) {
                                Some((name.clone(), entry.clone()))
                            } else {
                                None
                            }
                        }
                        AutocompleteEntry::Role(role_entry) => match role_entry {
                            AutocompleteRoleEntry::Custom(role) => {
                                let servers = servers.read();
                                let server = servers.get(server.as_ref().unwrap()).unwrap();
                                let role = server.roles.get(role).unwrap();

                                if role.name.to_lowercase().starts_with(&query) {
                                    Some((role.name.clone(), entry.clone()))
                                } else {
                                    None
                                }
                            }
                            AutocompleteRoleEntry::Everyone => {
                                if "everyone".starts_with(&query) {
                                    Some(("everyone".to_string(), entry.clone()))
                                } else {
                                    None
                                }
                            }
                            AutocompleteRoleEntry::Online => {
                                if "online".starts_with(&query) {
                                    Some(("online".to_string(), entry.clone()))
                                } else {
                                    None
                                }
                            }
                        },
                    })
                    .collect::<Vec<_>>();

                filtered.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(&name_b));
                filtered
            }
        });

        let filtered = use_memo({
            let mut visible = self.visible;
            move || {
                let filtered = filtered.read().cloned();

                visible.set(!filtered.is_empty());

                filtered
            }
        });

        let insert_autocomplete_value = {
            let editable = self.editable.clone();
            let query = self.query.clone();

            move |entry| {
                let mut text = match entry {
                    AutocompleteEntry::User(id) => format!("<@{id}>"),
                    AutocompleteEntry::Channel(id) => format!("<#{id}>"),
                    AutocompleteEntry::Emoji { value, .. } => {
                        if value.len() == 26 {
                            format!(":{value}:")
                        } else {
                            value
                        }
                    }
                    AutocompleteEntry::Role(AutocompleteRoleEntry::Custom(id)) => {
                        format!("<%{id}>")
                    }
                    AutocompleteEntry::Role(AutocompleteRoleEntry::Everyone) => {
                        "@everyone".to_string()
                    }
                    AutocompleteEntry::Role(AutocompleteRoleEntry::Online) => "@online".to_string(),
                };

                text.push(' ');

                let mut editor = editable.clone().editor_mut().write();
                let end_pos = editor.selection().end();
                let start_pos = end_pos - query.len() - 1;

                editor.remove(start_pos..end_pos);
                let len = editor.insert(&text, start_pos);
                editor.selection_mut().move_to(start_pos + len);
                editor.selection_mut().set_as_cursor();
            }
        };

        if (self.visible)() {
            rect()
                .a11y_id(a11y_id)
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
                .on_global_pointer_down({
                    let mut visible = self.visible;
                    move |e: Event<PointerEventData>| {
                        if !area().contains(e.global_location().to_f32()) {
                            visible.set(false);
                        }
                    }
                })
                .on_global_key_down({
                    let insert_autocomplete_value = insert_autocomplete_value.clone();
                    let mut visible = self.visible;
                    move |e: Event<KeyboardEventData>| {
                        if !visible() {
                            return;
                        };

                        if e.key == Key::Named(NamedKey::ArrowUp) {
                            let mut idx = selected_idx.write();
                            *idx = idx.wrapping_sub(1).min(filtered.read().len() - 1);
                            e.stop_propagation();
                            e.prevent_default();
                        } else if e.key == Key::Named(NamedKey::ArrowDown) {
                            let mut idx = selected_idx.write();

                            *idx = idx.wrapping_add(1);

                            if *idx >= filtered.read().len() {
                                *idx = 0;
                            }
                            e.stop_propagation();
                            e.prevent_default();
                        } else if e.key == Key::Named(NamedKey::Enter) {
                            let filtered = filtered.read();

                            if let Some((_, entry)) = filtered.get(selected_idx()) {
                                insert_autocomplete_value(entry.clone());
                                e.stop_propagation();
                                e.prevent_default();
                            }
                        } else if e.key == Key::Named(NamedKey::Escape) {
                            visible.set(false);
                        }
                    }
                })
                .child(
                    VirtualScrollView::new({
                        let server = server.clone();

                        move |item, _| {
                            let idx = item.index;
                            let filtered = filtered.read();

                            let entry = &filtered[idx].1;

                            StoatButton::new()
                                .on_hover(move |_| selected_idx.set(idx))
                                .maybe(selected_idx() == idx, |this| {
                                    this.background(
                                        Color::from(theme.md.on_surface.as_argb_u32()).with_a(20),
                                    )
                                })
                                .on_press({
                                    let entry = entry.clone();
                                    let insert_autocomplete_value =
                                        insert_autocomplete_value.clone();
                                    move |_| insert_autocomplete_value(entry.clone())
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
                                            .child(Avatar::new(
                                                user.clone().into_readable(),
                                                None,
                                                24.,
                                            ))
                                            .child(
                                                label().font_size(14.).text(user.username.clone()),
                                            )
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
                                                MaterialIcon::new(grid_3x3()).size(Size::px(24.)),
                                            )
                                            .child(
                                                label()
                                                    .font_size(14.)
                                                    .text(channel.name().unwrap().to_string()),
                                            )
                                    }
                                    AutocompleteEntry::Emoji { value, name } => rect()
                                        .horizontal()
                                        .cross_align(Alignment::Center)
                                        .padding((4., 16.))
                                        .spacing(8.)
                                        .width(Size::Fill)
                                        .child(Emoji::new(value.clone()).size(Size::px(24.)))
                                        .child(label().font_size(14.).text(format!(":{name}:"))),
                                    AutocompleteEntry::Role(AutocompleteRoleEntry::Custom(
                                        role,
                                    )) => {
                                        let servers = servers.read();
                                        let server = servers.get(server.as_ref().unwrap()).unwrap();

                                        let role = server.roles.get(role).unwrap();

                                        let mut color =
                                            rect().margin(6.).corner_radius(6.).size(Size::px(12.));

                                        color.get_style().background = role
                                            .colour
                                            .as_deref()
                                            .and_then(parse_fill)
                                            .unwrap_or_else(|| {
                                                Fill::Color(
                                                    theme
                                                        .md
                                                        .surface_container_highest
                                                        .as_argb_u32()
                                                        .into(),
                                                )
                                            });

                                        rect()
                                            .horizontal()
                                            .cross_align(Alignment::Center)
                                            .padding((4., 16.))
                                            .spacing(8.)
                                            .width(Size::Fill)
                                            .child(color)
                                            .child(label().font_size(14.).text(role.name.clone()))
                                    }
                                    AutocompleteEntry::Role(
                                        role @ (AutocompleteRoleEntry::Everyone
                                        | AutocompleteRoleEntry::Online),
                                    ) => {
                                        let title = match role {
                                            AutocompleteRoleEntry::Everyone => "everyone",
                                            AutocompleteRoleEntry::Online => "online",
                                            AutocompleteRoleEntry::Custom(_) => unreachable!(),
                                        };

                                        rect()
                                            .horizontal()
                                            .cross_align(Alignment::Center)
                                            .padding((4., 16.))
                                            .spacing(8.)
                                            .width(Size::Fill)
                                            .child(
                                                MaterialIcon::new(alternate_email())
                                                    .margin(6.)
                                                    .color(
                                                        theme
                                                            .md
                                                            .surface_container_highest
                                                            .as_argb_u32(),
                                                    )
                                                    .size(Size::px(12.)),
                                            )
                                            .child(label().font_size(14.).text(title))
                                    }
                                })
                                .into_element()
                        }
                    })
                    .length(filtered.read().len())
                    .item_size(32.),
                )
        } else {
            rect()
        }
    }
}
