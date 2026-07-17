use std::time::Duration;

use chumsky::container::Seq;
use freya::{
    animation::{AnimColor, AnimatedValue, Ease, OnChange, OnCreation, use_animation},
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;
use stoat_permissions::{ChannelPermission, PermissionValue};

use crate::{
    AppChannel, calculate_channel_permissions, components::{
        EmojiPicker, MaterialIcon, MessageContextMenu, MessageModel, ModalValue, ReplyController, StoatButton, StoatButtonColorsThemePartialExt, material::{outlined::{delete, edit, more_vert, reply}, round::insert_emoticon}, use_floating, use_modals
    }, consume_material_theme, http, theme::Theme, user_permissions_query
};

#[derive(PartialEq)]
pub struct MessageActions {
    pub children: Vec<Element>,
    pub layout: LayoutData,
    pub replies: ReplyController,
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
}

impl MessageActions {
    pub fn new(
        replies: ReplyController,
        channel: Readable<v0::Channel>,
        message: MessageModel,
    ) -> Self {
        Self {
            children: Vec::new(),
            layout: LayoutData::default(),
            replies,
            channel,
            message,
        }
    }
}

impl ChildrenExt for MessageActions {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl LayoutExt for MessageActions {
    fn get_layout(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl ContainerExt for MessageActions {}

impl Component for MessageActions {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let theme = consume_material_theme();
        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());

        let mut editing_message = radio.slice_mut(AppChannel::EditingMessage, |state| {
            &mut state.editing_message
        });
        let mut floating = use_floating();
        let mut modals = use_modals();

        let mut hovering = use_state(|| false);
        let mut hover_actions = use_state(|| false);

        let mentions_user = use_memo({
            let message = self.message.message.clone();
            let user_id = user_id.clone();

            move || {
                message
                    .mentions
                    .as_ref()
                    .is_some_and(|mentions| mentions.contains(&*user_id.read()))
            }
        });

        let change_background = use_reactive(&(hovering() || hover_actions()));

        let background = use_animation(move |conf| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Nothing);

            let theme = consume_material_theme();

            let start = if mentions_user() {
                theme.md.primary_container.as_argb_u32().into()
            } else {
                Color::TRANSPARENT
            };

            let anim = AnimColor::new(start, theme.md.surface_container.as_argb_u32())
                .duration(Duration::from_secs_f32(0.1))
                .ease(Ease::InOut);

            if change_background() {
                anim
            } else {
                anim.into_reversed()
            }
        });

        let permissions = use_state(|| PermissionValue::from_raw(0));

        use_side_effect({
            let radio = radio.clone();
            let channel = self.channel.clone();

            move || {
                let radio = radio.clone();
                let channel = channel.clone();

                spawn(async move {
                    let mut query =
                        user_permissions_query(radio.clone()).channel(channel.read().clone());

                    let value = calculate_channel_permissions(&mut query).await;
                    permissions.clone().set(value);
                });
            }
        });

        let mut shift = use_state(|| false);

        rect()
            .layout(self.layout.clone())
            .width(Size::Fill)
            .on_global_key_down(move |e: Event<KeyboardEventData>| {
                if e.key == Key::Named(NamedKey::Shift) {
                    shift.set(true);
                }
            })
            .on_global_key_up(move |e: Event<KeyboardEventData>| {
                if e.key == Key::Named(NamedKey::Shift) {
                    shift.set(false);
                }
            })
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .on_secondary_down({
                let message = self.message.clone();
                let replies = self.replies;

                move |e| {
                    ContextMenu::open_from_event(
                        &e,
                        Menu::new()
                        .child(MessageContextMenu {
                            message: message.clone(),
                            replies,
                            current_permissions: permissions()
                        })
                    );
                }
            })
            .corner_radius(12.)
            .children(self.children.clone())
            .background(background.read().value())
            .maybe_child((*hovering.read() || *hover_actions.read()).then(|| {
                rect()
                    .on_pointer_over(move |_| {
                        hover_actions.set(true);
                    })
                    .on_pointer_out(move |_| hover_actions.set_if_modified(false))
                    .position(Position::new_absolute().right(16.).top(-18.))
                    .corner_radius(4.)
                    .overflow(Overflow::Clip)
                    .horizontal()
                    .shadow(Shadow::new().blur(3.).color(Color::BLACK))
                    .layer(Layer::Relative(2))
                    .maybe_child(permissions.read().has_channel_permission(ChannelPermission::SendMessage).then(|| message_actions_button(reply(), &theme).on_press({
                        let mut replies = self.replies;
                        let message = self.message.clone();

                        move |_| {
                            replies.add_reply(message.clone(), true);
                        }
                    })))
                    .maybe_child(permissions.read().has_channel_permission(ChannelPermission::React).then(|| message_actions_button(insert_emoticon(), &theme).on_press({
                        let message_id = self.message.message.id.clone();
                        let channel_id = self.channel.peek().id().to_string();

                        move |_| {
                            floating.set(Some(
                                EmojiPicker::new({
                                    let message_id = message_id.clone();
                                    let channel_id = channel_id.clone();

                                    move |id: String| {
                                        let message_id = message_id.clone();
                                        let channel_id = channel_id.clone();
                                        floating.set(None);

                                        spawn_forever(async move {
                                            println!(
                                                "{:?}",
                                                http()
                                                    .react_message(&channel_id, &message_id, &id)
                                                    .await
                                            );
                                        });
                                    }
                                })
                                .into_element(),
                            ));
                        }
                    })))
                    .maybe_child((&self.message.message.author == &*user_id.read()).then(|| {
                        message_actions_button(edit(), &theme).on_press({
                            let message = self.message.message.clone();

                            move |_| {
                                *editing_message.write() = Some(crate::EditingMessage {
                                    id: message.id.clone(),
                                    content: message.content.clone().unwrap_or_default(),
                                });
                            }
                        })
                    }))
                    .maybe_child(
                        (&self.message.message.author == &*user_id.read()
                            || permissions
                                .read()
                                .has_channel_permission(ChannelPermission::ManageMessages))
                        .then(|| {
                            message_actions_button(delete(), &theme).on_press({
                                let message = self.message.message.id.clone();
                                let channel = self.message.message.channel.clone();

                                move |_| {
                                    let channel = channel.clone();
                                    let message = message.clone();

                                    if shift() {
                                        spawn(async move {
                                            http()
                                                .delete_message(&channel, &message)
                                                .await
                                                .unwrap();
                                        });
                                    } else {
                                        modals.write().push_modal(ModalValue::DeleteMessage {
                                            channel,
                                            message,
                                        });
                                    }
                                }
                            })
                        }),
                    )
                    .child(
                        message_actions_button(more_vert(), &theme).on_press({
                            let id = self.message.message.id.clone();

                            move |e| {
                                ContextMenu::open_from_event(
                                    &e,
                                    Menu::new().child(
                                        MenuButton::new()
                                            .child(label().font_size(14.).text("Copy Message ID"))
                                            .on_press({
                                                let id = id.clone();

                                                move |_| {
                                                    Clipboard::set(id.clone()).unwrap();
                                                }
                                            }),
                                    ),
                                );
                            }
                        }),
                    )
            }))
    }

    fn render_key(&self) -> DiffKey {
        (&self.message.message.id).into()
    }
}

pub fn message_actions_button(icon: Bytes, theme: &Theme) -> StoatButton {
    StoatButton::new()
        .child(
            rect().padding(4.).child(
                MaterialIcon::new(icon)
                    .color(theme.md.on_secondary_container.as_argb_u32())
                    .width(Size::px(20.))
                    .height(Size::px(20.)),
            ),
        )
        .background(theme.md.secondary_container.as_argb_u32())
}
