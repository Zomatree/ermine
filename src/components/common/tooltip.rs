use freya::{
    animation::{AnimNum, AnimatedValue, Ease, Function, OnChange, OnCreation, use_animation},
    prelude::*,
};

#[derive(PartialEq)]
pub struct StoatTooltip {
    tooltip: Element,
    children: Vec<Element>,
    position: AttachedPosition,
    layout: LayoutData,
    key: DiffKey,
}

impl KeyExt for StoatTooltip {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl LayoutExt for StoatTooltip {
    fn layout_mut(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl ChildrenExt for StoatTooltip {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl StoatTooltip {
    pub fn new(tooltip: impl IntoElement) -> Self {
        Self {
            tooltip: tooltip.into_element(),
            children: vec![],
            position: AttachedPosition::Bottom,
            layout: LayoutData::default(),
            key: DiffKey::None,
        }
    }

    pub fn position(mut self, position: AttachedPosition) -> Self {
        self.position = position;
        self
    }
}

impl Component for StoatTooltip {
    fn render(&self) -> impl IntoElement {
        let mut is_hovering = use_state(|| false);

        let animation = use_animation(move |conf| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Finish);

            let opacity = AnimNum::new(0., 1.)
                .time(350)
                .ease(Ease::Out)
                .function(Function::Expo);

            if is_hovering() {
                opacity
            } else {
                opacity.into_reversed()
            }
        });

        let opacity = animation.read().value();

        let on_pointer_over = move |_| {
            is_hovering.set(true);
        };

        let on_pointer_out = move |_| {
            is_hovering.set(false);
        };

        let is_visible = opacity > 0. && !ContextMenu::is_open();

        let padding = match self.position {
            AttachedPosition::Top => (0., 0., 5., 0.),
            AttachedPosition::Bottom => (5., 0., 0., 0.),
            AttachedPosition::Left => (0., 5., 0., 0.),
            AttachedPosition::Right => (0., 0., 0., 5.),
        };

        rect()
            .layout(self.layout.clone())
            .a11y_focusable(false)
            .a11y_role(AccessibilityRole::Tooltip)
            .on_pointer_over(on_pointer_over)
            .on_pointer_out(on_pointer_out)
            .child(
                Attached::new(rect().children(self.children.clone()))
                    .position(self.position)
                    .maybe_child(is_visible.then(|| {
                        rect().opacity(opacity).padding(padding).child(
                            rect()
                                .interactive(Interactive::No)
                                .padding(8.)
                                .background(Color::BLACK)
                                .color(Color::WHITE)
                                .corner_radius(12.)
                                .child(self.tooltip.clone()),
                        )
                    })),
            )
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
