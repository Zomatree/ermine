use freya::prelude::*;

#[derive(PartialEq)]
pub struct FloatingManager {}

impl Component for FloatingManager {
    fn render(&self) -> impl IntoElement {
        let mut mouse_pos = use_state(CursorPoint::default);
        let element = use_floating();

        rect()
            .layer(Layer::OverlayLevel(7))
            .maybe_child(element.read().cloned().map(|element| Floating {
                element,
                mouse_pos: *mouse_pos.read(),
            }))
            .on_capture_global_pointer_move(move |e: Event<PointerEventData>| {
                mouse_pos.set(e.global_location());
            })
    }
}

pub fn use_floating() -> State<Option<Element>> {
    use_hook(|| match try_consume_root_context() {
        Some(state) => state,
        None => {
            let state = State::create_global(None);
            provide_root_context(state);
            state
        }
    })
}

#[derive(PartialEq)]
struct Floating {
    pub element: Element,
    pub mouse_pos: CursorPoint,
}

impl Component for Floating {
    fn render(&self) -> impl IntoElement {
        let mouse_pos = use_hook(|| self.mouse_pos);
        // let mut show = use_state(|| false);
        let mut area = use_state(Area::default);
        let mut pos = use_state(|| (0., 0.));

        let mut floating = use_floating();

        use_side_effect({
            let area = area.clone();

            move || {
                let area = *area.read();

                if area.is_empty() || *pos.read() != (0., 0.) {
                    return;
                };

                Platform::get().with_window(None, move |window| {
                    let window_size = window.inner_size();

                    let x = if (mouse_pos.x as u32 + area.size.width as u32) > window_size.width {
                        (mouse_pos.x as f32 - area.size.width)
                            .min(window_size.width as f32 - area.size.width)
                            - 24.
                    } else {
                        mouse_pos.x as f32 + 24.
                    };

                    let y = if (mouse_pos.y as u32 + area.size.height as u32)
                        > window_size.height - 32
                    {
                        mouse_pos.y as f32 - area.size.height + 16.
                    } else {
                        mouse_pos.y as f32 - 16.
                    }
                    .max(16.)
                    .min(window_size.height as f32 - area.size.height - 65.);

                    pos.set((x, y));
                });
            }
        });

        let (left, top) = *pos.read();

        rect()
            .position(Position::new_global().left(left).top(top))
            .layer(Layer::Overlay)
            .opacity(if left != 0. && top != 0. { 100. } else { 0. })
            .on_sized(move |e: Event<SizedEventData>| area.set(e.area))
            .on_global_pointer_down(move |e: Event<PointerEventData>| {
                if !area.read().to_f64().contains(e.global_location()) {
                    floating.set(None);
                }
            })
            .child(self.element.clone())
    }
}
