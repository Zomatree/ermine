use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;
use ulid::Ulid;

use crate::{
    AppChannel,
    components::{Dialog, SingleLineEntry, use_modals},
    consume_material_theme, http,
};

#[derive(PartialEq)]
pub struct CreateCategory {
    pub server: String,
}

impl Component for CreateCategory {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let radio = use_radio(AppChannel::Servers);
        let server = radio.slice_current({
            let server = self.server.clone();
            move |state| state.servers.get(&server).unwrap()
        });

        let name = use_state(String::new);
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(2.).text("Create Channel"))
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
            .action("Create", {
                move || {
                    spawn({
                        let name = name.read().clone();
                        let server = server.read().clone();

                        let mut categories = server.categories.unwrap_or_default();
                        categories.push(v0::Category {
                            id: Ulid::new().to_string(),
                            title: name,
                            channels: Vec::new()
                        });

                        async move {

                            match http().edit_server(&server.id, &v0::DataEditServer {
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
                            }).await {
                                Ok(_) => {
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
