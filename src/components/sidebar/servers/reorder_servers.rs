use std::collections::HashSet;

use freya::{
    animation::{AnimNum, use_animation_transition},
    prelude::*,
    radio::use_radio,
};
use itertools::Itertools;
use stoat_models::v0;
use ulid::Ulid;

use crate::{
    AppChannel, NotificationBadge, Selection, ServerFolder, SizeExt,
    components::{
        ServerFolderMenu, ServerIcon, ServerListButton, StoatTooltip,
        material::{MaterialIcon, filled::folder as filled_folder},
        stoat_label,
    },
    consume_material_theme, fast_func, get_unread_badge, is_server_muted, parse_fill,
};

#[derive(PartialEq, Debug, Clone)]
pub enum Entry {
    Server {
        server: v0::Server,
        folder: Option<String>,
    },
    Folder {
        id: String,
    },
}

impl Entry {
    pub fn id(&self) -> &str {
        match self {
            Self::Server { server, .. } => &server.id,
            Self::Folder { id } => id,
        }
    }

    pub fn is_server(&self) -> bool {
        matches!(self, Self::Server { .. })
    }

    pub fn is_folder(&self) -> bool {
        matches!(self, Self::Folder { .. })
    }
}

#[derive(PartialEq)]
pub struct ReorderServers {}

impl Component for ReorderServers {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);

        let order_settings = radio.slice_mut(AppChannel::Settings("ordering"), |state| {
            &mut state.settings.ordering
        });
        let server_folder_settings = radio
            .slice_mut(AppChannel::Settings("server-folders"), |state| {
                &mut state.settings.server_folders
            });

        let servers = radio.slice(AppChannel::Servers, |state| &state.servers);
        let members = radio.slice(AppChannel::Members, |state| &state.members);
        let user_id = radio.slice(AppChannel::UserId, |state| state.user_id.as_ref().unwrap());

        use_side_effect({
            let mut server_folder_settings = server_folder_settings.clone();
            move || {
                if let Some(folders) = &mut *server_folder_settings.write() {
                    folders.folders.retain(|f| !f.servers.is_empty());
                }
            }
        });

        let entries = use_side_effect_value({
            let order_settings = order_settings.clone();
            let server_folder_settings = server_folder_settings.clone();
            let servers = servers.clone();

            move || {
                let mut entries = Vec::new();
                let folders = server_folder_settings
                    .read()
                    .cloned()
                    .unwrap_or_default()
                    .folders;
                let order_settings = order_settings.read().cloned().unwrap_or_default();
                let mut order = order_settings
                    .server_sidebar
                    .or(order_settings.servers)
                    .unwrap_or_default();

                let servers = servers.read();
                let mut unordered = Vec::new();

                for server_id in servers.keys() {
                    if !order.contains(server_id) {
                        unordered.push((
                            server_id.clone(),
                            members
                                .read()
                                .get(server_id)
                                .unwrap()
                                .get(&*user_id.read())
                                .unwrap()
                                .joined_at
                                .clone(),
                        ));
                    }
                }

                unordered.sort_by(|(_, a), (_, b)| a.cmp(b));
                order.extend(unordered.into_iter().map(|(id, _)| id));

                let mut found = HashSet::new();

                for entry in order {
                    if let Some(folder) = folders.iter().find(|f| &f.id == &entry) {
                        entries.push(Entry::Folder { id: entry });

                        for server in &folder.servers {
                            found.insert(server.clone());
                        }
                    } else if found.contains(&entry) {
                        continue;
                    } else if let Some(server) = servers.get(&entry).cloned() {
                        entries.push(Entry::Server {
                            server,
                            folder: None,
                        });
                    }
                }

                entries
            }
        });

        let dragging = use_state(|| None);

        rect().width(Size::Fill).children(
            (0..)
                .into_iter()
                .map(|idx| {
                    InsertArea {
                        idx,
                        folder: None,
                        dragging,
                        entries,
                    }
                    .into_element()
                })
                .interleave_shortest(entries.read().iter().cloned().map(|entry| {
                    match entry {
                        Entry::Folder { id } => FolderEntry {
                            id,
                            dragging,
                            entries,
                        }
                        .into_element(),
                        Entry::Server { server, folder } => ServerEntry {
                            server,
                            folder,
                            dragging,
                            entries,
                        }
                        .into_element(),
                    }
                })),
        )
    }
}

#[derive(PartialEq)]
pub struct InsertArea {
    pub idx: usize,
    pub folder: Option<String>,

    pub dragging: State<Option<Entry>>,
    pub entries: State<Vec<Entry>>,
}

impl Component for InsertArea {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Settings("ordering"));
        let ordering = radio.slice_mut_current(|state| {
            state
                .settings
                .ordering
                .get_or_insert_default()
                .server_sidebar
                .get_or_insert_default()
        });
        let folders = radio.slice_mut(AppChannel::Settings("server-folders"), |state| {
            &mut state
                .settings
                .server_folders
                .get_or_insert_default()
                .folders
        });

        let mut hovering = use_state(|| false);

        let in_folder = self.folder.is_some();

        rect().height(Size::px(0.)).child(
            rect()
                .position(Position::new_absolute().top(-18.))
                .layer(Layer::Overlay)
                .maybe_child(self.dragging.read().is_some().then(|| {
                    DropZone::<Entry>::new({
                        let idx = self.idx;
                        let folder = self.folder.clone();
                        let mut sidebar = ordering.clone();
                        let mut folders = folders.clone();
                        let mut entries = self.entries;

                        move |entry| {
                            if let Entry::Server {
                                server,
                                folder: entry_folder,
                            } = &entry
                                && (entry_folder.is_some() || folder.is_some())
                            {
                                if let Some(entry_folder) = &entry_folder
                                    && let Some(folder) = &folder
                                {
                                    if folder == entry_folder {
                                        let mut folders = folders.write();
                                        let servers = &mut folders
                                            .iter_mut()
                                            .find(|f| &f.id == folder)
                                            .unwrap()
                                            .servers;

                                        let from =
                                            servers.iter().position(|s| s == &server.id).unwrap();

                                        if from == idx {
                                            return;
                                        }

                                        if from > idx {
                                            servers[idx..=from].rotate_right(1);
                                        } else {
                                            if idx == servers.len() {
                                                servers[from..].rotate_left(1);
                                            } else {
                                                servers[from..idx].rotate_left(1);
                                            }
                                        }
                                    } else {
                                        let mut folders = folders.write();

                                        folders
                                            .iter_mut()
                                            .find(|f| &f.id == entry_folder)
                                            .unwrap()
                                            .servers
                                            .retain(|s| s != &server.id);

                                        folders
                                            .iter_mut()
                                            .find(|f| &f.id == folder)
                                            .unwrap()
                                            .servers
                                            .insert(idx, server.id.clone());
                                    }
                                } else if let Some(entry_folder) = &entry_folder {
                                    folders
                                        .write()
                                        .iter_mut()
                                        .find(|f| &f.id == entry_folder)
                                        .unwrap()
                                        .servers
                                        .retain(|s| s != &server.id);

                                    entries.write().insert(
                                        idx,
                                        Entry::Server {
                                            server: server.clone(),
                                            folder: None,
                                        },
                                    );
                                } else if let Some(folder) = &folder {
                                    entries.write().retain(|e| e.id() != &server.id);

                                    folders
                                        .write()
                                        .iter_mut()
                                        .find(|f| &f.id == folder)
                                        .unwrap()
                                        .servers
                                        .insert(idx, server.id.clone());
                                }
                            } else {
                                let mut entries = entries.write();
                                let from =
                                    entries.iter().position(|e| e.id() == entry.id()).unwrap();

                                if from == idx {
                                    return;
                                }

                                if from > idx {
                                    entries[idx..=from].rotate_right(1);
                                } else {
                                    if idx == entries.len() {
                                        entries[from..].rotate_left(1);
                                    } else {
                                        entries[from..idx].rotate_left(1);
                                    }
                                }
                            }

                            let mut new_sidebar = Vec::new();
                            {
                                let folders = folders.read();

                                for entry in entries.read().iter() {
                                    match entry {
                                        Entry::Server { server, folder: _ } => {
                                            new_sidebar.push(server.id.clone())
                                        }
                                        Entry::Folder { id } => {
                                            new_sidebar.push(id.clone());
                                            if let Some(folder) =
                                                folders.iter().find(|f| &f.id == id)
                                            {
                                                for server in &folder.servers {
                                                    new_sidebar.push(server.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            folders.write().retain(|f| !f.servers.is_empty());
                            sidebar.set(new_sidebar);
                        }
                    })
                    .on_drag_over({
                        let dragging = self.dragging;
                        move |is_in| {
                            if is_in {
                                if in_folder
                                    && dragging
                                        .read()
                                        .as_ref()
                                        .is_none_or(|entry| entry.is_folder())
                                {
                                    return;
                                };

                                hovering.set(true);
                            } else {
                                hovering.set(false);
                            }
                        }
                    })
                    .child(
                        rect()
                            .width(Size::px(56.))
                            .height(Size::px(36.))
                            .center()
                            .maybe_child(
                                (hovering()
                                    && self.dragging.read().as_ref().is_some_and(|entry| {
                                        if entry.is_folder() { !in_folder } else { true }
                                    }))
                                .then(|| {
                                    rect()
                                        .width(Size::px(44.))
                                        .height(Size::px(4.))
                                        .background(theme.md.primary)
                                        .corner_radius(2.)
                                }),
                            ),
                    )
                })),
        )
    }
}

#[derive(PartialEq)]
pub struct ServerEntry {
    pub server: v0::Server,
    pub folder: Option<String>,

    pub dragging: State<Option<Entry>>,
    pub entries: State<Vec<Entry>>,
}

impl Component for ServerEntry {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Servers);

        let ordering = radio.slice_mut(AppChannel::Settings("ordering"), |state| {
            state
                .settings
                .ordering
                .get_or_insert_default()
                .server_sidebar
                .get_or_insert_default()
        });
        let folders = radio.slice_mut(AppChannel::Settings("server-folders"), |state| {
            &mut state
                .settings
                .server_folders
                .get_or_insert_default()
                .folders
        });

        let mut hovering = use_state(|| false);

        rect()
            .size(Size::px(56.))
            .child(
                rect()
                    .size(Size::px(56.))
                    .corner_radius(28.)
                    .border(hovering().then(|| Border::new().fill(theme.md.primary).width(2.)))
                    .on_pointer_down(|evt: Event<PointerEventData>| evt.stop_propagation())
                    .child(
                        DragZone::new(Entry::Server {
                            server: self.server.clone(),
                            folder: self.folder.clone(),
                        })
                        .child(
                            rect()
                                .opacity(
                                    if self
                                        .dragging
                                        .read()
                                        .as_ref()
                                        .is_some_and(|entry| entry.id() == &self.server.id)
                                    {
                                        0.35
                                    } else {
                                        1.
                                    },
                                )
                                .child(ServerListButton {
                                    server: self.server.clone(),
                                }),
                        )
                        .drag_element(DragServer {
                            server: self.server.clone(),
                            folder: self.folder.clone(),
                            dragging: self.dragging,
                        }),
                    ),
            )
            .maybe_child(
                (self.folder.is_none()
                    && self
                        .dragging
                        .read()
                        .as_ref()
                        .is_some_and(|entry| entry.id() != &self.server.id))
                .then(|| {
                    rect()
                        .position(Position::new_absolute().top(18.))
                        .layer(Layer::Overlay)
                        .child(
                            DropZone::<Entry>::new({
                                let mut folders = folders.clone();
                                let mut entries = self.entries;
                                let mut sidebar = ordering.clone();
                                let current_id = self.server.id.clone();
                                move |entry| {
                                    let Entry::Server { server, folder } = entry else {
                                        return;
                                    };

                                    if let Some(folder) = folder {
                                        folders
                                            .write()
                                            .iter_mut()
                                            .find(|f| &f.id == &folder)
                                            .unwrap()
                                            .servers
                                            .retain(|s| s != &server.id);
                                    }

                                    let folder_id = format!("folder-{}", Ulid::new().to_string());

                                    {
                                        let mut entries = entries.write();

                                        entries.retain(|e| e.id() != &server.id);

                                        let idx = entries
                                            .iter()
                                            .position(|e| e.id() == &current_id)
                                            .unwrap();

                                        entries[idx] = Entry::Folder {
                                            id: folder_id.clone(),
                                        };
                                    }

                                    folders.write().push(ServerFolder {
                                        id: folder_id,
                                        name: "Untitled Folder".to_string(),
                                        colour: None,
                                        collapsed: false,
                                        servers: vec![current_id.clone(), server.id],
                                    });

                                    let mut new_sidebar = Vec::new();

                                    {
                                        let folders = folders.read();

                                        for entry in entries.read().iter() {
                                            match entry {
                                                Entry::Server { server, folder: _ } => {
                                                    new_sidebar.push(server.id.clone())
                                                }
                                                Entry::Folder { id } => {
                                                    new_sidebar.push(id.clone());
                                                    if let Some(folder) =
                                                        folders.iter().find(|f| &f.id == id)
                                                    {
                                                        for server in &folder.servers {
                                                            new_sidebar.push(server.clone());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    folders.write().retain(|f| !f.servers.is_empty());
                                    sidebar.set(new_sidebar);
                                }
                            })
                            .on_drag_over({
                                let dragging = self.dragging;

                                move |is_in| {
                                    hovering.set(
                                        is_in
                                            && dragging
                                                .read()
                                                .as_ref()
                                                .is_some_and(|e| e.is_server()),
                                    );
                                }
                            })
                            .child(rect().width(Size::px(56.)).height(Size::px(20.))),
                        )
                }),
            )
    }
}

#[derive(PartialEq)]
pub struct DragServer {
    pub server: v0::Server,
    pub folder: Option<String>,

    pub dragging: State<Option<Entry>>,
}

impl Component for DragServer {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);
        let state = radio.read();

        let mut dragging = self.dragging;

        use_hook(|| {
            dragging.set(Some(Entry::Server {
                server: self.server.clone(),
                folder: self.folder.clone(),
            }));
        });
        use_drop(move || dragging.set(None));

        rect()
            .opacity(0.85)
            .child(ServerIcon::new(self.server.clone(), 42.).corner_radius(21.))
    }
}

#[derive(PartialEq)]
pub struct FolderEntry {
    pub id: String,

    pub dragging: State<Option<Entry>>,
    pub entries: State<Vec<Entry>>,
}

impl Component for FolderEntry {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Servers);
        let servers = radio.slice_current(|state| &state.servers);
        let selection = radio.slice(AppChannel::Selection, |state| &state.selection);
        let ordering = radio.slice_mut(AppChannel::Settings("ordering"), |state| {
            state
                .settings
                .ordering
                .get_or_insert_default()
                .server_sidebar
                .get_or_insert_default()
        });
        let folders = radio.slice_mut(AppChannel::Settings("server-folders"), |state| {
            &mut state
                .settings
                .server_folders
                .get_or_insert_default()
                .folders
        });

        let folder = folders
            .read()
            .iter()
            .find(|f| &f.id == &self.id)
            .unwrap()
            .clone();

        let mut hovering = use_state(|| false);
        let mut drag_hovering = use_state(|| false);

        let opacity = if self
            .dragging
            .read()
            .as_ref()
            .is_some_and(|entry| entry.id() == &self.id)
        {
            0.35
        } else {
            1.
        };

        let notifications_settings = radio.slice(AppChannel::Settings("notifications"), |state| {
            &state.settings.notifications
        });
        let unreads = radio.slice(AppChannel::ChannelUnreads, |state| &state.channel_unreads);
        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);

        let badge = use_memo({
            let folder = use_reactive(&folder);
            let channels = channels.clone();
            let notifications_settings = notifications_settings.clone().into_readable();
            let servers = servers.clone();

            move || {
                let folder = folder.read().cloned();
                let mut mention_count = 0;
                let mut has_unread = false;
                let servers = servers.read();

                for server_id in &folder.servers {
                    if !is_server_muted(server_id, &notifications_settings)
                        && let Some(server) = servers.get(server_id)
                    {
                        let badges = server
                            .channels
                            .iter()
                            .filter_map(|id| {
                                if let Some(unread) = unreads.read().get(id) {
                                    Some((id.clone(), unread.clone()))
                                } else {
                                    None
                                }
                            })
                            .filter_map(|(id, unread)| {
                                channels
                                    .read()
                                    .get(&id)
                                    .and_then(|channel| get_unread_badge(channel, &unread))
                            })
                            .collect::<Vec<_>>();

                        for badge in badges {
                            match badge {
                                NotificationBadge::Unread => has_unread = true,
                                NotificationBadge::Mentions(count) => mention_count += count,
                            }
                        }
                    }
                }

                if mention_count > 0 {
                    Some(NotificationBadge::Mentions(mention_count))
                } else if has_unread {
                    Some(NotificationBadge::Unread)
                } else {
                    None
                }
            }
        });

        let indicator_height = if let Selection::Server(id) = &*selection.read()
            && folder.servers.contains(id)
        {
            32.
        } else if hovering() {
            16.
        } else if badge.read().is_some() {
            8.
        } else {
            0.
        };

        let indicator_anim = use_animation_transition(indicator_height, move |before, after| {
            AnimNum::new(before, after).time(100).function(fast_func())
        });

        rect()
            .center()
            .width(Size::px(56.))
            .corner_radius(if folder.collapsed { 28. } else { 0. })
            .height(if folder.collapsed {
                Size::px(56.)
            } else {
                Size::Inner
            })
            .border(drag_hovering().then(|| Border::new().fill(theme.md.primary).width(2.)))
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .on_secondary_down({
                let id = self.id.clone();
                move |_| {
                    ContextMenu::open_from_down(Menu::new().child(ServerFolderMenu {
                        folder_id: id.clone(),
                    }));
                }
            })
            .maybe_child(folder.collapsed.then(|| {
                rect()
                    .center()
                    .position(Position::new_absolute().left(0.))
                    .height(Size::px(56.))
                    .child(
                        rect()
                            .width(Size::px(4.))
                            .height(Size::px(indicator_anim.read().value()))
                            .corner_radius(CornerRadius::new(0., 4., 4., 0.))
                            .background(theme.md.on_surface.as_argb_u32()),
                    )
            }))
            .child(
                DragZone::new(Entry::Folder {
                    id: self.id.clone(),
                })
                .drag_element(DragFolder {
                    id: self.id.clone(),
                    dragging: self.dragging,
                })
                .child(if folder.collapsed {
                    StoatTooltip::new(stoat_label().max_lines(1).text(folder.name.clone()))
                        .position(AttachedPosition::Right)
                        .child(
                            rect()
                                .size(Size::px(42.))
                                .child(FolderPreview {
                                    id: self.id.clone(),
                                    opacity,
                                    dragging: self.dragging,
                                })
                                .maybe_child(badge().and_then(|badge| badge.mention_count()).map(
                                    |count| {
                                        rect()
                                            .position(Position::new_absolute().right(0.).top(0.))
                                            .layer(Layer::Relative(10))
                                            .width(Size::px(13.))
                                            .height(Size::px(13.))
                                            .corner_radius(13.)
                                            .center()
                                            .background(theme.md.error.as_argb_u32())
                                            .color(theme.md.on_error.as_argb_u32())
                                            .font_size(8.)
                                            .font_weight(FontWeight::SEMI_BOLD)
                                            .child(if count <= 9 {
                                                count.to_string()
                                            } else {
                                                "+".to_string()
                                            })
                                    },
                                )),
                        )
                        .into_element()
                } else {
                    rect()
                        .width(Size::px(56.))
                        .corner_radius(16.)
                        .background(
                            folder
                                .colour
                                .as_deref()
                                .and_then(parse_fill)
                                .and_then(|f| f.as_color())
                                .map(|c| c.with_af32(0.18))
                                .unwrap_or_else(|| Color::from(theme.md.on_surface).with_af32(0.1)),
                        )
                        .child(
                            StoatTooltip::new(label().max_lines(1).text(folder.name.clone()))
                                .position(AttachedPosition::Right)
                                .child(
                                    rect()
                                        .size(Size::px(56.))
                                        .center()
                                        .on_press({
                                            let mut folders = folders.clone();
                                            let id = self.id.clone();
                                            move |_| {
                                                let mut folders = folders.write();
                                                let folder = folders
                                                    .iter_mut()
                                                    .find(|f| &f.id == &id)
                                                    .unwrap();
                                                folder.collapsed = true;
                                            }
                                        })
                                        .child(
                                            MaterialIcon::new(filled_folder())
                                                .size(Size::px(32.))
                                                .opacity(opacity)
                                                .color(theme.md.on_surface),
                                        ),
                                ),
                        )
                        .children(
                            (0..)
                                .into_iter()
                                .map(|idx| {
                                    InsertArea {
                                        idx,
                                        folder: Some(self.id.clone()),
                                        dragging: self.dragging,
                                        entries: self.entries,
                                    }
                                    .into_element()
                                })
                                .zip(
                                    folder
                                        .servers
                                        .iter()
                                        .filter_map(|id| servers.read().get(id).cloned())
                                        .map(|server| {
                                            ServerEntry {
                                                server,
                                                folder: Some(self.id.clone()),
                                                dragging: self.dragging,
                                                entries: self.entries,
                                            }
                                            .into_element()
                                        }),
                                )
                                .map(|(a, b)| [a, b])
                                .flatten(),
                        )
                        .into_element()
                }),
            )
            .maybe_child(
                (folder.collapsed
                    && self
                        .dragging
                        .read()
                        .as_ref()
                        .is_some_and(|entry| entry.id() != &self.id && !entry.is_folder()))
                .then(|| {
                    rect()
                        .position(Position::new_absolute().top(18.))
                        .layer(Layer::Overlay)
                        .child(
                            DropZone::<Entry>::new({
                                let mut folders = folders.clone();
                                let mut sidebar = ordering.clone();
                                let folder_id = self.id.clone();

                                move |entry| {
                                    let Entry::Server {
                                        server,
                                        folder: entry_folder,
                                    } = entry
                                    else {
                                        return;
                                    };

                                    if let Some(entry_folder) = entry_folder {
                                        folders
                                            .write()
                                            .iter_mut()
                                            .find(|f| &f.id == &entry_folder)
                                            .unwrap()
                                            .servers
                                            .retain(|s| s != &server.id);
                                    } else {
                                        sidebar.write().retain(|e| e != &server.id);
                                    }

                                    if let Some(folder) =
                                        folders.write().iter_mut().find(|f| &f.id == &folder_id)
                                    {
                                        folder.servers.push(server.id);
                                    }
                                }
                            })
                            .on_drag_over({
                                move |is_in| {
                                    drag_hovering.set(is_in);
                                }
                            })
                            .child(rect().width(Size::px(56.)).height(Size::px(20.))),
                        )
                }),
            )
    }
}

#[derive(PartialEq)]
pub struct FolderPreview {
    pub id: String,

    pub opacity: f32,

    pub dragging: State<Option<Entry>>,
}

impl Component for FolderPreview {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let radio = use_radio(AppChannel::Servers);

        let folders = radio.slice_mut(AppChannel::Settings("server-folders"), |state| {
            &mut state
                .settings
                .server_folders
                .get_or_insert_default()
                .folders
        });

        let folder = folders
            .read()
            .iter()
            .find(|f| &f.id == &self.id)
            .unwrap()
            .clone();

        let first = folder.servers.iter().take(2).cloned().collect::<Vec<_>>();
        let second = folder
            .servers
            .iter()
            .skip(2)
            .take(2)
            .cloned()
            .collect::<Vec<_>>();

        let state = radio.read();

        rect()
            .size(Size::px(42.))
            .corner_radius(21.)
            .background(
                folder
                    .colour
                    .as_deref()
                    .and_then(parse_fill)
                    .and_then(|f| f.as_color())
                    .map(|c| c.with_af32(0.3))
                    .unwrap_or_else(|| theme.md.surface_container_low.into()),
            )
            .opacity(self.opacity)
            .center()
            .overflow(Overflow::Clip)
            .on_press({
                let dragging = self.dragging;
                let mut folders = folders.clone();
                let id = self.id.clone();
                move |_| {
                    if dragging.read().is_some() {
                        return;
                    };

                    let mut folders = folders.write();
                    let folder = folders.iter_mut().find(|f| &f.id == &id).unwrap();
                    folder.collapsed = false;
                }
            })
            .spacing(2.)
            .child(
                rect()
                    .maybe(!second.is_empty(), |this| this.horizontal())
                    .spacing(2.)
                    .children(first.into_iter().map(|id| {
                        let server = state.servers.get(&id).unwrap();

                        ServerIcon::from_values(
                            server.icon.clone(),
                            server.name.clone(),
                            if second.is_empty() { 21. } else { 17. },
                        )
                        .corner_radius(11.)
                    })),
            )
            .maybe_child((!second.is_empty()).then(|| {
                rect()
                    .horizontal()
                    .spacing(2.)
                    .children(second.into_iter().map(|id| {
                        let server = state.servers.get(&id).unwrap();

                        ServerIcon::from_values(server.icon.clone(), server.name.clone(), 17.)
                            .corner_radius(9.)
                    }))
            }))
    }
}

#[derive(PartialEq)]
pub struct DragFolder {
    pub id: String,

    pub dragging: State<Option<Entry>>,
}

impl Component for DragFolder {
    fn render(&self) -> impl IntoElement {
        let mut dragging = self.dragging;

        use_hook(|| {
            dragging.set(Some(Entry::Folder {
                id: self.id.clone(),
            }));
        });
        use_drop(move || dragging.set(None));

        FolderPreview {
            id: self.id.clone(),
            opacity: 0.85,
            dragging: self.dragging,
        }
    }
}
