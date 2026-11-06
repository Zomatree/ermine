use freya::prelude::*;

use crate::{
    SizeExt,
    components::{
        MaterialIcon, StoatButton, StoatButtonLayoutThemePartialExt,
        material::outlined::{chevron_right, content_copy, open_in_new},
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct CategoryButtonGroup {
    children: Vec<Element>,
}

impl CategoryButtonGroup {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl Component for CategoryButtonGroup {
    fn render(&self) -> impl IntoElement {
        rect()
            .corner_radius(28.)
            .overflow(Overflow::Clip)
            .spacing(2.)
            .children(self.children.clone())
    }
}

impl ChildrenExt for CategoryButtonGroup {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl WithExt for CategoryButtonGroup {}

#[derive(PartialEq)]
pub struct CategoryButton {
    icon: Bytes,
    icon_color: Option<Color>,

    action: Option<Bytes>,

    enabled: bool,

    on_press: Option<EventHandler<Event<PressEventData>>>,

    title: Vec<Element>,
    description: Vec<Element>,
}

impl CategoryButton {
    pub fn new(icon: Bytes) -> Self {
        Self {
            icon,
            icon_color: None,
            action: None,
            enabled: true,
            on_press: None,
            title: Vec::new(),
            description: Vec::new(),
        }
    }

    pub fn icon_color(mut self, color: impl Into<Color>) -> Self {
        self.icon_color = Some(color.into());

        self
    }

    pub fn chevron(mut self) -> Self {
        self.action = Some(chevron_right());

        self
    }

    pub fn copy(mut self) -> Self {
        self.action = Some(content_copy());

        self
    }

    pub fn external(mut self) -> Self {
        self.action = Some(open_in_new());

        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;

        self
    }

    pub fn on_press(mut self, on_press: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_press = Some(on_press.into());
        self
    }

    pub fn title(mut self, child: impl IntoElement) -> Self {
        self.title.push(child.into_element());

        self
    }

    pub fn description(mut self, child: impl IntoElement) -> Self {
        self.description.push(child.into_element());

        self
    }
}

impl Component for CategoryButton {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        StoatButton::new()
            .corner_radius(12.)
            .enabled(self.enabled)
            .map(self.on_press.clone(), |this, on_press| {
                this.on_press(on_press)
            })
            .child(
                rect()
                    .padding(13.)
                    .background(theme.md.secondary_container)
                    .color(theme.md.on_secondary_container)
                    .child(
                        rect()
                            .horizontal()
                            .spacing(16.)
                            .cross_align(Alignment::Center)
                            .content(Content::Flex)
                            .child(
                                rect()
                                    .corner_radius(36.)
                                    .width(Size::px(36.))
                                    .height(Size::px(36.))
                                    .background(theme.md.surface_dim)
                                    .center()
                                    .child(
                                        MaterialIcon::new(self.icon.clone())
                                            .size(Size::px(22.))
                                            .color(
                                                self.icon_color
                                                    .unwrap_or_else(|| theme.md.on_surface.into()),
                                            ),
                                    ),
                            )
                            .child(
                                rect()
                                    .width(Size::flex(1.))
                                    .child(
                                        rect()
                                            .font_size(14.)
                                            .font_weight(500)
                                            .children(self.title.clone()),
                                    )
                                    .child(
                                        rect().font_size(12.).font_weight(500).children(self.description.clone()),
                                    ),
                            )
                            .maybe_child(
                                self.action
                                    .clone()
                                    .map(|action| rect().size(Size::px(24.)).center().child(MaterialIcon::new(action).size(Size::px(18.)))),
                            ),
                    ),
            )
    }
}
