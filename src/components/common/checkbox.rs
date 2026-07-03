use std::time::Duration;

use freya::{
    animation::{AnimColor, AnimatedValue, Ease, OnChange, OnCreation, use_animation},
    icons::lucide::{square, square_check},
    prelude::*,
};

use crate::use_material_theme;

#[derive(PartialEq)]
pub struct StoatCheckbox {
    value: Writable<bool>,
    children: Vec<Element>,
}

impl StoatCheckbox {
    pub fn new(value: impl IntoWritable<bool>) -> Self {
        Self {
            value: value.into_writable(),
            children: Vec::new(),
        }
    }
}

impl ChildrenExt for StoatCheckbox {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl Component for StoatCheckbox {
    fn render(&self) -> impl IntoElement {
        let mut hover = use_state(|| false);
        let a11y_id = use_a11y();
        let theme = use_material_theme();

        let animation = use_animation({
            let value = self.value.clone();
            move |anim| {
                anim.on_creation(OnCreation::Finish);
                anim.on_change(OnChange::Rerun);

                let c = AnimColor::new(
                    theme.md.on_surface.as_argb_u32(),
                    theme.md.primary.as_argb_u32(),
                )
                .duration(Duration::from_millis(200))
                .ease(Ease::InOut);

                if *value.read() { c } else { c.into_reversed() }
            }
        });

        let color = animation.read().value();

        use_drop(move || {
            if hover() {
                Cursor::set(CursorIcon::default());
            }
        });

        rect()
            .horizontal()
            .spacing(8.)
            .a11y_id(a11y_id)
            .a11y_role(AccessibilityRole::CheckBox)
            .cross_align(Alignment::Center)
            .on_press({
                let mut value = self.value.clone();

                move |_| {
                    a11y_id.request_focus();

                    let mut v = value.write();
                    *v = !*v;
                }
            })
            .on_pointer_over(move |_| {
                hover.set(true);
            })
            .on_pointer_out(move |_| hover.set_if_modified(false))
            .on_pointer_enter(move |_| {
                Cursor::set(CursorIcon::Pointer);
            })
            .on_pointer_leave(move |_| {
                Cursor::set(CursorIcon::default());
            })
            .child(
                rect()
                    .child(
                        rect()
                            .width(Size::px(40.))
                            .height(Size::px(40.))
                            .center()
                            .color(color)
                            .child(
                                SvgViewer::new(if *self.value.read() { square_check() } else { square() })
                                    .width(Size::px(24.))
                                    .height(Size::px(24.)),
                            ),
                    )
                    .maybe_child(hover().then(|| {
                        rect()
                            .position(Position::new_absolute().left(0.).top(0.))
                            .width(Size::px(40.))
                            .height(Size::px(40.))
                            .corner_radius(20.)
                            .background(color)
                            .opacity(0.08)
                    })),
            )
            .child(
                rect()
                    .min_height(Size::px(40.))
                    .width(Size::Fill)
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .children(self.children.clone()),
            )
    }
}
