use freya::{
    prelude::*,
    radio::{use_radio, use_radio_station},
};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Dialog, SingleLineEntry, use_modals},
    consume_material_theme, http, insert_server,
};

#[derive(PartialEq)]
pub struct RenameCategory {
    pub server: String,
    pub category: String,
}

impl Component for RenameCategory {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);
        let server = radio.slice_current({
            let server = self.server.clone();
            move |state| state.servers.get(&server).unwrap()
        });

        let station = use_radio_station();

        let mut modals = use_modals();
        let theme = consume_material_theme();
        let name = use_state(|| {
            server
                .read()
                .categories
                .as_ref()
                .and_then(|cats| {
                    cats.iter()
                        .find(|c| &c.id == &self.category)
                        .map(|c| c.title.clone())
                })
                .unwrap_or_default()
        });
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(2.).text("Rename category"))
            .body(
                rect()
                    .spacing(8.)
                    .child(SingleLineEntry::new("Category Name", name))
                    .maybe_child(
                        error
                            .read()
                            .clone()
                            .map(|error| label().text(error).color(theme.md.error.as_argb_u32())),
                    ),
            )
            .default_action("Close")
            .action("Rename", {
                let server_id = self.server.clone();
                let server = server.clone();
                let category = self.category.clone();

                move || {
                    spawn({
                        let name = name.read().clone();
                        let server_id = server_id.clone();
                        let server = server.clone();
                        let category = category.clone();

                        async move {
                            let mut server = server.read().cloned();

                            if let Some(categories) = &mut server.categories
                                && let Some(category) =
                                    categories.iter_mut().find(|c| &c.id == &category)
                            {
                                category.title = name;
                            }

                            match http()
                                .edit_server(
                                    &server_id,
                                    &v0::DataEditServer {
                                        name: None,
                                        description: None,
                                        icon: None,
                                        banner: None,
                                        categories: server.categories,
                                        system_messages: None,
                                        flags: None,
                                        discoverable: None,
                                        analytics: None,
                                        owner: None,
                                        remove: Vec::new(),
                                    },
                                )
                                .await
                            {
                                Ok(server) => {
                                    insert_server(server, station);

                                    modals.write().pop_modal();
                                }
                                Err(e) => error.set(Some(format!("{e:?}"))),
                            };
                        }
                    });

                    false
                }
            })
    }
}
