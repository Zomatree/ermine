use std::{fmt::Debug, hash::Hash};

use freya::{
    animation::{
        AnimNum, AnimatedValue, Ease, OnChange, OnCreation, use_animation_with_dependencies,
    },
    prelude::*,
};

use crate::{
    components::{StoatButton, StoatButtonLayoutThemePartialExt},
    consume_material_theme,
};

pub struct StoatSegmentedButton<T: PartialEq + 'static, B> {
    state: Writable<T>,
    options: Vec<T>,
    builder: B,
    height: f32,
}

impl<T: PartialEq + 'static, B> PartialEq for StoatSegmentedButton<T, B> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state && self.options == other.options && self.height == other.height
    }
}

impl<T: Clone + PartialEq + 'static, B: Fn(&T) -> Element> StoatSegmentedButton<T, B> {
    pub fn new(state: impl IntoWritable<T>, options: Vec<T>, builder: B) -> Self {
        Self {
            state: state.into_writable(),
            options,
            builder,
            height: 0.,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;

        self
    }
}

impl<T: Debug + Hash + Clone + PartialEq + 'static, B: Fn(&T) -> Element + 'static> Component
    for StoatSegmentedButton<T, B>
{
    fn render(&self) -> impl IntoElement {
        let children = self
            .options
            .iter()
            .enumerate()
            .map(|(i, value)| (i, value, (self.builder)(value)));

        rect()
            .horizontal()
            .content(Content::Flex)
            .spacing(8.)
            .children(children.map(|(i, value, element)| {
                StoatInnerSegmentedButton {
                    state: self.state.clone(),
                    value: value.clone(),
                    height: self.height,
                    child: element,
                    first: i == 0,
                    last: i == self.options.len() - 1,
                }
                .into_element()
            }))
    }
}

#[derive(PartialEq)]
struct StoatInnerSegmentedButton<T: 'static> {
    pub state: Writable<T>,
    pub value: T,
    pub height: f32,
    pub child: Element,
    pub first: bool,
    pub last: bool,
}

impl<T: Hash + Clone + PartialEq + 'static> Component for StoatInnerSegmentedButton<T> {
    fn render(&self) -> impl IntoElement {
        const ANIM_TIME: u64 = 250;
        let radius = self.height / 2.;

        let theme = consume_material_theme();

        let selected = &*self.state.read() == &self.value;

        let animation = use_animation_with_dependencies(&selected, {
            move |conf, selected| {
                conf.on_change(OnChange::Rerun);
                conf.on_creation(OnCreation::Finish);

                let anim = AnimNum::new(12., radius).time(ANIM_TIME).ease(Ease::InOut);

                if *selected {
                    anim
                } else {
                    anim.into_reversed()
                }
            }
        });

        let corners_value = animation.read().value();

        let corners = CornerRadius::new(
            if self.first { radius } else { corners_value },
            if self.last { radius } else { corners_value },
            if self.last { radius } else { corners_value },
            if self.first { radius } else { corners_value },
        );

        let (color, background) = if selected {
            (theme.md.on_primary, theme.md.primary)
        } else {
            (
                theme.md.on_secondary_container,
                theme.md.secondary_container,
            )
        };

        rect()
            .width(Size::flex(1.))
            .height(Size::px(self.height))
            .child(
                StoatButton::new()
                    .width(Size::Fill)
                    .height(Size::px(self.height))
                    .corner_radius(corners)
                    .child(
                        rect()
                            .width(Size::Fill)
                            .height(Size::px(self.height))
                            .color(color.as_argb_u32())
                            .background(background.as_argb_u32())
                            .center()
                            .child(self.child.clone()),
                    )
                    .on_press({
                        let state = self.state.clone();
                        let value = self.value.clone();

                        move |_| state.clone().set(value.clone())
                    }),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.value).into()
    }
}
