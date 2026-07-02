use std::{cell::RefCell, rc::Rc};

use freya::prelude::*;

use crate::{
    components::{
        ChannelDescriptionModal, CreateJoinServerModal, CreateRoleModal, CreateServerModal,
        DeleteMessageModal, JoinServerModal, ServerInfoModal, StoatButton,
        StoatButtonLayoutThemePartialExt,
    },
    use_material_theme,
};

#[derive(PartialEq, Clone, Debug)]
pub enum ModalValue {
    ServerInfo { server: String },
    CreateJoinServer,
    CreateServer,
    JoinServer,
    ChannelDescription { channel: String },
    CreateRole { server: String },
    DeleteMessage { channel: String, message: String },
}

#[derive(Clone)]
pub struct ModalController {
    modal: Option<ModalValue>,
}

impl ModalController {
    pub fn push_modal(&mut self, modal: ModalValue) {
        self.modal = Some(modal);
    }

    pub fn pop_modal(&mut self) -> Option<ModalValue> {
        self.modal.take()
    }

    pub fn get_modal(&self) -> Option<ModalValue> {
        self.modal.clone()
    }
}

pub fn use_modals() -> State<ModalController> {
    use_hook(|| {
        match try_consume_root_context() {
            Some(state) => state,
            None => {
                let state = State::create_global(ModalController { modal: None });
                provide_root_context(state);
                state
            }
        }
    })
}

#[derive(PartialEq)]
pub struct ModalManager {}

impl Component for ModalManager {
    fn render(&self) -> impl IntoElement {
        let controller = use_modals();

        let modal = controller.read().get_modal();

        rect().layer(Layer::RelativeOverlay(7)).maybe_child(modal.map(|value| {
            Modal {
                value: value.clone(),
            }
            .into_element()
        }))
    }
}

#[derive(PartialEq)]
struct Modal {
    value: ModalValue,
}

impl Component for Modal {
    fn render(&self) -> impl IntoElement {
        let theme = use_material_theme();
        let mut controller = use_modals();

        let on_global_key_down = move |e: Event<KeyboardEventData>| {
            if e.key == Key::Named(NamedKey::Escape) {
                controller.write().pop_modal();
            }
        };

        rect().position(Position::new_global()).child(
            rect()
                .child(
                    rect()
                        .on_press(move |_| {
                            controller.write().pop_modal();
                        })
                        .position(Position::new_global().top(0.).left(0.))
                        .height(Size::window_percent(100.))
                        .width(Size::window_percent(100.))
                        .background(0x99000000),
                )
                .child(
                    rect()
                        .position(Position::new_global().top(0.).left(0.))
                        .height(Size::window_percent(100.))
                        .width(Size::window_percent(100.))
                        .center()
                        .child(
                            rect()
                                .a11y_role(AccessibilityRole::Dialog)
                                .corner_radius(28.)
                                .background(theme.md.surface_container_high.as_argb_u32())
                                .color(theme.md.on_surface.as_argb_u32())
                                .min_width(Size::px(280.))
                                .max_width(Size::px(560.))
                                .padding(24.)
                                .on_global_key_down(on_global_key_down)
                                .child(match self.value.clone() {
                                    ModalValue::ServerInfo { server } => {
                                        ServerInfoModal { server }.into_element()
                                    }
                                    ModalValue::CreateJoinServer => {
                                        CreateJoinServerModal {}.into_element()
                                    }
                                    ModalValue::CreateServer => CreateServerModal {}.into_element(),
                                    ModalValue::JoinServer => JoinServerModal {}.into_element(),
                                    ModalValue::ChannelDescription { channel } => {
                                        ChannelDescriptionModal { channel }.into_element()
                                    }
                                    ModalValue::CreateRole { server } => {
                                        CreateRoleModal { server }.into_element()
                                    }
                                    ModalValue::DeleteMessage { channel, message } => {
                                        DeleteMessageModal { channel, message }.into_element()
                                    }
                                }),
                        ),
                ),
        )
    }
}

#[derive(Clone)]
pub struct DialogAction(Rc<RefCell<dyn FnMut() -> bool + 'static>>);

impl DialogAction {
    pub fn new(handler: impl FnMut() -> bool + 'static) -> Self {
        Self(Rc::new(RefCell::new(handler)))
    }

    pub fn call(&self) -> bool {
        (self.0.borrow_mut())()
    }
}

impl<H: FnMut() -> bool + 'static> From<H> for DialogAction {
    fn from(value: H) -> Self {
        DialogAction::new(value)
    }
}

impl PartialEq for DialogAction {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(PartialEq)]
pub struct Dialog {
    title: Vec<Element>,
    body: Vec<Element>,
    actions: Vec<(&'static str, Option<DialogAction>)>,
}

impl Dialog {
    pub fn new() -> Self {
        Self {
            title: Vec::new(),
            body: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn title(mut self, title: impl Into<Element>) -> Self {
        self.title.push(title.into());

        self
    }

    pub fn body(mut self, body: impl Into<Element>) -> Self {
        self.body.push(body.into());

        self
    }

    pub fn default_action(mut self, title: &'static str) -> Self {
        self.actions.push((title, None));

        self
    }

    pub fn action(mut self, title: &'static str, callback: impl Into<DialogAction>) -> Self {
        self.actions.push((title, Some(callback.into())));

        self
    }
}

impl Component for Dialog {
    fn render(&self) -> impl IntoElement {
        let theme = use_material_theme();
        let mut controller = use_modals();

        rect()
            .child(
                rect()
                    .font_size(24.)
                    .margin((0., 0., 16., 0.))
                    .children(self.title.clone()),
            )
            .child(
                rect()
                    .color(theme.md.on_surface_variant.as_argb_u32())
                    .font_size(14.)
                    .children(self.body.clone()),
            )
            .child(
                rect()
                    .margin((24., 0., 0., 0.))
                    .horizontal()
                    .width(Size::Fill)
                    .spacing(8.)
                    .main_align(Alignment::End)
                    .children(self.actions.iter().cloned().map(|(title, callback)| {
                        StoatButton::new()
                            .corner_radius(20.)
                            .on_press(move |_| {
                                if let Some(callback) = &callback {
                                    if callback.call() {
                                        controller.write().pop_modal();
                                    }
                                } else {
                                    controller.write().pop_modal();
                                }
                            })
                            .child(
                                rect()
                                    .padding((0., 16.))
                                    .height(Size::px(40.))
                                    .center()
                                    .child(
                                        label()
                                            .color(theme.md.primary.as_argb_u32())
                                            .font_size(14.)
                                            .text(title),
                                    ),
                            )
                            .into_element()
                    })),
            )
    }
}
