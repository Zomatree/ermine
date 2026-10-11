use freya::{
    prelude::*,
    radio::{Readable},
};

pub fn use_changed_from_readable<T: PartialEq + Clone + 'static>(
    value: Readable<T>,
    mut callback: impl FnMut(&T) + 'static,
) {
    let mut previous = use_state(|| value.read().clone());

    use_side_effect_with_deps(&*value.read(), move |after| {
        let changed = &*previous.peek() != after;

        if changed {
            previous.set(after.clone());
            callback(after)
        }
    });
}

pub fn use_changed<T: PartialEq + Clone + 'static>(
    value: impl IntoReadable<T>,
    callback: impl FnMut(&T) + 'static,
) {
    use_changed_from_readable(value.into_readable(), callback)
}

pub fn use_changed_with_previous_from_readable<T: PartialEq + Clone + 'static>(
    value: Readable<T>,
    mut callback: impl FnMut(T, &T) + 'static,
) {
    let mut previous = use_state(|| value.read().clone());

    use_side_effect_with_deps(&*value.read(), move |after| {
        let before = previous.peek();
        let changed = &*before != after;

        if changed {
            let before_value = before.cloned();
            drop(before);
            previous.set(after.clone());
            callback(before_value, after)
        }
    });
}

pub fn use_changed_with_previous<T: PartialEq + Clone + 'static>(
    value: impl IntoReadable<T>,
    callback: impl FnMut(T, &T) + 'static,
) {
    use_changed_with_previous_from_readable(value.into_readable(), callback)
}
