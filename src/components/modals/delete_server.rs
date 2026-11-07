use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, Selection,
    components::{Dialog, ModalValue, stoat_label, use_modals},
    http,
};

#[derive(PartialEq)]
pub struct DeleteServer {
    pub id: String,
}

impl Component for DeleteServer {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        let radio = use_radio(AppChannel::Servers);
        let selection = radio.slice_mut(AppChannel::Selection, |state| &mut state.selection);
        let server_settings = radio.slice_mut(AppChannel::ServerSettingsPage, |state| {
            &mut state.server_settings_page
        });

        Dialog::new()
            .title(label().line_height(1.5).text(format!(
                "Delete {}?",
                radio.read().servers.get(&self.id).unwrap().name
            )))
            .body(
                stoat_label().text(
                    "Once it's deleted, there's no going back.",
                ),
            )
            .default_action("Cancel")
            .action("Delete", {
                let id = self.id.clone();

                move || {
                    let id = id.clone();
                    let mut selection = selection.clone();
                    let mut server_settings = server_settings.clone();

                    modals.write().push_modal(ModalValue::MFA {
                        callback: EventHandler::new(move |ticket: v0::MFATicket| {
                            let id = id.clone();
                            selection.set(Selection::Home);
                            server_settings.set(None);

                            spawn_in_window(async move {
                                http()
                                    .delete_server(
                                        &id,
                                        Some(ticket.token),
                                        &v0::OptionsServerDelete {
                                            leave_silently: None,
                                        },
                                    )
                                    .await
                                    .unwrap();
                            });
                        }),
                    });

                    false
                }
            })
    }
}
