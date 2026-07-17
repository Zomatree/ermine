use freya::prelude::*;

use crate::{BASE, HTTP, HttpClient, use_config, consume_material_theme};

#[derive(PartialEq)]
pub struct HttpManager {
    children: Vec<Element>,
}

impl HttpManager {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl ChildrenExt for HttpManager {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl Component for HttpManager {
    fn render(&self) -> impl IntoElement {
        let config = use_config();
        let theme = consume_material_theme();

        let future = use_future(move || {
            let config = config.clone();

            async move {
                let http = HttpClient::new(BASE.to_string(), config.read().token.clone())
                    .await
                    .unwrap();
                HTTP.set(http).unwrap();
            }
        });

        let mut rect = rect().width(Size::Fill).height(Size::Fill).center();

        rect = match *future.state() {
            FutureState::Fulfilled(_) => rect.children(self.children.clone()),
            _ => rect.child(CircularLoader::new().primary_color(theme.md.on_surface.as_argb_u32())),
        };

        rect
    }
}
