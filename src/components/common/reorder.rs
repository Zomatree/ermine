use std::rc::Rc;

use freya::prelude::*;

pub struct Reorder<T: 'static, B> {
    values: Writable<Vec<T>>,
    builder: Rc<B>,

    layout: LayoutData,
    style: StyleState,
}

impl<T, B: Fn(&T) -> Element + 'static> Reorder<T, B> {
    pub fn new(values: impl IntoWritable<Vec<T>>, builder: B) -> Self {
        Self {
            values: values.into_writable(),
            builder: Rc::new(builder),
            layout: LayoutData::default(),
            style: StyleState::default(),
        }
    }
}

impl<T, B> LayoutExt for Reorder<T, B> {
    fn get_layout(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl<T, B> ContainerExt for Reorder<T, B> {}

impl<T, B> ContainerWithContentExt for Reorder<T, B> {}

impl<T, B> PartialEq for Reorder<T, B> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T: Clone + PartialEq, B: Fn(&T) -> Element + 'static> Component for Reorder<T, B> {
    fn render(&self) -> impl IntoElement {
        let mut state = use_state(|| self.values.read().cloned());
        let mut selected = use_state(|| None);
        let mut hovered = use_state(|| None);

        let size = use_state(Size2D::default);

        let mut container = rect();
        *container.get_style() = self.style.clone();

        container.layout(self.layout.clone()).children(
            state
                .read()
                .cloned()
                .into_iter()
                .enumerate()
                .map(|(i, value)| {
                    rect()
                        .child(
                            DropZone::<T>::new(|_| {})
                                .child(
                                    DragZone::new(value.clone())
                                        .child(
                                            if hovered.read().as_ref().is_none_or(|(_, v)| {
                                                v != &value || selected.read().as_ref() != Some(v)
                                            }) {
                                                rect()
                                                    .interactive(selected.read().is_none())
                                                    .child((self.builder)(&value))
                                                    .into_element()
                                            } else {
                                                rect()
                                                    .width(Size::px(size.read().width))
                                                    .height(Size::px(size.read().height))
                                                    .into_element()
                                            },
                                        )
                                        .drag_element(
                                            rect()
                                                .on_sized({
                                                    let value = value.clone();

                                                    move |_| {
                                                        let is_dragging = selected.read().is_some();

                                                        if !is_dragging {
                                                            selected.set(Some(value.clone()));
                                                            hovered.set(Some((i, value.clone())));
                                                        }
                                                    }
                                                })
                                                .child(DraggedElement {
                                                    values: self.values.clone(),
                                                    state,
                                                    value,
                                                    builder: self.builder.clone(),
                                                    size,
                                                    selected,
                                                }),
                                        )
                                        .show_while_dragging(true)
                                        .into_element(),
                                )
                                .on_drag_over(move |hovering: bool| {
                                    if !hovering {
                                        return;
                                    };

                                    let current = hovered.read().cloned();
                                    let selected = selected.read().cloned();

                                    if let Some(current) = current
                                        && let Some(selected) = selected
                                    {
                                        state.write().remove(current.0);
                                        state.write().insert(i, current.1.clone());
                                        hovered.set(Some((i, selected)));
                                    }
                                }),
                        )
                        .into_element()
                }),
        )
    }
}

struct DraggedElement<T: 'static, B> {
    pub values: Writable<Vec<T>>,
    pub state: State<Vec<T>>,
    pub value: T,
    pub builder: Rc<B>,
    pub size: State<Size2D>,
    pub selected: State<Option<T>>,
}

impl<T: 'static, B> PartialEq for DraggedElement<T, B> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T: Clone + PartialEq, B: Fn(&T) -> Element + 'static> Component for DraggedElement<T, B> {
    fn render(&self) -> impl IntoElement {
        let mut values = self.values.clone();
        let state = self.state;
        let mut size = self.size;
        let mut selected = self.selected;

        use_drop(move || {
            selected.set(None);
            values.set(state.read().cloned());
        });

        rect()
            .cursor(CursorIcon::Pointer)
            .on_sized(move |area: Event<SizedEventData>| size.set_if_modified(area.area.size))
            .child((self.builder)(
                self.selected.read().as_ref().unwrap_or(&self.value),
            ))
    }
}
