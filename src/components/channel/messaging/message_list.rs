use std::ops::Not;

use freya::prelude::*;

#[derive(PartialEq)]
pub struct MessageList {
    pub children: Vec<Element>,
    pub on_top: EventHandler<()>,
    pub on_bottom: EventHandler<()>,
    pub at_start: Readable<bool>,
    pub at_end: Readable<bool>,
    pub at_bottom: Writable<bool>,
    pub permit_fetching: Readable<bool>,
    pub controller: ScrollController,
}

impl MessageList {
    pub fn new(
        on_top: impl Into<EventHandler<()>>,
        on_bottom: impl Into<EventHandler<()>>,
        at_start: Readable<bool>,
        at_end: Readable<bool>,
        at_bottom: Writable<bool>,
        permit_fetching: Readable<bool>,
        controller: ScrollController,
    ) -> Self {
        Self {
            children: Vec::new(),
            on_top: on_top.into(),
            on_bottom: on_bottom.into(),
            at_start,
            at_end,
            at_bottom,
            permit_fetching,
            controller,
        }
    }
}

impl ChildrenExt for MessageList {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl Component for MessageList {
    fn render(&self) -> impl IntoElement {
        let mut list_viewport = use_state(Area::default);
        let mut top_viewport = use_state(Area::default);
        let mut bottom_viewport = use_state(Area::default);

        let mut at_top = use_state(|| false);
        let mut at_bottom = use_state(|| false);
        let mut autoscroll = use_state(|| false);

        use_side_effect_with_deps(&self.children, {
            let mut controller = self.controller.clone();

            move |_| {
                if autoscroll() {
                    controller.scroll_to(ScrollPosition::End, Direction::Vertical);
                }
            }
        });

        rect()
            .on_sized(move |e: Event<SizedEventData>| list_viewport.set_if_modified(e.area))
            .child(
                ScrollView::new_controlled(self.controller)
                    .max_height(Size::Fill)
                    .height(Size::Inner)
                    .maybe_child(self.at_start.read().not().then(|| {
                        rect()
                            .key("top")
                            .on_sized({
                                let on_top = self.on_top.clone();

                                move |e: Event<SizedEventData>| {
                                    top_viewport.set_if_modified(e.area);

                                    if list_viewport.read().intersects(&e.visible_area) {
                                        at_top.set_if_modified_and_then(true, || on_top.call(()));
                                    } else {
                                        at_top.set_if_modified(false);
                                    }
                                }
                            })
                            .height(Size::px(100.))
                            .child("top")
                    }))
                    .children(self.children.clone())
                    .maybe_child(self.at_end.read().not().then(|| {
                        rect()
                            .key("bottom")
                            .on_sized({
                                let on_bottom = self.on_bottom.clone();

                                move |e: Event<SizedEventData>| {
                                    bottom_viewport.set_if_modified(e.area);
                                    if list_viewport.read().intersects(&e.visible_area) {
                                        at_bottom
                                            .set_if_modified_and_then(true, || on_bottom.call(()));
                                    } else {
                                        at_bottom.set_if_modified(false);
                                    }
                                }
                            })
                            .height(Size::px(100.))
                            .child("bottom")
                    }))
                    .child(rect().height(Size::px(1.)).width(Size::px(1.)).on_sized({
                        let mut at_bottom = self.at_bottom.clone();

                        move |e: Event<SizedEventData>| {
                            let is_at_bottom = list_viewport.read().intersects(&e.visible_area);

                            autoscroll.set(is_at_bottom);
                            at_bottom.set(is_at_bottom);
                        }
                    })),
            )
    }
}
