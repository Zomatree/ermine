use freya::{
    prelude::*,
    radio::{use_radio, use_radio_station},
};
use stoat_models::v0;

use crate::{
    AppChannel, AppState, Selection,
    components::{Dialog, SingleLineEntry, use_modals},
    consume_material_theme, format_error, http, insert_channel, insert_member, insert_server,
    insert_user,
};

#[derive(PartialEq)]
pub struct JoinServer {}

impl Component for JoinServer {
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
        let code = use_state(String::new);
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(2.).text("Join a server"))
            .body(
                rect()
                    .spacing(8.)
                    .child(label().text("Use a code or invite link"))
                    .child(SingleLineEntry::new("Code", code))
                    .maybe_child(
                        error
                            .read()
                            .clone()
                            .map(|error| label().text(error).color(theme.md.error.as_argb_u32())),
                    ),
            )
            .default_action("Close")
            .action("Join", move || {
                let user_id = user_id.clone();
                let mut selected_channel = selected_channel.clone();
                let mut selection = selection.clone();

                spawn({
                    let code = code.read().clone();

                    let code = if code.contains('/')
                        && let Ok(uri) = code.parse::<Url>()
                    {
                        uri.path()
                            .split('/')
                            .last()
                            .map(|p| p.to_string())
                            .unwrap_or_default()
                    } else {
                        code
                    };

                    async move {
                        match http().join_invite(&code).await {
                            Ok(response) => match response {
                                v0::InviteJoinResponse::Server { channels, server } => {
                                    let id = server.id.clone();

                                    if let Some(first_channel) = channels.first() {
                                        *selected_channel.write() =
                                            Some(first_channel.id().to_string());
                                    }

                                    for channel in channels {
                                        insert_channel(channel, station);
                                    }

                                    insert_server(server, station);

                                    let user_id = user_id.read().clone();

                                    let member = http().fetch_member(&id, &user_id).await.unwrap();
                                    insert_member(member, station);

                                    *selection.write() = Selection::Server(id.clone());

                                    modals.write().pop_modal();
                                }
                                v0::InviteJoinResponse::Group { channel, users } => {
                                    let id = channel.id().to_string();

                                    insert_channel(channel, station);

                                    for user in users {
                                        insert_user(user, station);
                                    }

                                    *selected_channel.write() = Some(id);
                                    *selection.write() = Selection::Home;

                                    modals.write().pop_modal();
                                }
                            },
                            Err(e) => error.set(Some(format_error(&e, "Server"))),
                        };
                    }
                });

                false
            })
    }
}
