use std::borrow::Cow;

use freya::{
    animation::{
        AnimNum, AnimatedValue, Ease, Function, OnChange, OnCreation, ReadAnimatedValue,
        use_animation,
    },
    prelude::*,
};

use crate::{
    SizeExt,
    components::{MaterialIcon, StoatButton, material::filled::expand_more},
    consume_material_theme,
};

pub struct Dropdown<T: PartialEq + 'static, B> {
    title: Cow<'static, str>,
    state: Writable<T>,
    options: Vec<T>,
    builder: B,

    layout: LayoutData,
}

impl<T: PartialEq + 'static, B> PartialEq for Dropdown<T, B> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state && self.options == other.options
    }
}

impl<T: Clone + PartialEq + 'static, B: Fn(&T) -> Element> Dropdown<T, B> {
    pub fn new(
        title: impl Into<Cow<'static, str>>,
        state: Writable<T>,
        options: Vec<T>,
        builder: B,
    ) -> Self {
        Self {
            title: title.into(),
            state,
            options,
            builder,

            layout: LayoutData::default(),
        }
    }
}

impl<T: PartialEq + 'static, B> LayoutExt for Dropdown<T, B> {
    fn get_layout(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl<T: PartialEq + 'static, B> ContainerExt for Dropdown<T, B> {}

impl<T: Clone + PartialEq + 'static, B: Fn(&T) -> Element + 'static> Component for Dropdown<T, B> {
    fn render(&self) -> impl IntoElement {
        let current_value = &*self.state.read();
        let theme = consume_material_theme();

        let a11y_id = use_a11y();
        let mut hovering = use_state(|| false);
        let mut open = use_state(|| false);
        let mut button_area = use_state(Area::zero);
        let mut list_size = use_state(|| None::<Size2D>);

        let animation = use_animation(move |conf| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Finish);

            let opacity = AnimNum::new(0., 1.)
                .time(200)
                .ease(Ease::InOut)
                .function(Function::Quart);
            let offset_y = AnimNum::new(-8., 0.)
                .time(200)
                .ease(Ease::InOut)
                .function(Function::Quart);
            if open() {
                (opacity, offset_y)
            } else {
                (opacity.into_reversed(), offset_y.into_reversed())
            }
        });

        use_drop(move || {
            if hovering() {
                Cursor::set(CursorIcon::default());
            }
        });

        use_side_effect(move || {
            let platform = Platform::get();
            let should_close = platform
                .focused_accessibility_node
                .read()
                .member_of()
                .is_none_or(|member_of| member_of != a11y_id);
            if should_close {
                open.set_if_modified(false);
            }
        });

        let on_press = move |e: Event<PressEventData>| {
            a11y_id.request_focus();
            open.toggle();
            e.prevent_default();
            e.stop_propagation();
        };

        let on_pointer_enter = move |_| {
            *hovering.write() = true;
            Cursor::set(CursorIcon::Pointer);
        };

        let on_pointer_leave = move |_| {
            *hovering.write() = false;
            Cursor::set(CursorIcon::default());
        };

        let on_global_pointer_press = move |_: Event<PointerEventData>| {
            open.set_if_modified(false);
        };

        let on_global_key_down = move |e: Event<KeyboardEventData>| match e.key {
            Key::Named(NamedKey::Escape) => {
                open.set_if_modified(false);
            }
            Key::Named(NamedKey::Enter) if a11y_id.is_focused() => {
                open.toggle();
            }
            _ => {}
        };

        let (opacity, slide) = animation.read().value();

        let offset_y = match (button_area(), list_size()) {
            (button, Some(list)) => {
                let root_height = Platform::get().root_size.peek().height;
                let space_below = root_height - button.max_y();
                let space_above = button.min_y();
                let flips = list.height > space_below && list.height <= space_above;
                if flips {
                    -(button.height() + list.height) - slide
                } else {
                    slide
                }
            }
            _ => slide,
        };

        let opacity = if list_size().is_some() { opacity } else { 0. };

        let border_color = if open() {
            theme.md.primary
        } else if hovering() {
            theme.md.on_surface
        } else {
            theme.md.outline
        }
        .as_argb_u32();

        rect()
            .layout(self.layout.clone())
            .child(
                rect()
                    .child(
                        rect()
                            .a11y_id(a11y_id)
                            .a11y_member_of(a11y_id)
                            .a11y_role(AccessibilityRole::ListBox)
                            .a11y_focusable(Focusable::Enabled)
                            .on_pointer_enter(on_pointer_enter)
                            .on_pointer_leave(on_pointer_leave)
                            .on_press(on_press)
                            .on_global_key_down(on_global_key_down)
                            .on_global_pointer_press(on_global_pointer_press)
                            .on_sized(move |e: Event<SizedEventData>| {
                                button_area.set_if_modified(e.area);
                            })
                            .width(Size::Fill)
                            .background(theme.md.surface_container_highest.as_argb_u32())
                            .padding((8., 16.))
                            .corner_radius(CornerRadius::new_symmetric(4., 0.))
                            .horizontal()
                            .main_align(Alignment::SpaceBetween)
                            .cross_align(Alignment::Center)
                            .color(theme.md.on_surface.as_argb_u32())
                            .child(
                                rect()
                                    .child(
                                        label()
                                            .font_size(12.)
                                            .color(theme.md.on_surface_variant.as_argb_u32())
                                            .text(self.title.clone()),
                                    )
                                    .child(
                                        rect().font_size(16.).child((self.builder)(current_value)),
                                    ),
                            )
                            .child(
                                MaterialIcon::new(expand_more())
                                    .size(Size::px(24.))
                                    .color(theme.md.on_surface_variant.as_argb_u32()),
                            ),
                    )
                    .child(
                        rect()
                            .width(Size::Fill)
                            .height(Size::px(1.))
                            .background(border_color),
                    ),
            )
            .maybe_child((open() || opacity > 0.).then(|| {
                rect().height(Size::px(0.)).width(Size::px(0.)).child(
                    rect()
                        .width(Size::window_percent(100.))
                        .margin(Gaps::new(2., 0., 2., 0.))
                        .offset_y(offset_y)
                        .on_sized(move |e: Event<SizedEventData>| {
                            list_size.set_if_modified(Some(e.area.size));
                        })
                        .child(
                            rect()
                                .layer(Layer::Overlay)
                                .width(Size::px(button_area.read().width()))
                                .corner_radius(8.)
                                .background(theme.md.surface_container.as_argb_u32())
                                .opacity(opacity)
                                .shadow(Shadow::new().blur(8.).y(2.).color(0x33000000))
                                .child(
                                    ScrollView::new()
                                        .max_height(Size::window_percent(40.))
                                        .height(Size::Inner)
                                        .child(rect().padding((8., 0.)).children(
                                            self.options.iter().map(move |value| {
                                                DropdownItem::new(value.clone(), self.state.clone())
                                                    .child((self.builder)(value))
                                                    .into_element()
                                            }),
                                        )),
                                ),
                        ),
                )
            }))
    }
}

#[derive(PartialEq)]
pub struct DropdownItem<T: 'static> {
    value: T,
    state: Writable<T>,
    children: Vec<Element>,
}

impl<T> DropdownItem<T> {
    pub fn new(value: T, state: Writable<T>) -> Self {
        Self {
            value,
            state,
            children: Vec::new(),
        }
    }
}

impl<T> ChildrenExt for DropdownItem<T> {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl<T: Clone + PartialEq + 'static> Component for DropdownItem<T> {
    fn render(&self) -> impl IntoElement {
        StoatButton::new()
            .on_press({
                let value = self.value.clone();
                let state = self.state.clone();

                move |_| state.clone().set(value.clone())
            })
            .child(
                rect()
                    .width(Size::Fill)
                    .padding((0., 36.))
                    .height(Size::px(48.))
                    .main_align(Alignment::Center)
                    .cross_align(Alignment::Start)
                    .children(self.children.clone()),
            )
    }
}
