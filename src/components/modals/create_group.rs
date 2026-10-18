use std::collections::HashSet;

use freya::{
    prelude::*,
    radio::{use_radio, use_radio_station},
};
use stoat_models::v0;

use crate::{
    AppChannel, Selection,
    components::{
        Avatar, Dialog, SingleLineEntry, checkbox::StoatCheckbox, stoat_label, use_modals,
    },
    consume_material_theme, format_error, http, insert_channel,
};

#[derive(PartialEq)]
pub struct CreateGroup {}

impl Component for CreateGroup {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let station = use_radio_station();
        let radio = use_radio(AppChannel::Users);
        let users = radio.slice_current(|state| &state.users);
        let dm_channel = radio.slice_mut(AppChannel::SelectedChannel, |state| {
            &mut state.selected_channel
        });
        let main_selection = radio.slice_mut(AppChannel::Selection, |state| &mut state.selection);

        let name = use_state(String::new);
        let username = use_state(String::new);
        let selected = use_state(HashSet::<String>::new);

        let mut error = use_state(|| None);

        let filtered = use_memo({
            move || {
                let username = username.read();
                let users = users.read();

                let mut filtered = Vec::new();

                for user in users.values() {
                    if user.relationship != v0::RelationshipStatus::Friend {
                        continue;
                    }

                    if username.is_empty()
                        || user.username.contains(&*username)
                        || user
                            .display_name
                            .as_ref()
                            .is_some_and(|name| name.contains(&*username))
                    {
                        filtered.push(user.clone());
                    }
                }

                filtered
            }
        });

        Dialog::new()
            .title(label().line_height(2.).text("Create a new group"))
            .body(
                rect()
                    .spacing(8.)
                    .child(SingleLineEntry::new("Group Name", name))
                    .child(
                        stoat_label()
                            .color(theme.md.on_surface_variant)
                            .font_size(12.)
                            .font_weight(500)
                            .text("Select members to add"),
                    )
                    .child(SingleLineEntry::new("Search for users...", username))
                    .child(
                        VirtualScrollView::new({
                            move |item, _| {
                                CreateGroupItem {
                                    users: filtered,
                                    idx: item.index,
                                    selected,
                                }
                                .into_element()
                            }
                        })
                        .height(Size::px(320.))
                        .item_size(40.)
                        .length(filtered.read().len()),
                    )
                    .maybe_child(
                        error
                            .read()
                            .cloned()
                            .map(|error| label().color(theme.md.error).text(error)),
                    ),
            )
            .default_action("Close")
            .action_with_state(
                "Create",
                !selected.read().is_empty() && !name.read().is_empty(),
                {
                    move || {
                        let users = selected.read().cloned();
                        let name = name.read().cloned();
                        let mut main_selection = main_selection.clone();
                        let mut dm_channel = dm_channel.clone();

                        spawn(async move {
                            match http()
                                .create_group(&v0::DataCreateGroup {
                                    name,
                                    description: None,
                                    icon: None,
                                    users,
                                    nsfw: None,
                                })
                                .await
                            {
                                Ok(channel) => {
                                    let id = channel.id().to_string();
                                    main_selection.set(Selection::Home);
                                    dm_channel.set(Some((id, None)));
                                    insert_channel(channel, station);
                                    modals.write().pop_modal();
                                }
                                Err(e) => {
                                    error.set(Some(format_error(&e, "user")));
                                }
                            }
                        });

                        false
                    }
                },
            )
    }
}

#[derive(PartialEq)]
struct CreateGroupItem {
    pub users: Memo<Vec<v0::User>>,
    pub idx: usize,
    pub selected: State<HashSet<String>>,
}

impl Component for CreateGroupItem {
    fn render(&self) -> impl IntoElement {
        let users = self.users.read();
        let user = &users[self.idx];
        let is_selected = use_state(|| self.selected.read().contains(&user.id));

        use_side_effect({
            let id = user.id.clone();
            let mut selected = self.selected;

            move || {
                let is_selected = is_selected();

                if is_selected {
                    selected.write().insert(id.clone());
                } else {
                    selected.write().remove(&id);
                }
            }
        });

        StoatCheckbox::new(is_selected).child(
            rect()
                .horizontal()
                .spacing(8.)
                .cross_align(Alignment::Center)
                .child(Avatar::new(user.clone().into_readable(), None, 24.))
                .child(
                    stoat_label()
                        .max_lines(1)
                        .font_size(14.)
                        .font_weight(500)
                        .text(user.display_name.as_ref().unwrap_or(&user.username).clone()),
                ),
        )
    }
}
