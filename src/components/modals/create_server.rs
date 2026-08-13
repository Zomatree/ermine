use freya::{
    prelude::*,
    radio::{use_radio, use_radio_station},
};
use stoat_models::v0;

use crate::{
    AppChannel, AppState, Selection,
    components::{Dialog, SingleLineEntry, use_modals},
    consume_material_theme, http, insert_channel, insert_member, insert_server,
};

#[derive(PartialEq)]
pub struct CreateServer {}

impl Component for CreateServer {
    fn render(&self) -> impl IntoElement {
        let station = use_radio_station::<AppState, AppChannel>();
        let radio = use_radio(AppChannel::UserId);

        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());
        let selected_channel = radio.slice_mut(AppChannel::SelectedChannel, |state| {
            &mut state.selected_channel
        });
        let selection = radio.slice_mut(AppChannel::Selection, |state| &mut state.selection);

        let mut modals = use_modals();
        let theme = consume_material_theme();
        let name = use_state(String::new);
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(2.).text("Create server"))
            .body(
                rect()
                    .spacing(8.)
                    .child(
                        rect()
                            .horizontal()
                            .spacing(4.)
                            .child("By creating this server, you agree to the")
                            .child(
                                label()
                                    .color(theme.md.primary.as_argb_u32())
                                    .on_pointer_enter(move |_| {
                                        Cursor::set(CursorIcon::Pointer);
                                    })
                                    .on_pointer_leave(move |_| {
                                        Cursor::set(CursorIcon::default());
                                    })
                                    .on_press(move |_| {
                                        open::that_in_background("https://stoat.chat/aup");
                                    })
                                    .text("Acceptable Use Policy"),
                            ),
                    )
                    .child(SingleLineEntry::new("Server Name", name))
                    .maybe_child(
                        error
                            .read()
                            .clone()
                            .map(|error| label().text(error).color(theme.md.error.as_argb_u32())),
                    ),
            )
            .default_action("Close")
            .action("Create", move || {
                let user_id = user_id.clone();
                let mut selected_channel = selected_channel.clone();
                let mut selection = selection.clone();

                spawn({
                    let name = name.read().clone();

                    async move {
                        match http()
                            .create_server(&v0::DataCreateServer {
                                name,
                                description: None,
                                nsfw: None,
                            })
                            .await
                        {
                            Ok(response) => {
                                let id = response.server.id.clone();

                                if let Some(first_channel) = response.channels.first() {
                                    *selected_channel.write() =
                                        Some(first_channel.id().to_string());
                                }

                                for channel in response.channels {
                                    insert_channel(channel, station);
                                }

                                insert_server(response.server, station);

                                let user_id = user_id.read().clone();

                                let member = http().fetch_member(&id, &user_id).await.unwrap();
                                insert_member(member, station);

                                *selection.write() = Selection::Server(id.clone());

                                modals.write().pop_modal();
                            }
                            Err(e) => error.set(Some(format!("{e:?}"))),
                        };
                    }
                });

                false
            })
    }
}
