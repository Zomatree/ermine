use freya::prelude::*;

use crate::{components::{Dialog, SingleLineEntry}, use_config};

#[derive(PartialEq)]
pub struct EditApi {
}

impl Component for EditApi {
    fn render(&self) -> impl IntoElement {
        let mut config = use_config();

        let url = use_state(|| config.read().api.clone());

        Dialog::new()
            .title(label().line_height(1.5).text("Edit Api Url"))
            .body(SingleLineEntry::new("Api Url", url))
            .default_action("Cancel")
            .action("Save", move || {
                config.write().api = url.read().cloned();
                true
            })
    }
}
