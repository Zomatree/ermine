use std::time::Duration;

use freya::{
    animation::{
        AnimColor, AnimNum, AnimatedValue, Ease, OnChange, OnCreation, ReadAnimatedValue,
        use_animation, use_animation_with_dependencies,
    },
    prelude::*,
};

use crate::{
    SizeExt,
    components::{
        MaterialIcon,
        material::filled::{check_box, check_box_outline_blank},
    },
    consume_material_theme,
};

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

    pub fn from_writable(value: Writable<bool>) -> Self {
        Self {
            value,
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

        let theme = consume_material_theme();

        let animation = use_animation_with_dependencies(&*self.value.read(), {
            move |anim, value| {
                anim.on_creation(OnCreation::Finish);
                anim.on_change(OnChange::Rerun);

                let opacity = AnimNum::new(0., 1.)
                    .duration(Duration::from_millis(200))
                    .ease(Ease::Out);

                if *value {
                    opacity
                } else {
                    opacity.into_reversed()
                }
            }
        });

        let opacity = animation.read().value();
        let color = Color::lerp(
            theme.md.on_surface.as_argb_u32().into(),
            theme.md.primary.as_argb_u32().into(),
            opacity,
        );

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
            .cursor(CursorIcon::Pointer)
            .child(
                rect()
                    .child(
                        rect()
                            .width(Size::px(40.))
                            .height(Size::px(40.))
                            .center()
                            .child(
                                rect()
                                    .child(
                                        MaterialIcon::new(check_box_outline_blank())
                                            .color(color)
                                            .size(Size::px(24.)),
                                    )
                                    .child(
                                        MaterialIcon::new(check_box())
                                            .opacity(opacity)
                                            .color(color)
                                            .size(Size::px(24.))
                                            .position(Position::new_absolute()),
                                    ),
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
