use freya::{
    animation::{
        AnimNum, AnimatedValue, Ease, Function, OnChange, OnCreation, ReadAnimatedValue,
        use_animation,
    },
    prelude::*,
};

use crate::{
    SizeExt,
    components::{MaterialIcon, material::outlined::color_lens},
    consume_material_theme,
};

#[derive(Clone, PartialEq)]
pub struct StoatColorPicker {
    value: Color,
    on_change: EventHandler<Color>,
    key: DiffKey,
}

impl KeyExt for StoatColorPicker {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl StoatColorPicker {
    pub fn new(value: Color, on_change: impl Into<EventHandler<Color>>) -> Self {
        Self {
            value,
            on_change: on_change.into(),
            key: DiffKey::None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
enum DragTarget {
    #[default]
    None,
    Sv,
    Hue,
}

impl Component for StoatColorPicker {
    fn render(&self) -> impl IntoElement {
        let mut open = use_state(|| false);
        let mut color = use_state(|| self.value);
        let mut dragging = use_state(DragTarget::default);
        let mut area = use_state(Area::default);
        let mut hue_area = use_state(Area::default);
        let theme = consume_material_theme();

        let is_open = open();

        let preview = rect()
            .width(Size::px(56.))
            .height(Size::px(56.))
            .corner_radius(16.)
            .background(theme.md.primary.as_argb_u32())
            .color(theme.md.on_primary.as_argb_u32())
            .center()
            .child(MaterialIcon::new(color_lens()).size(Size::px(24.)))
            .on_press(move |_| {
                open.toggle();
            });

        let hue_bar = rect()
            .height(Size::px(18.))
            .width(Size::fill())
            .corner_radius(4.)
            .on_sized(move |e: Event<SizedEventData>| hue_area.set(e.area))
            .background(
                LinearGradient::new()
                    .angle(-90.)
                    .stop(((255, 0, 0), 0.))
                    .stop(((255, 255, 0), 16.))
                    .stop(((0, 255, 0), 33.))
                    .stop(((0, 255, 255), 50.))
                    .stop(((0, 0, 255), 66.))
                    .stop(((255, 0, 255), 83.))
                    .stop(((255, 0, 0), 100.)),
            );

        let sv_area = rect()
            .height(Size::px(140.))
            .width(Size::fill())
            .corner_radius(4.)
            .overflow(Overflow::Clip)
            .child(
                rect()
                    .expanded()
                    .background(
                        // left: white -> right: hue color
                        LinearGradient::new()
                            .angle(-90.)
                            .stop(((255, 255, 255), 0.))
                            .stop((Color::from_hsv(color.read().to_hsv().h, 1.0, 1.0), 100.)),
                    )
                    .child(
                        rect()
                            .position(Position::new_absolute())
                            .expanded()
                            .background(
                                // top: transparent -> bottom: black
                                LinearGradient::new()
                                    .stop(((255, 255, 255, 0.0), 0.))
                                    .stop(((0, 0, 0), 100.)),
                            ),
                    ),
            );

        let mut update_sv = {
            let on_change = self.on_change.clone();
            move |coords: CursorPoint| {
                let sv_area = area.read().to_f64();
                let sat = ((coords.x - sv_area.min_x()) / sv_area.width()).clamp(0., 1.) as f32;
                let rel_y = ((coords.y - sv_area.min_y()) / sv_area.height()).clamp(0., 1.) as f32;
                let v = 1.0 - rel_y;
                let hsv = color.read().to_hsv();
                let new_color = Color::from_hsv(hsv.h, sat, v);
                color.set_if_modified_and_then(new_color, || on_change.call(new_color));
            }
        };

        let mut update_hue = {
            let on_change = self.on_change.clone();
            move |coords: CursorPoint| {
                let bar_area = hue_area.read().to_f64();
                let rel_x = ((coords.x - bar_area.min_x()) / bar_area.width()).clamp(0., 1.) as f32;
                let hsv = color.read().to_hsv();
                let new_color = Color::from_hsv(rel_x * 360.0, hsv.s, hsv.v);
                color.set_if_modified_and_then(new_color, || on_change.call(new_color));
            }
        };

        let on_sv_pointer_down = {
            let mut update_sv = update_sv.clone();
            move |e: Event<PointerEventData>| {
                dragging.set(DragTarget::Sv);
                update_sv(e.global_location());
                e.stop_propagation();
                e.prevent_default();
            }
        };

        let on_hue_pointer_down = {
            let mut update_hue = update_hue.clone();
            move |e: Event<PointerEventData>| {
                dragging.set(DragTarget::Hue);
                update_hue(e.global_location());
                e.stop_propagation();
                e.prevent_default();
            }
        };

        let on_global_pointer_move = move |e: Event<PointerEventData>| match *dragging.read() {
            DragTarget::Sv => {
                update_sv(e.global_location());
            }
            DragTarget::Hue => {
                update_hue(e.global_location());
            }
            DragTarget::None => {}
        };

        let on_global_pointer_press = move |_| {
            // Only close the popup if it wasnt being dragged and it is open
            if is_open && dragging() == DragTarget::None {
                open.set(false);
            }
            dragging.set_if_modified(DragTarget::None);
        };

        let animation = use_animation(move |conf| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Finish);

            let scale = AnimNum::new(0.8, 1.)
                .time(200)
                .ease(Ease::Out)
                .function(Function::Expo);
            let opacity = AnimNum::new(0., 1.)
                .time(200)
                .ease(Ease::Out)
                .function(Function::Expo);

            if open() {
                (scale, opacity)
            } else {
                (scale, opacity).into_reversed()
            }
        });

        let (scale, opacity) = animation.read().value();

        let popup = rect()
            .on_global_pointer_move(on_global_pointer_move)
            .on_global_pointer_press(on_global_pointer_press)
            .width(Size::px(220.))
            .padding(8.)
            .corner_radius(6.)
            .background(Color::WHITE)
            .border(
                Border::new()
                    .fill(Color::BLACK)
                    .width(1.)
                    .alignment(BorderAlignment::Inner),
            )
            .color(Color::BLACK)
            .spacing(8.)
            .shadow(Shadow::new().x(0.).y(2.).blur(8.).color((0, 0, 0, 0.1)))
            .child(
                rect()
                    .on_sized(move |e: Event<SizedEventData>| area.set(e.area))
                    .on_pointer_down(on_sv_pointer_down)
                    .child(sv_area),
            )
            .child(
                rect()
                    .height(Size::px(18.))
                    .on_pointer_down(on_hue_pointer_down)
                    .child(hue_bar),
            )
            .child({
                let hex = format!(
                    "#{:02X}{:02X}{:02X}",
                    color.read().r(),
                    color.read().g(),
                    color.read().b()
                );

                rect()
                    .horizontal()
                    .width(Size::fill())
                    .main_align(Alignment::center())
                    .spacing(8.)
                    .child(
                        Button::new()
                            .on_press(move |e: Event<PressEventData>| {
                                e.stop_propagation();
                                e.prevent_default();
                                if ContextMenu::is_open() {
                                    ContextMenu::close();
                                } else {
                                    ContextMenu::open(
                                        Menu::new()
                                            .child(
                                                MenuButton::new()
                                                    .on_press(move |e: Event<PressEventData>| {
                                                        e.stop_propagation();
                                                        e.prevent_default();
                                                        ContextMenu::close();
                                                        let _ =
                                                            Clipboard::set(color().to_rgb_string());
                                                    })
                                                    .child("Copy as RGB"),
                                            )
                                            .child(
                                                MenuButton::new()
                                                    .on_press(move |e: Event<PressEventData>| {
                                                        e.stop_propagation();
                                                        e.prevent_default();
                                                        ContextMenu::close();
                                                        let _ =
                                                            Clipboard::set(color().to_hex_string());
                                                    })
                                                    .child("Copy as HEX"),
                                            ),
                                    )
                                }
                            })
                            .compact()
                            .child(hex),
                    )
            });

        rect()
            .horizontal()
            .child(preview)
            .maybe_child((opacity > 0.).then(|| {
                rect()
                    .layer(Layer::Overlay)
                    .width(Size::px(0.))
                    .height(Size::px(0.))
                    .opacity(opacity)
                    .child(rect().scale(scale).child(popup).margin((0., 0., 0., 8.)))
            }))
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
