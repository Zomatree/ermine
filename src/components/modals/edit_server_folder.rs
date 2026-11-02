use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel,
    components::{ColorPickerPallete, Dialog, SingleLineEntry, stoat_label},
    parse_fill, use_initial,
};

#[derive(PartialEq)]
pub struct EditServerFolder {
    pub folder: String,
}

impl Component for EditServerFolder {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Settings("server-folders"));

        let mut folder = radio.slice_mut_current({
            let id = self.folder.clone();
            move |state| {
                state
                    .settings
                    .server_folders
                    .get_or_insert_default()
                    .folders
                    .iter_mut()
                    .find(|f| &f.id == &id)
                    .unwrap()
            }
        });

        let name = use_initial(|| folder.read().name.clone());

        let mut fill = use_initial(|| {
            folder
                .read()
                .colour
                .as_ref()
                .and_then(|color| parse_fill(color))
        });

        Dialog::new()
            .title(label().line_height(1.5).text("Edit Folder"))
            .body(
                rect()
                    .spacing(8.)
                    .child(SingleLineEntry::new("Name", name))
                    .child(
                        stoat_label()
                            .font_size(12.)
                            .font_weight(500)
                            .text("Folder colour"),
                    )
                    .child(ColorPickerPallete::new(
                        fill.read().cloned(),
                        WritableSetter::new(
                            fill.read().as_ref().and_then(|f| f.as_color()),
                            move |c: Option<Color>| {
                                fill.set(c.map(Into::into));
                            },
                        ),
                    )),
            )
            .default_action("Cancel")
            .action("Save", move || {
                let mut folder = folder.write();

                if let Some(name) = name.get_if_different() {
                    folder.name = name
                }

                if let Some(fill) = fill.get_if_different() {
                    if let Some(fill) = fill
                        && let Some(color) = fill.as_color()
                    {
                        folder.colour = Some(color.to_hex_string())
                    } else {
                        folder.colour = None
                    }
                }

                true
            })
    }
}
