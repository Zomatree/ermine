use std::{cell::RefCell, rc::Rc, time::Duration};

use freya::{
    animation::{
        AnimNum, AnimatedValue, Ease, Function, OnChange, OnCreation, OnFinish,
        use_animation_with_dependencies,
    },
    prelude::*,
};
use stoat_models::v0;

use crate::{
    Error,
    components::{
        StoatButton, StoatButtonLayoutThemePartialExt,
        modals::{
            ChannelDescription, CreateBot, CreateJoinServer, CreateRole, CreateServer, DeleteBot,
            DeleteCategory, DeleteChannel, DeleteInvite, DeleteMessage, EditApi,
            EditOwnServerIdentity, EditRoles, ErrorModal, ImageViewer, InviteBot, InviteInfo,
            JoinServer, LeaveGroup, LeaveServer, LogoutOtherSessions, MFA, OpenLink,
            RenameCategory, ResetBotToken, ServerInfo,
        },
    },
    consume_material_theme,
};

#[derive(PartialEq, Clone)]
pub enum ModalValue {
    ServerInfo {
        server: String,
    },
    CreateJoinServer,
    CreateServer,
    JoinServer,
    ChannelDescription {
        channel: String,
    },
    CreateRole {
        server: String,
    },
    DeleteMessage {
        channel: String,
        message: String,
    },
    DeleteInvite {
        invite: String,
    },
    LeaveServer {
        server: String,
    },
    RenameCategory {
        server: String,
        category: String,
    },
    DeleteCategory {
        server: String,
        category: String,
    },
    InviteInfo {
        code: String,
    },
    LeaveGroup {
        channel: String,
    },
    DeleteChannel {
        channel: String,
    },
    OpenLink {
        url: String,
    },
    EditApi,
    EditOwnServerIdentity {
        server: String,
    },
    LogoutOtherSessions {
        callback: EventHandler<()>,
    },
    MFA {
        callback: EventHandler<v0::MFATicket>,
    },
    CreateBot {
        callback: EventHandler<v0::BotWithUserResponse>,
    },
    ResetBotToken {
        id: String,
        name: String,
        callback: EventHandler<v0::BotWithUserResponse>,
    },
    DeleteBot {
        id: String,
        name: String,
        callback: EventHandler<bool>,
    },
    InviteBot {
        bot: v0::PublicBot,
    },
    ImageViewer(Url),
    EditRoles {
        user: String,
        server: String,
    },
    Error {
        error: Error,
    },
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
    use_hook(|| match try_consume_root_context() {
        Some(state) => state,
        None => {
            let state = State::create_global(ModalController { modal: None });
            provide_root_context(state);
            state
        }
    })
}

#[derive(PartialEq)]
pub struct ModalManager {}

impl Component for ModalManager {
    fn render(&self) -> impl IntoElement {
        let controller = use_modals();

        let modal = controller.read().get_modal();
        let mut last_modal = use_state(|| modal.clone());

        let animation = use_animation_with_dependencies(&modal, move |anim, modal| {
            anim.on_creation(OnCreation::Run);
            anim.on_change(OnChange::Rerun);
            anim.on_finish(OnFinish::Nothing);

            let num = AnimNum::new(0., 1.)
                .duration(Duration::from_millis(200))
                .ease(Ease::Out)
                .function(Function::Expo);

            if modal.is_some() {
                num
            } else {
                num.into_reversed()
            }
        });

        use_side_effect(move || {
            let modal = controller.read().get_modal();

            if modal.is_some() {
                last_modal.set(modal);
            } else if animation.read().value() == 0. {
                last_modal.set(None);
            };
        });

        let opacity = animation.get().value();

        rect()
            .layer(Layer::OverlayLevel(8))
            .opacity(opacity)
            .maybe_child(
                modal
                    .or(last_modal.read().cloned())
                    .map(|value| Modal { value }.into_element()),
            )
    }
}

#[derive(PartialEq)]
struct Modal {
    value: ModalValue,
}

impl Component for Modal {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
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
                                .color(theme.md.on_surface.as_argb_u32())
                                .on_global_key_down(on_global_key_down)
                                .child(match self.value.clone() {
                                    ModalValue::ServerInfo { server } => {
                                        ServerInfo { server }.into_element()
                                    }
                                    ModalValue::CreateJoinServer => {
                                        CreateJoinServer {}.into_element()
                                    }
                                    ModalValue::CreateServer => CreateServer {}.into_element(),
                                    ModalValue::JoinServer => JoinServer {}.into_element(),
                                    ModalValue::ChannelDescription { channel } => {
                                        ChannelDescription { channel }.into_element()
                                    }
                                    ModalValue::CreateRole { server } => {
                                        CreateRole { server }.into_element()
                                    }
                                    ModalValue::DeleteMessage { channel, message } => {
                                        DeleteMessage { channel, message }.into_element()
                                    }
                                    ModalValue::DeleteInvite { invite } => {
                                        DeleteInvite { invite }.into_element()
                                    }
                                    ModalValue::LeaveServer { server } => {
                                        LeaveServer { server }.into_element()
                                    }
                                    ModalValue::RenameCategory { server, category } => {
                                        RenameCategory { server, category }.into_element()
                                    }
                                    ModalValue::DeleteCategory { server, category } => {
                                        DeleteCategory { server, category }.into_element()
                                    }
                                    ModalValue::Error { error } => {
                                        ErrorModal { error }.into_element()
                                    }
                                    ModalValue::InviteInfo { code } => {
                                        InviteInfo { code }.into_element()
                                    }
                                    ModalValue::LeaveGroup { channel } => {
                                        LeaveGroup { channel }.into_element()
                                    }
                                    ModalValue::DeleteChannel { channel } => {
                                        DeleteChannel { channel }.into_element()
                                    }
                                    ModalValue::OpenLink { url } => OpenLink { url }.into_element(),
                                    ModalValue::EditApi => EditApi {}.into_element(),
                                    ModalValue::EditOwnServerIdentity { server } => {
                                        EditOwnServerIdentity { server }.into_element()
                                    }
                                    ModalValue::LogoutOtherSessions { callback } => {
                                        LogoutOtherSessions { callback }.into_element()
                                    }
                                    ModalValue::MFA { callback } => MFA { callback }.into_element(),
                                    ModalValue::CreateBot { callback } => {
                                        CreateBot { callback }.into_element()
                                    }
                                    ModalValue::ResetBotToken { id, name, callback } => {
                                        ResetBotToken { id, name, callback }.into_element()
                                    }
                                    ModalValue::DeleteBot { id, name, callback } => {
                                        DeleteBot { id, name, callback }.into_element()
                                    }
                                    ModalValue::InviteBot { bot } => {
                                        InviteBot { bot }.into_element()
                                    }
                                    ModalValue::ImageViewer(content) => {
                                        ImageViewer { content }.into_element()
                                    }
                                    ModalValue::EditRoles { user, server } => {
                                        EditRoles { user, server }.into_element()
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
        let theme = consume_material_theme();
        let mut controller = use_modals();

        rect()
            .min_width(Size::px(232.))
            .max_width(Size::px(512.))
            .corner_radius(28.)
            .background(theme.md.surface_container_high.as_argb_u32())
            .padding(24.)
            // .margin(80.)
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
                    .on_global_key_down({
                        let last_action = self.actions.last().cloned();

                        move |e: Event<KeyboardEventData>| {
                            if e.key == Key::Named(NamedKey::Enter) && e.modifiers.is_empty() {
                                if let Some((_, callback)) = &last_action {
                                    if let Some(callback) = &callback {
                                        if callback.call() {
                                            controller.write().pop_modal();
                                        }
                                    } else {
                                        controller.write().pop_modal();
                                    }
                                }
                            }
                        }
                    })
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
