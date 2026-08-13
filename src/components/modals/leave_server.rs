use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Dialog, checkbox::StoatCheckbox, use_modals},
    http,
};

#[derive(PartialEq)]
pub struct LeaveServer {
    pub server: String,
}

impl Component for LeaveServer {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);

        let server = radio.slice_current({
            let server = self.server.clone();
            move |state| state.servers.get(&server).unwrap()
        });

        let mut modals = use_modals();

        let silent = use_state(|| false);

        Dialog::new()
            .title(
                label()
                    .line_height(1.5)
                    .text(format!("Leave {}", server.read().name.clone())),
            )
            .body(
                rect()
                    .spacing(8.)
                    .child("You won't be able to rejoin unless you are re-invited.")
                    .child(
                        StoatCheckbox::new(silent).child("Don't notify others that you've left"),
                    ),
            )
            .default_action("Cancel")
            .action("Leave", {
                let server = self.server.clone();

                move || {
                    spawn({
                        let server = server.clone();
                        async move {
                            modals.write().pop_modal();

                            http()
                                .delete_server(
                                    &server,
                                    &v0::OptionsServerDelete {
                                        leave_silently: Some(silent()),
                                    },
                                )
                                .await
                                .unwrap();
                        }
                    });

                    false
                }
            })
    }
}
