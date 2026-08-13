use freya::prelude::*;

use crate::components::{Dialog, ModalValue, use_modals};

#[derive(PartialEq)]
pub struct CreateJoinServer {}

impl Component for CreateJoinServer {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        Dialog::new()
            .title(label().line_height(2.).text("Create or join a server"))
            .body(label().text("Would you like to create a new server or join an existing one?"))
            .action("Create", move || {
                modals.write().push_modal(ModalValue::CreateServer);
                false
            })
            .action("Join", move || {
                modals.write().push_modal(ModalValue::JoinServer);
                false
            })
    }
}
