use freya::{
    animation::{
        AnimNum, AnimatedValue, Ease, Function, OnChange, OnCreation,
        use_animation_with_dependencies,
    },
    prelude::*,
};

pub fn stoat_scrollbar(context: ScrollBarContext) -> Element {
    let ScrollBarContext {
        axis,
        timeout,
        clicking_scrollbar,
        thumb_events,
        thumb_offset,
        track_size,
        thumb_length,
        ..
    } = context;
    let mut hovering = use_state(|| false);
    let is_hidden = timeout.elapsed() && clicking_scrollbar.read().is_none() && !hovering();
    let is_expanded = clicking_scrollbar.read().is_some() || hovering();

    let expand_animation =
        use_animation_with_dependencies(&is_expanded, move |conf, is_expanded| {
            conf.on_creation(OnCreation::Finish);
            conf.on_change(OnChange::Rerun);

            let cross_size = AnimNum::new(7., 11.)
                .time(207)
                .function(Function::Expo)
                .ease(Ease::Out);

            if *is_expanded {
                cross_size
            } else {
                cross_size.into_reversed()
            }
        });
    let cross_size = expand_animation.get().value();

    let hidden_animation = use_animation_with_dependencies(&is_hidden, move |conf, is_hidden| {
        conf.on_creation(OnCreation::Finish);
        conf.on_change(OnChange::Rerun);

        let hidden = AnimNum::new(0., 1.)
            .time(300)
            .function(Function::Expo)
            .ease(Ease::Out);

        if *is_hidden {
            hidden.into_reversed()
        } else {
            hidden
        }
    });

    let opacity = hidden_animation.get().value();

    let (width, height, offset_x, offset_y, thumb_offset_x, thumb_offset_y) = match axis {
        Axis::X => (
            track_size,
            Size::px(cross_size),
            0.,
            -cross_size,
            thumb_offset,
            0.,
        ),
        Axis::Y => (
            Size::px(cross_size),
            track_size,
            -cross_size,
            0.,
            0.,
            thumb_offset,
        ),
    };

    rect()
        .position(Position::new_absolute())
        .opacity(opacity)
        .width(width.clone())
        .height(height.clone())
        .offset_x(offset_x)
        .offset_y(offset_y)
        .layer(999)
        .cursor(CursorIcon::Default)
        .on_pointer_down(|e: Event<PointerEventData>| {
            e.stop_propagation();
        })
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .direction(if axis == Axis::Y {
                    Direction::vertical()
                } else {
                    Direction::horizontal()
                })
                .cross_align(Alignment::end())
                .child(
                    rect()
                        .position(Position::new_absolute())
                        .width(width)
                        .height(height)
                        .offset_x(thumb_offset_x)
                        .offset_y(thumb_offset_y)
                        .on_pointer_over(move |_| {
                            hovering.set_if_modified(true);
                        })
                        .on_pointer_out(move |_| {
                            hovering.set_if_modified(false);
                        })
                        .child(ScrollThumb {
                            theme: None,
                            clicking_scrollbar,
                            thumb_events,
                            axis,
                            size: thumb_length,
                            cross_size,
                            cross_gap: 1.,
                        }),
                ),
        )
        .into_element()
}
