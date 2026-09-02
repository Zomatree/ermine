use std::time::Duration;

use freya::prelude::*;
use futures::FutureExt;
use itertools::Itertools;
use tokio::time::sleep;

use crate::{
    CategoriesQueryParams, SearchQueryParams, SizeExt, TrendingQueryParams,
    components::{
        AnimatedImage, MaterialIcon, StoatButton, StoatButtonLayoutThemePartialExt,
        material::outlined::arrow_back,
    },
    consume_material_theme, http, proxy_url,
};

const LOCALE: &str = "en_US";

#[derive(PartialEq)]
pub struct GifPicker {
    on_select: EventHandler<String>,
}

impl GifPicker {
    pub fn new(on_select: impl Into<EventHandler<String>>) -> Self {
        Self {
            on_select: on_select.into(),
        }
    }
}

impl Component for GifPicker {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let mut categories =
            use_provide_root_context(|| State::create_in_scope(None, ScopeId::ROOT));
        let mut trending_placeholder =
            use_provide_root_context(|| State::create_in_scope(None, ScopeId::ROOT));

        let mut query = use_state(String::new);
        let mut on_trending = use_state(|| false);

        let mut results = use_state(Vec::new);
        let mut next = use_state(|| None);

        use_hook(|| {
            if categories.read().is_some() && trending_placeholder.read().is_some() {
                return;
            };

            spawn(async move {
                let http = http();

                let category_query = CategoriesQueryParams {
                    locale: LOCALE.to_string(),
                };

                let trending_query = TrendingQueryParams {
                    locale: LOCALE.to_string(),
                    limit: Some(1),
                    position: None,
                };

                let a = http.fetch_gif_categories(&category_query).boxed_local();
                let b = http.fetch_gif_trending(&trending_query).boxed_local();

                if let Ok((cats, trending)) = tokio::try_join!(a, b) {
                    categories.set(Some(cats));

                    if let Some(first) = trending.results.into_iter().next() {
                        trending_placeholder.set(Some(first));
                    }
                };
            });
        });

        let mut query_task = use_state(|| None::<TaskHandle>);

        use_side_effect({
            move || {
                let query = query.read().cloned();

                query_task.take().map(|t| t.cancel());

                if query.is_empty() {
                    results.write().clear();
                    return;
                };

                on_trending.set_if_modified(false);

                query_task.set(Some(spawn(async move {
                    sleep(Duration::from_secs_f32(0.5)).await;

                    if let Ok(resp) = http()
                        .search_gifs(&SearchQueryParams {
                            query,
                            locale: LOCALE.to_string(),
                            limit: Some(50),
                            is_category: None,
                            position: None,
                        })
                        .await
                    {
                        results.set(resp.results);
                        next.set(resp.next);
                    }
                })));
            }
        });

        use_side_effect({
            move || {
                if on_trending() {
                    spawn(async move {
                        if let Ok(resp) = http()
                            .fetch_gif_trending(&TrendingQueryParams {
                                locale: LOCALE.to_string(),
                                limit: Some(50),
                                position: None,
                            })
                            .await
                        {
                            results.set(resp.results);
                            next.set(resp.next);
                        }
                    });
                };
            }
        });

        rect()
            .spacing(8.)
            .padding((0., 8.))
            .child(
                rect()
                    .horizontal()
                    .spacing(8.)
                    .cross_align(Alignment::Center)
                    .maybe_child((on_trending() || !query.read().is_empty()).then(|| {
                        StoatButton::new()
                            .corner_radius(20.)
                            .on_press(move |_| {
                                results.write().clear();
                                query.write().clear();
                                on_trending.set(false);
                            })
                            .child(
                                rect()
                                    .size(Size::px(40.))
                                    .center()
                                    .child(MaterialIcon::new(arrow_back()).size(Size::px(24.))),
                            )
                    }))
                    .child(
                        Input::new(query)
                            .auto_focus(true)
                            .width(Size::Fill)
                            .placeholder("Search for GIFs...")
                            .background(Color::TRANSPARENT)
                            .border_fill(theme.md.outline.as_argb_u32())
                            .focus_background(Color::TRANSPARENT)
                            .focus_border_fill(theme.md.primary.as_argb_u32())
                            .corner_radius(4.)
                            .inner_margin(16.),
                    ),
            )
            .child(ScrollView::new().height(Size::px(280.)).child({
                if !(on_trending() || !query.read().is_empty()) {
                    let trending = trending_placeholder.read().cloned();
                    let mut categories = categories.read().cloned().unwrap_or_default().into_iter();

                    let first = categories.next();

                    rect()
                        .spacing(4.)
                        .child(
                            rect()
                                .horizontal()
                                .spacing(4.)
                                .child(GifCategory {
                                    title: "Trending GIFs".to_string(),
                                    image: trending.map(|res| res.url.clone()),
                                    query,
                                    is_trending: true,
                                    on_trending,
                                })
                                .child(rect().width(Size::px(190.)).maybe_child(first.map(
                                    |category| {
                                        GifCategory {
                                            title: category.title,
                                            image: Some(category.image),
                                            query,
                                            is_trending: false,
                                            on_trending,
                                        }
                                        .into_element()
                                    },
                                ))),
                        )
                        .children(categories.chunks(2).into_iter().map(|mut chunk| {
                            let l = chunk.next().unwrap();
                            let r = chunk.next();

                            rect()
                                .horizontal()
                                .spacing(4.)
                                .child(
                                    GifCategory {
                                        title: l.title,
                                        image: Some(l.image),
                                        query,
                                        is_trending: false,
                                        on_trending,
                                    }
                                    .into_element(),
                                )
                                .child(rect().width(Size::px(190.)).maybe_child(r.map(
                                    |category| {
                                        GifCategory {
                                            title: category.title,
                                            image: Some(category.image),
                                            query,
                                            is_trending: false,
                                            on_trending,
                                        }
                                        .into_element()
                                    },
                                )))
                        }))
                } else {
                    let results = results.read();

                    rect()
                        .spacing(4.)
                        .horizontal()
                        .content(Content::wrap_spacing(4.))
                        .width(Size::px(384.))
                        .children(results.iter().map(|result| {
                            StoatButton::new()
                                .key(&result.id)
                                .corner_radius(12.)
                                .on_press({
                                    let on_select = self.on_select.clone();
                                    let url = result.url.clone();

                                    move |_| {
                                        on_select.call(url.clone());
                                    }
                                })
                                .child(
                                    AnimatedImage::new(proxy_url(&result.url))
                                        .aspect_ratio(AspectRatio::Min)
                                        .image_cover(ImageCover::Center)
                                        .width(Size::px(190.)),
                                )
                        }))
                }
            }))
    }
}

#[derive(PartialEq)]
struct GifCategory {
    pub title: String,
    pub image: Option<String>,
    pub query: State<String>,
    pub on_trending: State<bool>,
    pub is_trending: bool,
}

impl Component for GifCategory {
    fn render(&self) -> impl IntoElement {
        StoatButton::new()
            .corner_radius(12.)
            .on_press({
                let mut query = self.query;
                let mut on_trending = self.on_trending;
                let is_trending = self.is_trending;
                let title = self.title.clone();

                move |_| {
                    if is_trending {
                        query.write().clear();
                        on_trending.set(true);
                    } else {
                        query.set(title.clone());
                        on_trending.set(false);
                    };
                }
            })
            .child(
                rect()
                    .width(Size::px(190.))
                    .height(Size::px(118.75))
                    .overflow(Overflow::Clip)
                    .maybe_child(self.image.as_ref().map(|url| {
                        AnimatedImage::new(proxy_url(url))
                            .expanded()
                            .image_cover(ImageCover::Center)
                            .aspect_ratio(AspectRatio::Max)
                    }))
                    .child(
                        rect()
                            .width(Size::percent(100.))
                            .height(Size::percent(100.))
                            .layer(Layer::Relative(5))
                            .position(Position::new_absolute().top(0.).left(0.))
                            .padding(8.)
                            .main_align(Alignment::End)
                            .shadow(
                                Shadow::new()
                                    .inset()
                                    .y(100.)
                                    .x(0.)
                                    .blur(200.)
                                    .spread(100.)
                                    .color(0xFF000000),
                            )
                            .child(label().text(self.title.clone())),
                    ),
            )
    }
}
