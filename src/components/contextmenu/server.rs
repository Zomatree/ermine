use freya::{prelude::*, radio::use_radio};

use crate::{AppChannel, ServerSettingsPage, components::{ContextMenuButton, ModalValue, material::outlined::{badge, logout, settings}, use_modals}};

#[derive(PartialEq)]
pub struct ServerContextMenu {
    pub server_id: String,
}

impl Component for ServerContextMenu {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);

        let server = radio.slice_current({
            let server_id = self.server_id.clone();
            move |state| state.servers.get(&server_id).unwrap()
        });

        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());

        let mut server_settings = radio.slice_mut(AppChannel::ServerSettingsPage, |state| {
            &mut state.server_settings_page
        });

        let mut modals = use_modals();

        rect()
            .content(Content::Fit)
            .child(
                ContextMenuButton::new(settings(), "Open Server Settings")
                    .on_press({
                        let server_id = self.server_id.clone();

                        move |_| {
                            *server_settings.write() =
                                Some((server_id.clone(), ServerSettingsPage::default()));
                        }
                    }),
            )
            .maybe_child((&server.read().owner != &*user_id.read()).then(|| {
                ContextMenuButton::new(logout(), "Leave Server")
                .danger()
                    .on_press({
                        let server_id = self.server_id.clone();

                        move |_| {
                            modals.write().push_modal(ModalValue::LeaveServer { server: server_id.clone() });
                        }
                    })
            }))
            .child(
                ContextMenuButton::new(badge(), "Copy Server ID")
                    .on_press({
                        let server_id = self.server_id.clone();

                        move |_| {
                            Clipboard::set(server_id.clone()).unwrap();
                        }
                    }),
            )
    }
}
