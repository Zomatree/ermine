use freya::prelude::*;

#[derive(PartialEq)]
pub struct Deferred {
    child: Element,
}

impl Deferred {
    pub fn new() -> Self {
        Self {
            child: rect().into_element(),
        }
    }

    pub fn child<C: IntoElement>(mut self, child: C) -> Self {
        self.child = child.into_element();

        self
    }
}

impl Component for Deferred {
    fn render(&self) -> impl IntoElement {
        let mut render = use_state(|| false);

        let value = *render.read();

        if !value {
            render.set(true);

            rect().into_element()
        } else {
            // if value == Some(false) {
            //     render.set(Some(true))
            // };

            self.child.clone()
        }
    }
}
