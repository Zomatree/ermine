use std::iter::once;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;
use ulid::Ulid;

use crate::{
    AppChannel, SizeExt,
    components::{
        MaterialIcon, ModalValue, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt,
        material::{
            filled::clear,
            outlined::{add, drag_indicator, grid_3x3, headset_mic},
        },
        use_modals,
    },
    consume_material_theme, http, use_changed_from_readable,
};

#[derive(PartialEq)]
pub struct CategoryServerSettings {
    pub server: Readable<v0::Server>,
}

impl Component for CategoryServerSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let categories = use_side_effect_value({
            let server = self.server.clone();
            move || {
                let server = server.read();
                let mut categories = server.categories.clone().unwrap_or_default();

                let category_channels = categories
                    .iter()
                    .flat_map(|cat| cat.channels.clone())
                    .collect::<Vec<_>>();

                let non_category_channels = server
                    .channels
                    .iter()
                    .filter(|&channel_id| !category_channels.contains(channel_id))
                    .cloned()
                    .collect::<Vec<_>>();

                if let Some(default_cat) = categories.iter_mut().find(|c| c.id == "default") {
                    default_cat.channels.extend(non_category_channels);
                } else {
                    categories.insert(
                        0,
                        v0::Category {
                            id: "default".to_string(),
                            title: "Default".to_string(),
                            channels: non_category_channels,
                        },
                    );
                };
                categories
            }
        });

        rect()
            .spacing(8.)
            .child(
                StoatButton::new()
                    .corner_radius(20.)
                    .background(theme.md.primary.as_argb_u32())
                    .color(theme.md.on_primary.as_argb_u32())
                    .on_press({
                        let server = self.server.clone();
                        move |_| {
                            let id = server.read().id.clone();
                            let categories = categories.read().clone();

                            spawn(async move {
                                http()
                                    .edit_server(
                                        &id,
                                        &v0::DataEditServer {
                                            name: None,
                                            description: None,
                                            icon: None,
                                            banner: None,
                                            categories: Some(categories),
                                            system_messages: None,
                                            flags: None,
                                            discoverable: None,
                                            analytics: None,
                                            owner: None,
                                            remove: Vec::new(),
                                        },
                                    )
                                    .await
                                    .unwrap();
                            });
                        }
                    })
                    .child(
                        rect()
                            .height(Size::px(40.))
                            .padding((0., 16.))
                            .main_align(Alignment::Center)
                            .child("Save"),
                    ),
            )
            .child(
                ScrollView::new()
                    .direction(Direction::Horizontal)
                    .child(ReorderCategories {
                        server: self.server.clone(),
                        categories,
                    }),
            )
    }
}

#[derive(PartialEq)]
struct DragChannel {
    pub id: String,
    pub channel: Option<v0::Channel>,
}

impl Component for DragChannel {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let name = self
            .channel
            .as_ref()
            .and_then(|c| c.name())
            .unwrap_or("Unknown Channel")
            .to_string();

        rect()
            .background(theme.md.surface_container_highest.as_argb_u32())
            .padding(8.)
            .spacing(8.)
            .height(Size::px(42.))
            .width(Size::px(180.))
            .corner_radius(28.)
            .horizontal()
            .cross_align(Alignment::Center)
            .font_size(15.)
            .cursor(CursorIcon::Pointer)
            .child(
                MaterialIcon::new(
                    if matches!(
                        &self.channel,
                        Some(v0::Channel::TextChannel { voice: Some(_), .. })
                    ) {
                        headset_mic()
                    } else {
                        grid_3x3()
                    },
                )
                .size(Size::px(24.)),
            )
            .child(name)
    }
}

#[derive(PartialEq)]
struct DragCategory {
    pub values: State<Vec<v0::Category>>,
    pub state: State<Vec<v0::Category>>,
    pub pos: usize,
    pub category: v0::Category,
    pub server: Readable<v0::Server>,

    pub selected_chan: State<Option<(String, Option<v0::Channel>)>>,
    pub hovered_chan: State<Option<(usize, usize, (String, Option<v0::Channel>))>>,
    pub size_chan: State<Size2D>,
}

impl Component for DragCategory {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();
        let radio = use_radio(AppChannel::Channels);
        let channels = radio.slice_current(|state| &state.channels);

        let channels = channels.read();

        let id = self.category.id.clone();
        let new_name = self.state.into_writable().map(
            {
                let id = id.clone();
                move |cats| &cats.iter().find(|c| &c.id == &id).unwrap().title
            },
            {
                let id = id.clone();
                move |cats| &mut cats.iter_mut().find(|c| &c.id == &id).unwrap().title
            },
        );

        use_changed_from_readable::<String>(new_name.clone().into(), {
            let mut values = self.values;
            move |name| {
                values
                    .write()
                    .iter_mut()
                    .find(|c| &c.id == &id)
                    .unwrap()
                    .title = name.clone();
            }
        });

        rect()
            .corner_radius(12.)
            .overflow(Overflow::Clip)
            .child(
                rect()
                    .width(Size::px(188.))
                    .height(Size::px(42.))
                    .padding(8.)
                    .background(theme.md.surface_container.as_argb_u32())
                    .font_size(15.)
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .content(Content::Flex)
                    .spacing(8.)
                    .maybe(self.pos != 0, |this| {
                        this.cursor(CursorIcon::Pointer).child(
                            MaterialIcon::new(drag_indicator())
                                .size(Size::px(16.))
                                .color(theme.md.on_surface_variant.as_argb_u32()),
                        )
                    })
                    .child(if self.pos == 0 {
                        label()
                            .width(Size::flex(1.0))
                            .margin((0., 32.))
                            .text(self.category.title.clone())
                            .into_element()
                    } else {
                        Input::new(new_name)
                            .multiline(false)
                            .height(Size::px(26.))
                            .width(Size::flex(1.0))
                            .border_fill(Color::TRANSPARENT)
                            .background(Color::TRANSPARENT)
                            .focus_background(
                                Color::from(theme.md.on_surface.as_argb_u32()).with_af32(0.08),
                            )
                            .focus_border_fill(Color::TRANSPARENT)
                            .into_element()
                    })
                    .maybe_child((self.pos != 0).then(|| {
                        StoatButton::new()
                            .corner_radius(12.)
                            .on_press({
                                let server = self.server.clone();
                                let values = self.values;
                                let id = self.category.id.clone();

                                move |_| {
                                    let server_id = server.read().id.clone();
                                    let mut categories = values.read().clone();
                                    let id = id.clone();
                                    categories.retain(|c| &c.id != &id);

                                    spawn(async move {
                                        http()
                                            .edit_server(
                                                &server_id,
                                                &v0::DataEditServer {
                                                    name: None,
                                                    description: None,
                                                    icon: None,
                                                    banner: None,
                                                    categories: Some(categories),
                                                    system_messages: None,
                                                    flags: None,
                                                    discoverable: None,
                                                    analytics: None,
                                                    owner: None,
                                                    remove: Vec::new(),
                                                },
                                            )
                                            .await
                                            .unwrap();
                                    });
                                }
                            })
                            .child(
                                rect().size(Size::px(24.)).center().child(
                                    MaterialIcon::new(clear())
                                        .size(Size::px(16.))
                                        .color(theme.md.on_surface_variant.as_argb_u32()),
                                ),
                            )
                    })),
            )
            .child(
                rect()
                    .padding(4.)
                    .background(theme.md.surface_container_high.as_argb_u32())
                    .spacing(4.)
                    .children(
                        self.category
                            .channels
                            .clone()
                            .into_iter()
                            .chain(once("".to_string()))
                            .enumerate()
                            .map(|(i, id)| {
                                let channel = channels.get(&id).cloned();

                                rect()
                                    .on_pointer_down(|e: Event<PointerEventData>| {
                                        e.stop_propagation()
                                    })
                                    .child(
                                        DropZone::<String>::new(|_| {})
                                            .child(
                                                DragZone::new(id.clone())
                                                    .enabled(i != self.category.channels.len())
                                                    .child(if i == self.category.channels.len() {
                                                        rect().interactive(self.selected_chan.read().is_none()).child(
                                                        StoatButton::new()
                                                            .on_press({
                                                                let category_id = self.category.id.clone();
                                                                let server = self.server.clone();
                                                                let values = self.values;

                                                                move |_| {
                                                                    let server_id = server.read().id.clone();
                                                                    let values = values.read().cloned();
                                                                    let category_id = category_id.clone();

                                                                    modals.write().push_modal(ModalValue::CreateChannel {
                                                                        server: server_id.clone(),
                                                                        callback: EventHandler::new(
                                                                            move |channel: v0::Channel| {
                                                                                let server_id = server_id.clone();
                                                                                let mut categories = values.clone();

                                                                                categories
                                                                                    .iter_mut()
                                                                                    .find(|c| &c.id == &category_id)
                                                                                    .unwrap()
                                                                                    .channels
                                                                                    .push(channel.id().to_string());

                                                                                spawn(async move {
                                                                                    http().edit_server(&server_id, &v0::DataEditServer {
                                                                                        name: None,
                                                                                        description: None,
                                                                                        icon: None,
                                                                                        banner: None,
                                                                                        categories: Some(categories),
                                                                                        system_messages: None,
                                                                                        flags: None,
                                                                                        discoverable: None,
                                                                                        analytics: None,
                                                                                        owner: None,
                                                                                        remove: Vec::new(),
                                                                                    }).await.unwrap();
                                                                                });
                                                                            }
                                                                        )
                                                                    });
                                                                }
                                                            })
                                                            .corner_radius(21.)
                                                            .child(
                                                                rect()
                                                                    .height(Size::px(42.))
                                                                    .width(Size::px(180.))
                                                                    .center()
                                                                    .child(MaterialIcon::new(add()).size(Size::px(24.)))
                                                            ))
                                                            .into_element()
                                                    } else if self
                                                        .hovered_chan
                                                        .read()
                                                        .as_ref()
                                                        .is_none_or(|(_, _, (v, _))| {
                                                            v != &id
                                                                || self
                                                                    .selected_chan
                                                                    .read()
                                                                    .as_ref()
                                                                    .is_none_or(|(id, _)| id != v)
                                                        })
                                                    {
                                                        rect()
                                                            .interactive(
                                                                self.selected_chan.read().is_none(),
                                                            )
                                                            .child(DragChannel {
                                                                id: id.clone(),
                                                                channel: channel.clone(),
                                                            })
                                                            .into_element()
                                                    } else {
                                                        rect()
                                                            .width(Size::px(
                                                                self.size_chan.read().width,
                                                            ))
                                                            .height(Size::px(
                                                                self.size_chan.read().height,
                                                            ))
                                                            .into_element()
                                                    })
                                                    .drag_element(
                                                        rect()
                                                            .on_sized({
                                                                let id = id.clone();
                                                                let channel = channel.clone();
                                                                let mut selected_chan =
                                                                    self.selected_chan;
                                                                let mut hovered_chan =
                                                                    self.hovered_chan;
                                                                let pos = self.pos;

                                                                move |_| {
                                                                    let is_dragging = selected_chan
                                                                        .read()
                                                                        .is_some();

                                                                    if !is_dragging {
                                                                        selected_chan.set(Some((
                                                                            id.clone(),
                                                                            channel.clone(),
                                                                        )));
                                                                        hovered_chan.set(Some((
                                                                            pos,
                                                                            i,
                                                                            (
                                                                                id.clone(),
                                                                                channel.clone(),
                                                                            ),
                                                                        )));
                                                                    }
                                                                }
                                                            })
                                                            .child(DraggedChannel {
                                                                values: self.values,
                                                                state: self.state,
                                                                id: id.clone(),
                                                                channel: channel.clone(),
                                                                size: self.size_chan,
                                                                selected: self.selected_chan,
                                                            }),
                                                    )
                                                    .show_while_dragging(true)
                                                    .into_element(),
                                            )
                                            .on_drag_over({
                                                let mut state = self.state;
                                                let pos = self.pos;
                                                let mut hovered_chan = self.hovered_chan;
                                                let selected_chan = self.selected_chan;

                                                move |hovering: bool| {
                                                    if !hovering {
                                                        return;
                                                    };

                                                    let current = hovered_chan.read().cloned();
                                                    let selected = selected_chan.read().cloned();

                                                    if let Some(current) = current
                                                        && let Some(selected) = selected
                                                    {
                                                        let mut s = state.write();
                                                        let from_len = s[current.0].channels.len();
                                                        let to_len = s[pos].channels.len();

                                                        if current.1 == from_len {
                                                            s[current.0].channels.pop();
                                                        } else {
                                                            s[current.0].channels.remove(current.1);
                                                        }

                                                        if to_len == i || to_len == 0 {
                                                            s[pos]
                                                                .channels
                                                                .push(current.2.0.clone());
                                                            hovered_chan
                                                                .set(Some((pos, i, selected)));
                                                        } else {
                                                            s[pos]
                                                                .channels
                                                                .insert(i, current.2.0.clone());
                                                            hovered_chan
                                                                .set(Some((pos, i, selected)));
                                                        }
                                                    }
                                                }
                                            }),
                                    )
                                    .into_element()
                            }),
                    ),
            )
            .into_element()
    }

    fn render_key(&self) -> DiffKey {
        (&self.category.id).into()
    }
}

#[derive(PartialEq)]
struct ReorderCategories {
    pub server: Readable<v0::Server>,
    pub categories: State<Vec<v0::Category>>,
}

impl Component for ReorderCategories {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut state = use_state(|| self.categories.read().cloned());

        use_side_effect({
            let values = self.categories;
            move || state.set_if_modified(values.read().cloned())
        });

        let mut selected_cat = use_state(|| None);
        let mut hovered_cat = use_state(|| None);

        let size_cat = use_state(Size2D::default);

        let selected_chan = use_state(|| None);
        let hovered_chan = use_state(|| None);

        let size_chan = use_state(Size2D::default);

        rect()
            .horizontal()
            .spacing(8.)
            .children(
                state
                    .read()
                    .cloned()
                    .into_iter()
                    .enumerate()
                    .map(|(i, category)| {
                        rect()
                            .key(&category.id)
                            .child(
                                DropZone::<v0::Category>::new(|_| {})
                                    .child(
                                        DragZone::new(category.clone())
                                            .enabled(i != 0)
                                            .child(
                                                if hovered_cat.read().as_ref().is_none_or(
                                                    |(_, v)| {
                                                        v != &category
                                                            || selected_cat.read().as_ref()
                                                                != Some(v)
                                                    },
                                                ) {
                                                    rect()
                                                        .interactive(selected_cat.read().is_none())
                                                        .child(DragCategory {
                                                            pos: i,
                                                            values: self.categories,
                                                            state: state,
                                                            category: category.clone(),
                                                            server: self.server.clone(),
                                                            selected_chan,
                                                            hovered_chan,
                                                            size_chan,
                                                        })
                                                        .into_element()
                                                } else {
                                                    rect()
                                                        .width(Size::px(size_cat.read().width))
                                                        .height(Size::px(size_cat.read().height))
                                                        .into_element()
                                                },
                                            )
                                            .drag_element(
                                                rect()
                                                    .on_press(|e: Event<PressEventData>| {
                                                        e.stop_propagation()
                                                    })
                                                    .on_sized({
                                                        let category = category.clone();

                                                        move |_| {
                                                            let is_dragging =
                                                                selected_cat.read().is_some();

                                                            if !is_dragging {
                                                                selected_cat
                                                                    .set(Some(category.clone()));
                                                                hovered_cat.set(Some((
                                                                    i,
                                                                    category.clone(),
                                                                )));
                                                            }
                                                        }
                                                    })
                                                    .child(DraggedCategory {
                                                        pos: i,
                                                        values: self.categories.clone(),
                                                        state,
                                                        category,
                                                        server: self.server.clone(),
                                                        size: size_cat,
                                                        selected: selected_cat,
                                                        selected_chan,
                                                        hovered_chan,
                                                        size_chan,
                                                    }),
                                            )
                                            .show_while_dragging(true)
                                            .into_element(),
                                    )
                                    .on_drag_over(move |hovering: bool| {
                                        if !hovering || i == 0 {
                                            return;
                                        };

                                        let current = hovered_cat.read().cloned();
                                        let selected = selected_cat.read().cloned();

                                        if let Some(current) = current
                                            && let Some(selected) = selected
                                        {
                                            state.write().remove(current.0);
                                            state.write().insert(i, current.1.clone());
                                            hovered_cat.set(Some((i, selected)));
                                        }
                                    }),
                            )
                            .into_element()
                    }),
            )
            .child(
                StoatButton::new()
                    .corner_radius(12.)
                    .background(theme.md.surface_container.as_argb_u32())
                    .on_press({
                        let server = self.server.clone();
                        move |_| {
                            let id = server.read().id.clone();

                            let mut categories = state.read().cloned();

                            categories.push(v0::Category {
                                id: Ulid::new().to_string(),
                                title: "New Category".to_string(),
                                channels: Vec::new(),
                            });

                            spawn(async move {
                                http()
                                    .edit_server(
                                        &id,
                                        &v0::DataEditServer {
                                            name: None,
                                            description: None,
                                            icon: None,
                                            banner: None,
                                            categories: Some(categories),
                                            system_messages: None,
                                            flags: None,
                                            discoverable: None,
                                            analytics: None,
                                            owner: None,
                                            remove: Vec::new(),
                                        },
                                    )
                                    .await
                                    .unwrap();
                            });
                        }
                    })
                    .child(
                        rect()
                            .width(Size::px(188.))
                            .height(Size::px(42.))
                            .padding(8.)
                            .center()
                            .font_size(15.)
                            .child("New Category"),
                    ),
            )
    }
}

#[derive(PartialEq)]
struct DraggedCategory {
    pub pos: usize,
    pub values: State<Vec<v0::Category>>,
    pub state: State<Vec<v0::Category>>,
    pub category: v0::Category,
    pub server: Readable<v0::Server>,
    pub size: State<Size2D>,
    pub selected: State<Option<v0::Category>>,

    pub selected_chan: State<Option<(String, Option<v0::Channel>)>>,
    pub hovered_chan: State<Option<(usize, usize, (String, Option<v0::Channel>))>>,
    pub size_chan: State<Size2D>,
}

impl Component for DraggedCategory {
    fn render(&self) -> impl IntoElement {
        let mut values = self.values.clone();
        let state = self.state;
        let mut size = self.size;
        let mut selected = self.selected;

        use_drop(move || {
            selected.set(None);
            values.set(state.read().cloned());
        });

        rect()
            .cursor(CursorIcon::Pointer)
            .on_sized(move |area: Event<SizedEventData>| size.set_if_modified(area.area.size))
            .child(DragCategory {
                pos: self.pos,
                values: self.values,
                state: self.state,
                category: self
                    .selected
                    .read()
                    .as_ref()
                    .unwrap_or(&self.category)
                    .clone(),
                server: self.server.clone(),
                selected_chan: self.selected_chan,
                hovered_chan: self.hovered_chan,
                size_chan: self.size_chan,
            })
    }
}

#[derive(PartialEq)]
struct DraggedChannel {
    pub values: State<Vec<v0::Category>>,
    pub state: State<Vec<v0::Category>>,
    pub id: String,
    pub channel: Option<v0::Channel>,
    pub size: State<Size2D>,
    pub selected: State<Option<(String, Option<v0::Channel>)>>,
}

impl Component for DraggedChannel {
    fn render(&self) -> impl IntoElement {
        let mut values = self.values.clone();
        let state = self.state;
        let mut size = self.size;
        let mut selected = self.selected;

        use_drop(move || {
            selected.set(None);
            values.set(state.read().cloned());
        });

        let (id, channel) = self
            .selected
            .read()
            .cloned()
            .unwrap_or_else(|| (self.id.clone(), self.channel.clone()));

        rect()
            .cursor(CursorIcon::Pointer)
            .on_sized(move |area: Event<SizedEventData>| size.set_if_modified(area.area.size))
            .child(DragChannel { id, channel })
    }
}
