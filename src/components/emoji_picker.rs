use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, SizeExt,
    components::{AnimatedImage, ServerIcon, StoatButton, StoatButtonLayoutThemePartialExt},
    consume_material_theme, get_unicode_emojis, http,
    types::Tag,
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
        let theme = consume_material_theme();

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
            .spacing(8.)
            .child(
                rect()
                    .padding((0., 8.))
                    .child(
                        Input::new(filter)
                            .auto_focus(true)
                            .width(Size::Fill)
                            .placeholder("Search for emojis...")
                            .background(Color::TRANSPARENT)
                            .border_fill(theme.md.outline.as_argb_u32())
                            .focus_background(Color::TRANSPARENT)
                            .focus_border_fill(theme.md.primary.as_argb_u32())
                            .corner_radius(4.)
                            .inner_margin(16.)
                    )
            )
            .child(
                VirtualScrollView::new({
                    let on_select = self.on_select.clone();

                    move |item, _| {
                        let idx = item.index;

                        let row_items = &items.read()[idx * 10..idx * 10 + 10];

                        rect()
                            .horizontal()
                            .children(row_items.iter().map(|item| {
                                rect()
                                    .width(Size::px(40.))
                                    .height(Size::px(40.))
                                    .center()
                                    .child(
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
                                                    .child(rect().corner_radius(12.).overflow(Overflow::Clip).child(ServerIcon::new(server.clone(), 24.)))
                                                    .child(label().text(server.name.clone()).max_lines(1))
                                                    .into_element()
                                            }
                                            Item::Spacer => rect().into_element(),
                                            Item::Emoji(emoji) => StoatButton::new()
                                                .corner_radius(8.)
                                                .on_press({
                                                    let id = emoji.id.clone();
                                                    let on_select = on_select.clone();
                                                    move |_| on_select.call(id.clone())
                                                })
                                                .child(
                                                    rect()
                                                        .padding(4.)
                                                        .overflow(Overflow::Clip)
                                                        .child(
                                                            AnimatedImage::new(
                                                                format!(
                                                                    "{}/{}/{}",
                                                                    http().api_config.features.autumn.url,
                                                                    Tag::Emojis,
                                                                    &emoji.id,
                                                                )
                                                                .parse::<Url>()
                                                                .unwrap(),
                                                            )
                                                            .sampling_mode(SamplingMode::Trilinear)
                                                            .width(Size::px(32.))
                                                            .height(Size::px(32.))
                                                        )
                                                )
                                                .into_element(),
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
                                            Item::Unicode { name: _, value } => {
                                                let codes = value
                                                    .chars()
                                                    .map(|c| format!("{:x}", c as i32))
                                                    .collect::<Vec<String>>()
                                                    .join("-");

                                                let url = format!(
                                                    "https://static.stoat.chat/emoji/fluent-3d/{codes}.svg?v=1"
                                                );

                                                StoatButton::new()
                                                    .corner_radius(8.)
                                                    .on_press({
                                                        let value = value.clone();
                                                        let on_select = on_select.clone();

                                                        move |_| on_select.call(value.clone())
                                                    })
                                                    .child(
                                                        rect()
                                                            .padding(4.)
                                                            .child(
                                                                SvgViewer::new(url.parse::<Url>().unwrap())
                                                                    .parallel(true)
                                                                    .size(Size::px(32.))
                                                            )
                                                    )
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
