use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{StoatButton, StoatButtonLayoutThemePartialExt, server_icon},
    get_unicode_emojis, http,
    types::Tag,
    use_material_theme,
};

#[derive(PartialEq, Clone, Debug)]
enum Item {
    Server(String),
    Spacer,
    Emoji(v0::Emoji),
    Title(String),
    Unicode { name: String, value: String },
}

#[derive(PartialEq)]
pub struct EmojiPicker {
    on_select: EventHandler<String>,
}

impl EmojiPicker {
    pub fn new(on_select: impl Into<EventHandler<String>>) -> Self {
        Self {
            on_select: on_select.into(),
        }
    }
}

impl Component for EmojiPicker {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Emojis);
        let servers = radio.slice(AppChannel::Servers, |state| &state.servers);
        let emojis = radio.slice_current(|state| &state.emojis);
        let theme = use_material_theme();

        let filter = use_state(String::new);

        let items = use_memo({
            let servers = servers.clone();

            move || {
                let filter = filter.read().to_lowercase();
                let emojis = emojis.read();

                if !filter.is_empty() {
                    let mut items = emojis
                        .values()
                        .filter(|emoji| emoji.name.to_lowercase().contains(&filter))
                        .map(|emoji| Item::Emoji(emoji.clone()))
                        .chain(
                            get_unicode_emojis()
                                .iter()
                                .filter(|(name, _)| name.to_lowercase().contains(&filter))
                                .map(|(name, value)| Item::Unicode {
                                    name: name.clone(),
                                    value: value.clone(),
                                }),
                        )
                        .collect::<Vec<_>>();

                    while (items.len() % 10) != 0 {
                        items.push(Item::Spacer);
                    }

                    items
                } else {
                    let mut items = Vec::new();

                    for server in servers.read().values() {
                        let mut server_emojis = emojis
                            .values()
                            .filter(|emoji| {
                                if let v0::EmojiParent::Server { id } = &emoji.parent
                                    && id == &server.id
                                {
                                    true
                                } else {
                                    false
                                }
                            })
                            .peekable();

                        if server_emojis.peek().is_none() {
                            continue;
                        };

                        items.push(Item::Server(server.id.clone()));

                        while (items.len() % 10) != 0 {
                            items.push(Item::Spacer);
                        }

                        for emoji in server_emojis {
                            items.push(Item::Emoji(emoji.clone()))
                        }

                        while (items.len() % 10) != 0 {
                            items.push(Item::Spacer);
                        }
                    }

                    items.push(Item::Title("Default".to_string()));

                    while (items.len() % 10) != 0 {
                        items.push(Item::Spacer);
                    }

                    for (name, value) in get_unicode_emojis().iter() {
                        items.push(Item::Unicode {
                            name: name.clone(),
                            value: value.clone(),
                        });
                    }

                    while (items.len() % 10) != 0 {
                        items.push(Item::Spacer);
                    }

                    items
                }
            }
        });

        rect()
        .width(Size::px(400.))
        .padding((8., 0.))
        .corner_radius(16.)
            .background(theme.md.surface_container.as_argb_u32())
            .child(rect().height(Size::px(40.)).margin((0., 0., 8., 0.)))
            .child(    rect()
        .corner_radius(CornerRadius {
            top_left: 4.,
            top_right: 4.,
            bottom_right: 0.,
            bottom_left: 0.,
            smoothing: 0.,
        })
        .border(
            Border::new()
                .width(BorderWidth {
                    top: 0.,
                    right: 0.,
                    bottom: 1.,
                    left: 0.,
                })
                .fill(theme.md.on_surface_variant.as_argb_u32())
                .alignment(BorderAlignment::Inner),
        )
        .background(theme.md.surface_container_highest.as_argb_u32())
        .padding((10., 8.))
        .center()
        .child(
            Input::new(filter)
                .color(theme.md.on_surface.as_argb_u32())
                .placeholder_color(theme.md.on_surface_variant.as_argb_u32())
                .placeholder("Search for emojis...")
                .width(Size::Fill)
                .flat()
                .background(Color::TRANSPARENT)
                .focus_background(Color::TRANSPARENT)
                .focus_border_fill(Color::TRANSPARENT),
        ))
            .child(
                VirtualScrollView::new({let
                    on_select = self.on_select.clone(); move |row_i, _| {
                    let row_items = &items.read()[row_i * 10..row_i * 10 + 10];

                    rect()
                        .horizontal()
                        .children(row_items.iter().map(|item| {
                            rect().width(Size::px(40.)).height(Size::px(40.)).center().child(
                        match item {
                            Item::Server(id) => {
                                let servers = servers.read();
                                let server = servers.get(id).unwrap();

                                    rect()
                                        .layer(Layer::Overlay)
                                        .position(Position::new_absolute().left(8.))
                                        .height(Size::px(40.))
                                        .horizontal()
                                        .cross_align(Alignment::Center)
                                        .spacing(8.)
                                        .child(rect().width(Size::px(24.)).height(Size::px(24.)).corner_radius(24.).overflow(Overflow::Clip).child(server_icon(server, &theme)))
                                        .child(label().text(server.name.clone()).max_lines(1))
                                        .into_element()
                            }
                            Item::Spacer => rect().into_element(),
                            Item::Emoji(emoji) =>
                            StoatButton::new().corner_radius(8.).child(rect().padding(4.).child(
                                ImageViewer::new(
                                    format!(
                                        "{}/{}/{}",
                                        http().api_config.features.autumn.url,
                                        Tag::Emojis,
                                        &emoji.id,
                                    )
                                    .parse::<Uri>()
                                    .unwrap(),
                                )
                                .sampling_mode(SamplingMode::Trilinear)
                                .error_renderer(move |_| rect().into_element())
                                .width(Size::px(32.))
                                .height(Size::px(32.)

                            )).overflow(Overflow::Clip))
                            .on_press({let id = emoji.id.clone(); let on_select = on_select.clone(); move |_| on_select.call(id.clone())})
                                .into_element()
                            ,
                            Item::Title(title) => {
                                    rect()
                                    .layer(Layer::Overlay)
                                        .position(Position::new_absolute().left(8.))
                                        .height(Size::px(40.))
                                        .horizontal()
                                        .cross_align(Alignment::Center)
                                        .spacing(8.)
                                        .child(label().text(title.clone()).max_lines(1))
                                        .into_element()
                            }
                            Item::Unicode { name, value } => {
                                let codes = value
                                    .chars()
                                    .map(|c| format!("{:x}", c as i32))
                                    .collect::<Vec<String>>()
                                    .join("-");

                                let url = format!(
                                    "https://static.stoat.chat/emoji/fluent-3d/{codes}.svg?v=1"
                                );

                                StoatButton::new().corner_radius(8.).child(rect().padding(4.).child(
                                    SvgViewer::new(url.parse::<Uri>().unwrap())
                                        .width(Size::px(32.))
                                        .height(Size::px(32.))
                                ))
                                .on_press({let value = value.clone(); let on_select = on_select.clone(); move |_| on_select.call(value.clone())})
                                        .into_element()
                            }
                        }
                    )
                        .into_element()
                        }))
                        .into_element()
                }})
                .item_size(40.)
                .length(items.read().len() / 10)
                .width(Size::Fill)
                .height(Size::px(280.)),
            )
    }
}
