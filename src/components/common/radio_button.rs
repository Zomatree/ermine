use freya::prelude::*;

use crate::{
    SizeExt,
    components::{
        MaterialIcon,
        material::outlined::{radio_button_checked, radio_button_unchecked},
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct RadioButton {
    selected: bool,
    children: Vec<Element>,

    on_press: Option<EventHandler<Event<PressEventData>>>,
}

impl RadioButton {
    pub fn new(selected: bool) -> Self {
        Self {
            selected,
            children: Vec::new(),
            on_press: None,
        }
    }

    pub fn on_press(mut self, on_press: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_press = Some(on_press.into());
        self
    }
}

impl ChildrenExt for RadioButton {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl Component for RadioButton {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let mut hovering = use_state(|| false);

        let background = Color::from(
            if self.selected {
                theme.md.primary
            } else {
                theme.md.on_surface
            }
            .as_argb_u32(),
        );

        let color = if self.selected {
            theme.md.primary
        } else if hovering() {
            theme.md.on_surface
        } else {
            theme.md.on_surface_variant
        }
        .as_argb_u32();

        rect()
            .horizontal()
            .cursor(CursorIcon::Pointer)
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .map(self.on_press.clone(), |this, on_press| {
                this.on_press(on_press)
            })
            .cross_align(Alignment::Center)
            .child(
                rect()
                    .size(Size::px(40.))
                    .center()
                    .child(
                        MaterialIcon::new(if self.selected {
                            radio_button_checked()
                        } else {
                            radio_button_unchecked()
                        })
                        .size(Size::px(24.))
                        .color(color),
                    )
                    .child(
                        rect()
                            .position(Position::new_absolute())
                            .layer(Layer::Relative(100))
                            .size(Size::px(40.))
                            .corner_radius(20.)
                            .interactive(false)
                            .background(if hovering() {
                                background.with_af32(0.08)
                            } else {
                                background.with_af32(0.)
                            }),
                    ),
            )
            .child(
                rect()
                    .font_size(14.)
                    .color(theme.md.on_surface_variant.as_argb_u32())
                    .children(self.children.clone()),
            )
    }
}
