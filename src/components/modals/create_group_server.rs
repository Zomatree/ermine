use freya::prelude::*;

use crate::components::{Dialog, ModalValue, use_modals};

#[derive(PartialEq)]
pub struct CreateGroupServer {}

impl Component for CreateGroupServer {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        Dialog::new()
            .title(label().line_height(2.).text("Create a group or server"))
            .body(label().text("Would you like to create a new group or server?"))
            .action("Group", move || {
                modals.write().push_modal(ModalValue::CreateGroup);
                false
            })
            .action("Server", move || {
                modals.write().push_modal(ModalValue::CreateServer);
                false
            })
    }
}
