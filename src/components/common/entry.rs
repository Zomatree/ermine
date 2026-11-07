use std::borrow::Cow;

use freya::{
    animation::{
        AnimColor, AnimNum, AnimatedValue, OnChange, OnCreation, ReadAnimatedValue,
        use_animation_with_dependencies,
    },
    prelude::*,
};

use crate::{components::stoat_label, consume_material_theme, text_entry_func};

#[derive(PartialEq)]
pub struct SingleLineEntry {
    value: Writable<String>,
    title: Cow<'static, str>,
    placeholder: Option<Cow<'static, str>>,
    max_length: Option<usize>,
    on_submit: Option<EventHandler<String>>,
    mode: InputMode,
    layout: LayoutData,
}

impl SingleLineEntry {
    pub fn new(title: impl Into<Cow<'static, str>>, value: impl IntoWritable<String>) -> Self {
        Self {
            value: value.into_writable(),
            title: title.into(),
            placeholder: None,
            max_length: None,
            on_submit: None,
            mode: InputMode::Shown,
            layout: LayoutData::default(),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<Cow<'static, str>>) -> Self {
        self.placeholder = Some(placeholder.into());

        self
    }

    pub fn mode(mut self, mode: InputMode) -> Self {
        self.mode = mode;

        self
    }

    pub fn password(mut self) -> Self {
        self.mode = InputMode::Hidden('•');

        self
    }

    pub fn on_submit(mut self, on_submit: impl Into<EventHandler<String>>) -> Self {
        self.on_submit = Some(on_submit.into());
        self
    }

    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);

        self
    }
}

impl LayoutExt for SingleLineEntry {
    fn layout_mut(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl ContainerExt for SingleLineEntry {}

impl Component for SingleLineEntry {
    fn render(&self) -> impl IntoElement {
        let a11y_id = use_a11y();
        let focus = use_focus(a11y_id);
        let theme = consume_material_theme();

        let title_animation = use_animation_with_dependencies(
            &(!self.value.read().is_empty() || focus.read().is_focused()),
            move |anim, focused| {
                anim.on_creation(OnCreation::Finish);
                anim.on_change(OnChange::Rerun);

                let num = |a, b| AnimNum::new(a, b).time(200).function(text_entry_func());

                let title_scale = num(1., 0.75);
                let top = num(16., 4.);

                if *focused {
                    (title_scale, top)
                } else {
                    (title_scale, top).into_reversed()
                }
            },
        );

        let color_animation =
            use_animation_with_dependencies(&focus.read().is_focused(), move |anim, focused| {
                anim.on_creation(OnCreation::Finish);
                anim.on_change(OnChange::Rerun);

                let color = |a, b| AnimColor::new(a, b).time(200).function(text_entry_func());

                let title_color = color(
                    theme.md.on_surface_variant.as_argb_u32(),
                    theme.md.primary.as_argb_u32(),
                );
                let placeholder_color = color(
                    // transparent
                    theme.md.on_surface_variant.as_argb_u32() & 0xFFFFFF,
                    theme.md.on_surface_variant.as_argb_u32(),
                );

                if *focused {
                    (title_color, placeholder_color)
                } else {
                    (title_color, placeholder_color).into_reversed()
                }
            });

        let (title_scale, title_top) = title_animation.read().value();
        let (title_color, placeholder_color) = color_animation.read().value();

        rect()
            .layout(self.layout.clone())
            .child(
                rect()
                    .corner_radius(CornerRadius::new_symmetric(4., 0.))
                    .background(theme.md.surface_container_highest.as_argb_u32())
                    .child(
                        rect()
                            .padding((2., 16.))
                            .center()
                            .font_size(16.)
                            .child(
                                Input::new(self.value.clone())
                                    .padding((24., 14., 8., 0.))
                                    .a11y_id(a11y_id)
                                    .color(theme.md.on_surface.as_argb_u32())
                                    .placeholder_color(placeholder_color)
                                    .map(self.on_submit.clone(), |this, on_submit| {
                                        this.on_submit(on_submit)
                                    })
                                    .map(self.placeholder.clone(), |this, placeholder| {
                                        this.placeholder(placeholder)
                                    })
                                    .on_validate({
                                        let max_length = self.max_length;
                                        move |validator: InputValidator| {
                                            if let Some(max_length) = max_length {
                                                if validator.text().len() > max_length {
                                                    validator.set_valid(false);
                                                }
                                            }
                                        }
                                    })
                                    .mode(self.mode.clone())
                                    .width(Size::Fill)
                                    .flat()
                                    .background(Color::TRANSPARENT)
                                    .focus_background(Color::TRANSPARENT)
                                    .focus_border_fill(Color::TRANSPARENT),
                            )
                            .child(
                                rect()
                                    .child(
                                        stoat_label()
                                            .text(self.title.clone())
                                            .text_align(TextAlign::Start)
                                            .max_lines(1)
                                            .color(title_color)
                                            .font_weight(500)
                                            .font_size(16),
                                    )
                                    .position(Position::new_absolute().top(title_top))
                                    .transform_origin(TransformOrigin::top_left())
                                    .scale(title_scale)
                                    .layer(1),
                            ),
                    )
                    .child(
                        rect()
                            .width(Size::percent(100.))
                            .height(Size::px(1.))
                            .background(title_color)
                            .layer(1),
                    ),
            )
            .maybe_child(self.max_length.map(|max_length| {
                rect()
                    .width(Size::Fill)
                    .padding((4., 16.))
                    .cross_align(Alignment::End)
                    .color(theme.md.on_surface_variant)
                    .font_size(12.)
                    .font_weight(500)
                    .child(stoat_label().text(format!("{}/{max_length}", self.value.read().len())))
            }))
    }
}
