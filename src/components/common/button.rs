use freya::prelude::*;

use crate::consume_material_theme;

define_theme! {
    for = StoatButton;
    theme_field = theme_layout;

    %[component]
    pub StoatButtonLayout {
        %[fields]
        margin: Gaps,
        corner_radius: CornerRadius,
        width: Size,
        height: Size,
        padding: Gaps,
    }
}

define_theme! {
    for = StoatButton;
    theme_field = theme_colors;

    %[component]
    pub StoatButtonColors {
        %[fields]
        background: Color,
        hover_background: Color,
        border_fill: Color,
        focus_border_fill: Color,
        color: Color,
    }
}

#[derive(PartialEq)]
pub struct StoatButton {
    theme_colors: Option<StoatButtonColorsThemePartial>,
    theme_layout: Option<StoatButtonLayoutThemePartial>,
    elements: Vec<Element>,
    on_press: Option<EventHandler<Event<PressEventData>>>,
    on_hover: Option<EventHandler<Event<PointerEventData>>>,
    key: DiffKey,
}

impl StoatButton {
    pub fn new() -> Self {
        Self {
            theme_colors: None,
            theme_layout: None,
            on_press: None,
            on_hover: None,
            elements: Vec::default(),
            key: DiffKey::None,
        }
    }

    pub fn on_press(mut self, on_press: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_press = Some(on_press.into());
        self
    }

    pub fn on_hover(mut self, on_hover: impl Into<EventHandler<Event<PointerEventData>>>) -> Self {
        self.on_hover = Some(on_hover.into());
        self
    }
}

impl ChildrenExt for StoatButton {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.elements
    }
}

impl CornerRadiusExt for StoatButton {
    fn with_corner_radius(self, corner_radius: f32) -> Self {
        self.corner_radius(corner_radius)
    }
}

impl KeyExt for StoatButton {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for StoatButton {
    fn render(&self) -> impl IntoElement {
        let mut hovering = use_state(|| false);
        let a11y_id = use_a11y();
        let theme = consume_material_theme();
        let mut size = use_state(Size2D::default);

        use_drop(move || {
            if hovering() {
                Cursor::set(CursorIcon::default());
            }
        });

        let theme_colors = get_theme!(
            &self.theme_colors,
            StoatButtonColorsThemePreference,
            "stoat_button"
        );
        let theme_layout = get_theme!(
            &self.theme_layout,
            StoatButtonLayoutThemePreference,
            "stoat_button_layout"
        );

        let color = if theme_colors.color == Color::TRANSPARENT {
            theme.md.on_surface.as_argb_u32().into()
        } else {
            theme_colors.color
        };

        rect()
            .overflow(Overflow::Clip)
            .a11y_id(a11y_id)
            .a11y_role(AccessibilityRole::Button)
            .background(theme_colors.background)
            .padding(theme_layout.padding)
            .margin(theme_layout.margin)
            .corner_radius(theme_layout.corner_radius)
            .width(theme_layout.width)
            .height(theme_layout.height)
            .color(color)
            .on_all_press({
                let on_press = self.on_press.clone();
                move |e: Event<PressEventData>| {
                    a11y_id.request_focus();
                    match e.data() {
                        PressEventData::Mouse(data) => match data.button {
                            Some(MouseButton::Left) => {
                                if let Some(handler) = &on_press {
                                    handler.call(e);
                                }
                            }
                            _ => {}
                        },
                        PressEventData::Touch(_) | PressEventData::Keyboard(_) => {
                            if let Some(handler) = &on_press {
                                handler.call(e);
                            }
                        }
                    }
                }
            })
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .on_pointer_enter({
                let on_hover = self.on_hover.clone();
                move |e| {
                    Cursor::set(CursorIcon::Pointer);

                    if let Some(on_hover) = &on_hover {
                        on_hover.call(e);
                    }
                }
            })
            .on_pointer_leave(move |_| {
                Cursor::set(CursorIcon::default());
            })
            .on_sized(move |e: Event<SizedEventData>| size.set(e.area.size))
            .child(
                rect()
                    // .layer(Layer::Relative(-100))
                    .children(self.elements.clone()),
            )
            .child(
                rect()
                    .position(Position::new_absolute())
                    .layer(Layer::Relative(100))
                    .width(Size::px(size.read().width))
                    .height(Size::px(size.read().height))
                    .interactive(false)
                    .background(theme.md.on_surface.as_argb_u32())
                    .overflow(Overflow::Clip)
                    .corner_radius(theme_layout.corner_radius)
                    .opacity(if *hovering.read() { 0.08 } else { 0. }),
            )
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
