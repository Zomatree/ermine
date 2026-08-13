use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, SelectedRole, ServerSettingsPage,
    components::{Dialog, SingleLineEntry, use_modals},
    consume_material_theme, http,
};

#[derive(PartialEq)]
pub struct CreateRole {
    pub server: String,
}

impl Component for CreateRole {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);

        let server_settings_page = radio.slice_mut(AppChannel::ServerSettingsPage, |state| {
            &mut state.server_settings_page
        });

        let mut modals = use_modals();
        let theme = consume_material_theme();
        let name = use_state(String::new);
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(2.).text("Create Role"))
            .body(
                rect()
                    .spacing(8.)
                    .child(SingleLineEntry::new("Role Name", name))
                    .maybe_child(
                        error
                            .read()
                            .clone()
                            .map(|error| label().text(error).color(theme.md.error.as_argb_u32())),
                    ),
            )
            .default_action("Close")
            .action("Create", {
                let server = self.server.clone();
                let radio = radio.clone();
                let server_settings_page = server_settings_page.clone();
                move || {
                    spawn({
                        let name = name.read().clone();
                        let server = server.clone();
                        let radio = radio.clone();
                        let server_settings_page = server_settings_page.clone();

                        async move {
                            match http()
                                .create_role(&server, &v0::DataCreateRole { name, rank: None })
                                .await
                            {
                                Ok(response) => {
                                    radio
                                        .clone()
                                        .write()
                                        .servers
                                        .get_mut(&server)
                                        .unwrap()
                                        .roles
                                        .insert(response.id.clone(), response.role);

                                    if let Some(page) = &mut *server_settings_page.clone().write() {
                                        page.1 = ServerSettingsPage::Roles(Some(
                                            SelectedRole::Role(response.id),
                                        ));
                                    };

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
