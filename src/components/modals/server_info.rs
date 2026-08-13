use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel, ServerSettingsPage,
    components::{Dialog, MarkdownViewer},
};

#[derive(PartialEq)]
pub struct ServerInfo {
    pub server: String,
}

impl Component for ServerInfo {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);

        let server = radio.read().servers.get(&self.server).unwrap().clone();
        let server_settings = radio.slice_mut(AppChannel::ServerSettingsPage, |state| {
            &mut state.server_settings_page
        });

        let readable = server.clone().into_readable();

        Dialog::new()
            .title(label().line_height(1.5).text(server.name))
            .body(MarkdownViewer::new(
                server.description.unwrap_or_default(),
                Some(readable),
            ))
            .action("Settings", move || {
                *server_settings.clone().write() =
                    Some((server.id.clone(), ServerSettingsPage::default()));
                true
            })
            .default_action("Close")
    }
}
