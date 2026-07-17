use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    fmt::Debug,
    rc::Rc,
    time::Duration,
};

use freya::{prelude::*, radio::use_radio};
use jiff::{Timestamp, tz::TimeZone};
use stoat_models::v0;
use tokio::time::sleep;

use crate::{
    AppChannel, ChannelState, EditingMessage,
    components::{
        ChannelTyping, Deferred, Message, MessageActions, MessageList, ReplyController,
        TrailingMessage,
    },
    http, map_readable,
    types::Tag,
};

#[derive(PartialEq)]
pub struct ChannelMessages {
    pub replies: ReplyController,
    pub channel: Readable<v0::Channel>,
    pub server: Option<Readable<v0::Server>>,
}

#[derive(Clone, PartialEq)]
pub struct MessageModel {
    pub message: v0::Message,
    pub user: Readable<v0::User>,
    pub member: Option<Readable<v0::Member>>,
}

impl Debug for MessageModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageModel")
            .field("message", &self.message)
            .field("user", &self.user.peek())
            .field("member", &self.member.as_ref().map(|m| m.peek()))
            .finish()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum FetchDirection {
    Initial,
    Upward,
    Downward,
    JumpEnd,
    JumpMsg,
}

/*
let radio = use_radio(AppChannel::Messages);

let messages_models = use_state(Vec::<MessageModel>::new);

*/

impl Component for ChannelMessages {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::ChannelStates);
        let user_id = radio.slice(AppChannel::UserId, |state| &state.user_id);
        let mut users_last_message = radio.slice_mut(AppChannel::UsersLastMessage, |state| {
            &mut state.users_last_message
        });

        let at_start = use_state(|| false);
        let at_end = use_state(|| false);
        let at_bottom = use_state(|| false);
        let fetching = use_state(|| None::<FetchDirection>);
        let failed = use_state(|| false);
        let messages = use_state(VecDeque::<v0::Message>::new);
        // let collected_messages = use_state(VecDeque::<v0::Message>::new);
        let preempt_fetch = use_state(|| None::<Rc<dyn Fn()>>);
        let preempted = use_state(|| false);
        let mut scroll_controller = use_scroll_controller(|| ScrollConfig {
            default_vertical_position: ScrollPosition::End,
            default_horizontal_position: ScrollPosition::End,
        });

        let mut ack_task = use_state(|| None::<TaskHandle>);
        let mut last_acked = use_state(|| None::<String>);

        use_side_effect({
            let messages = messages.clone();
            let platform = Platform::get();

            move || {
                let is_focused = (platform.is_app_focused)();
                let messages = messages.read();

                if at_bottom()
                    && is_focused
                    && let Some(message) = messages.front()
                    && last_acked.read().as_ref().is_none_or(|id| id < &message.id)
                {
                    let message_id = message.id.clone();
                    let channel_id = message.channel.clone();

                    drop(messages);

                    ack_task.take().map(|task| task.cancel());
                    last_acked.set(Some(message_id.clone()));

                    ack_task.set(Some(spawn(async move {
                        http().ack_channel(&channel_id, &message_id).await.unwrap();
                    })));
                }
            }
        });

        use_side_effect({
            let messages = messages.clone();
            let channel_id = self.channel.peek().id().to_string();

            let mut message_cache =
                radio.slice_mut(AppChannel::ChannelMessageCache, move |state| {
                    state
                        .channel_message_cache
                        .entry(channel_id.clone())
                        .or_default()
                });

            move || {
                let messages = { messages.read().cloned() };
                let mut cache = message_cache.write();

                for message in messages {
                    if !cache.contains_key(&message.id) {
                        cache.insert(message.id.clone(), message);
                    };
                }
            }
        });

        use_side_effect({
            let messages = messages.clone();
            let user_id = user_id.read().clone().unwrap();

            move || {
                let messages = messages.read();
                let mut found_message = None;

                if at_end() {
                    if let Some(message) = messages.iter().find(|m| m.author == user_id) {
                        found_message = Some(EditingMessage {
                            id: message.id.clone(),
                            content: message.content.clone().unwrap_or_default(),
                        });
                    }
                };

                *users_last_message.write() = found_message;
            }
        });

        let preempt = {
            let fetching = fetching.clone();
            let failed = failed.clone();
            let preempt_fetch = preempt_fetch.clone();

            move || {
                fetching.clone().set(None);
                failed.clone().set(false);

                if let Some(f) = &*preempt_fetch.read() {
                    f()
                }
            }
        };

        let new_preempted = {
            let preempted = preempted.clone();
            let preempt_fetch = preempt_fetch.clone();

            move || {
                preempted.clone().set(false);
                preempt_fetch.clone().set(Some(Rc::new({
                    // let mut preempted = preempted.clone();
                    move || preempted.clone().set(true)
                })));
            }
        };

        let can_fetch = {
            let fetching = fetching.clone();
            let failed = failed.clone();

            move || fetching.read().is_none() || !*failed.read()
        };

        let case_initial = {
            let channel = self.channel.clone();
            let server = self.server.clone();

            move |nearby: Option<String>| {
                let channel = channel.clone();
                let server = server.clone();

                spawn(async move {
                    let mut at_start = at_start.clone();
                    let mut at_end = at_end.clone();
                    // let mut collected_messages = collected_messages.clone();
                    let mut messages = messages.clone();
                    let mut fetching = fetching.clone();

                    preempt();
                    fetching.set(Some(FetchDirection::Initial));
                    new_preempted();
                    messages.write().clear();

                    at_start.set(false);
                    at_end.set(true);

                    // collected_messages.write().clear();
                    let channel = channel.read().clone();

                    let existing_state = radio.clone().write().channel_states.remove(channel.id());
                    let exists = existing_state.is_some();

                    let (existing_messages, existing_at_start, existing_at_end, scroll_pos) =
                        if let Some(state) = existing_state {
                            (
                                Some(state.messages),
                                Some(state.at_start),
                                Some(state.at_end),
                                state.scroll_pos,
                            )
                        } else {
                            (None, None, None, None)
                        };

                    let new_messages = if let Some(existing_messages) = existing_messages
                        && nearby.is_none()
                    {
                        existing_messages
                    } else {
                        let v0::BulkMessageResponse::MessagesAndUsers {
                            mut messages,
                            users,
                            members,
                        } = http()
                            .fetch_messages(
                                channel.id(),
                                &v0::OptionsQueryMessages {
                                    limit: Some(25),
                                    before: None,
                                    after: None,
                                    sort: None,
                                    nearby: None,
                                    include_users: Some(true),
                                },
                            )
                            .await
                            .unwrap()
                        else {
                            panic!()
                        };

                        {
                            let mut state = radio.clone().write_channel(AppChannel::Users);

                            for user in users {
                                state.users.entry(user.id.clone()).or_insert(user);
                            }
                        }

                        {
                            if let Some(members) = members
                                && let Some(server_id) = server.map(|s| s.read().id.clone())
                            {
                                let mut state = radio.clone().write_channel(AppChannel::Members);

                                let server_members = state.members.get_mut(&server_id).unwrap();

                                for member in members {
                                    server_members
                                        .entry(member.id.user.clone())
                                        .or_insert(member);
                                }
                            }
                        }

                        messages.sort_by(|a, b| b.id.cmp(&a.id));
                        messages.into()
                    };

                    if *preempted.read() {
                        return;
                    };

                    if nearby.is_some() {
                        match &channel {
                            v0::Channel::DirectMessage {
                                last_message_id, ..
                            }
                            | v0::Channel::Group {
                                last_message_id, ..
                            }
                            | v0::Channel::TextChannel {
                                last_message_id, ..
                            } => {
                                at_end.set(
                                    new_messages
                                        .iter()
                                        .any(|msg| Some(&msg.id) == last_message_id.as_ref()),
                                );
                            }
                            _ => {}
                        }
                    } else if !(exists && nearby.is_none()) && new_messages.len() < 25 {
                        at_start.set(true);
                    } else if let Some(existing_at_start) = existing_at_start
                        && let Some(existing_at_end) = existing_at_end
                    {
                        at_start.set(existing_at_start);
                        at_end.set(existing_at_end);
                    };

                    messages.set_if_modified(new_messages);

                    if let Some(scroll_pos) = scroll_pos
                        && let Some(at_end) = existing_at_end
                        && !at_end
                    {
                        spawn(async move {
                            // sleep(Duration::from_millis(100)).await;
                            scroll_controller.scroll_to_y(scroll_pos);
                        });
                    } else if *at_end.read() {
                        spawn(async move {
                            sleep(Duration::from_millis(100)).await;
                            scroll_controller.scroll_to(ScrollPosition::End, Direction::Vertical);
                        });
                    }

                    fetching.set(None);
                })
            }
        };

        // let upwards_task_running = use_state(|| None::<TaskHandle>);

        let case_fetch_upwards = {
            let channel = self.channel.clone();
            let fetching = fetching.clone();
            let mut at_start = at_start.clone();
            let mut at_end = at_end.clone();
            let can_fetch = can_fetch.clone();
            let new_preempted = new_preempted.clone();
            let mut messages = messages.clone();
            let server = self.server.clone();
            // let mut upwards_task_running = upwards_task_running.clone();

            move || {
                let channel = channel.clone();
                let mut fetching = fetching.clone();
                let server = server.clone();

                let task = spawn(async move {
                    if *at_start.read() || !can_fetch() {
                        return;
                    };

                    fetching.set(Some(FetchDirection::Upward));

                    new_preempted();

                    let channel = channel.read().clone();
                    let before_msg = if let Some(msg) = messages.read().back() {
                        msg.id.clone()
                    } else {
                        return;
                    };

                    let v0::BulkMessageResponse::MessagesAndUsers {
                        messages: new_messages,
                        users,
                        members,
                    } = http()
                        .fetch_messages(
                            channel.id(),
                            &v0::OptionsQueryMessages {
                                limit: Some(25),
                                before: Some(before_msg),
                                after: None,
                                sort: None,
                                nearby: None,
                                include_users: Some(true),
                            },
                        )
                        .await
                        .unwrap()
                    else {
                        panic!()
                    };

                    {
                        let mut state = radio.clone().write_channel(AppChannel::Users);

                        for user in users {
                            state.users.entry(user.id.clone()).or_insert(user);
                        }
                    }

                    {
                        if let Some(members) = members
                            && let Some(server_id) = server.map(|s| s.read().id.clone())
                        {
                            let mut state = radio.clone().write_channel(AppChannel::Members);

                            let server_members = state.members.get_mut(&server_id).unwrap();

                            for member in members {
                                server_members
                                    .entry(member.id.user.clone())
                                    .or_insert(member);
                            }
                        }
                    }

                    if *preempted.read() {
                        return;
                    };

                    if new_messages.len() < 25 {
                        at_start.set(true);
                    }

                    if !new_messages.is_empty() {
                        let cutoff =
                            (new_messages.len() + messages.read().len()).saturating_sub(75);

                        if cutoff > 0 {
                            at_end.set(false);
                        };

                        let mut existing = messages.write();
                        existing.extend(new_messages);
                        existing.make_contiguous().sort_by(|a, b| b.id.cmp(&a.id));

                        if cutoff > 0 {
                            existing.rotate_left(cutoff);
                            existing.resize_with(75, || unreachable!());
                        };
                    };

                    fetching.set(None);
                });

                // upwards_task_running.set(Some(task.clone()));

                task
            }
        };

        let case_fetch_downwards = {
            let channel = self.channel.clone();
            let fetching = fetching.clone();
            let mut at_start = at_start.clone();
            let mut at_end = at_end.clone();
            let can_fetch = can_fetch.clone();
            let new_preempted = new_preempted.clone();
            let mut messages = messages.clone();
            let server = self.server.clone();

            move || {
                let channel = channel.clone();
                let mut fetching = fetching.clone();
                let server = server.clone();

                spawn(async move {
                    if *at_end.read() || !can_fetch() {
                        return;
                    };

                    fetching.set(Some(FetchDirection::Downward));

                    new_preempted();

                    let channel = channel.read().clone();
                    let after_msg = if let Some(msg) = messages.read().front() {
                        msg.id.clone()
                    } else {
                        return;
                    };

                    let v0::BulkMessageResponse::MessagesAndUsers {
                        messages: new_messages,
                        users,
                        members,
                    } = http()
                        .fetch_messages(
                            channel.id(),
                            &v0::OptionsQueryMessages {
                                limit: Some(25),
                                before: None,
                                after: Some(after_msg),
                                sort: Some(v0::MessageSort::Oldest),
                                nearby: None,
                                include_users: Some(true),
                            },
                        )
                        .await
                        .unwrap()
                    else {
                        panic!()
                    };

                    {
                        let mut state = radio.clone().write_channel(AppChannel::Users);

                        for user in users {
                            state.users.entry(user.id.clone()).or_insert(user);
                        }
                    }

                    {
                        if let Some(members) = members
                            && let Some(server_id) = server.map(|s| s.read().id.clone())
                        {
                            let mut state = radio.clone().write_channel(AppChannel::Members);

                            let server_members = state.members.get_mut(&server_id).unwrap();

                            for member in members {
                                server_members
                                    .entry(member.id.user.clone())
                                    .or_insert(member);
                            }
                        }
                    }

                    if *preempted.read() {
                        return;
                    };

                    if new_messages.len() < 25 {
                        at_end.set(true);
                    };

                    if !new_messages.is_empty() {
                        let cutoff =
                            (new_messages.len() + messages.read().len()).saturating_sub(75);

                        if cutoff > 0 {
                            at_start.set(false);
                        };

                        let mut existing = messages.write();
                        existing.extend(new_messages);
                        existing.make_contiguous().sort_by(|a, b| b.id.cmp(&a.id));

                        if cutoff > 0 {
                            // existing.rotate_left(cutoff);
                            existing.resize_with(75, || unreachable!());
                        };
                    };

                    fetching.set(None);
                })
            }
        };

        use_hook({
            let channel = self.channel.clone();
            let case_initial = case_initial.clone();
            move || {
                let channel = channel.read().clone();
                let messages = messages.clone();
                let mut radio = radio.clone();

                radio
                    .write_channel(AppChannel::MessageHandlers)
                    .message_handlers = Some(crate::MessageHandlers {
                    on_message: Rc::new({
                        let messages = messages.clone();
                        let channel = channel.clone();
                        move |message| {
                            if channel.id() != &message.channel || !*at_end.read() {
                                return;
                            };

                            messages.clone().write().push_front(message);
                        }
                    }),
                    on_message_delete: Rc::new({
                        let messages = messages.clone();
                        let channel = channel.clone();
                        move |channel_id, message_id| {
                            if channel.id() != &channel_id {
                                return;
                            };

                            messages.clone().write().retain(|m| &m.id != &message_id);
                        }
                    }),
                    on_message_update: Rc::new({
                        let messages = messages.clone();
                        let channel = channel.clone();
                        move |channel_id, message_id, partial, remove| {
                            if channel.id() != &channel_id {
                                return;
                            };
                            if let Some(message) = messages
                                .clone()
                                .write()
                                .iter_mut()
                                .find(|m| m.id == message_id)
                            {
                                message.apply_options(partial);

                                for field in remove {
                                    match field {
                                        v0::FieldsMessage::Pinned => message.pinned = None,
                                    }
                                }
                            }
                        }
                    }),
                    on_message_react: Rc::new({
                        let messages = messages.clone();
                        let channel = channel.clone();
                        move |channel_id, message_id, emoji_id, user_id| {
                            if channel.id() != &channel_id {
                                return;
                            };

                            if let Some(message) = messages
                                .clone()
                                .write()
                                .iter_mut()
                                .find(|m| m.id == message_id)
                            {
                                message
                                    .reactions
                                    .entry(emoji_id.clone())
                                    .or_default()
                                    .insert(user_id.clone());
                            }
                        }
                    }),
                    on_message_unreact: Rc::new({
                        let messages = messages.clone();
                        let channel = channel.clone();
                        move |channel_id, message_id, emoji_id, user_id| {
                            if channel.id() != &channel_id {
                                return;
                            };

                            if let Some(message) = messages
                                .clone()
                                .write()
                                .iter_mut()
                                .find(|m| m.id == message_id)
                            {
                                if let Some(users) = message.reactions.get_mut(&emoji_id) {
                                    users.swap_remove(&user_id);

                                    if users.is_empty() {
                                        message.reactions.swap_remove(&emoji_id);
                                    };
                                }
                            }
                        }
                    }),
                    on_message_remove_reaction: Rc::new({
                        let messages = messages.clone();
                        let channel = channel.clone();
                        move |channel_id, message_id, emoji_id| {
                            if channel.id() != &channel_id {
                                return;
                            };

                            if let Some(message) = messages
                                .clone()
                                .write()
                                .iter_mut()
                                .find(|m| m.id == message_id)
                            {
                                message.reactions.swap_remove(&emoji_id);
                            }
                        }
                    }),
                    on_message_append: Rc::new({
                        let messages = messages.clone();
                        let channel = channel.clone();
                        move |channel_id, message_id, append| {
                            if channel.id() != &channel_id {
                                return;
                            };

                            if let Some(message) = messages
                                .clone()
                                .write()
                                .iter_mut()
                                .find(|m| m.id == message_id)
                            {
                                if let Some(embeds) = append.embeds.clone() {
                                    message.embeds.get_or_insert_default().extend(embeds);
                                }
                            }
                        }
                    }),
                    on_bulk_message_delete: Rc::new({
                        let messages = messages.clone();
                        let channel = channel.clone();
                        move |channel_id, message_ids| {
                            if channel.id() != &channel_id {
                                return;
                            };

                            messages
                                .clone()
                                .write()
                                .retain(|m| !message_ids.contains(&m.id));
                        }
                    }),
                });

                case_initial(None);
            }
        });

        use_drop({
            let fetching = fetching.clone();
            let channel = self.channel.clone();
            let messages = messages.clone();
            let at_start = at_start.clone();
            let at_end = at_end.clone();

            move || {
                if *fetching.read() != Some(FetchDirection::Initial) {
                    let channel_id = channel.read().id().to_string();
                    let (_, y): (i32, i32) = scroll_controller.into();

                    let state = ChannelState {
                        messages: messages.read().clone(),
                        at_start: *at_start.read(),
                        at_end: *at_end.read(),
                        scroll_pos: Some(y),
                    };

                    radio
                        .clone()
                        .write()
                        .channel_states
                        .insert(channel_id, state);
                }
            }
        });

        let mut user_radio = use_radio(AppChannel::Users);
        let mut member_radio = use_radio(AppChannel::Members);

        let messages_models = use_state(Vec::<MessageModel>::new);
        let task_running = use_state(|| None::<TaskHandle>);

        {
            let server = self.server.clone();
            let mut task_running = task_running.clone();
            let messages = messages.clone();

            use_side_effect(move || {
                let messages = messages.clone();
                let server = server.clone();
                let mut messages_models = messages_models.clone();

                let _ = messages.read();

                task_running.set(Some(spawn(async move {
                    let mut new_message_models = Vec::new();
                    let messages = messages.read().clone();

                    for message in messages {
                        if let Some(user) = message.user.clone() {
                            let users = &mut user_radio.write().users;

                            if !users.contains_key(&user.id) {
                                users.insert(user.id.clone(), user);
                            }
                        } else {
                            let exists = user_radio.read().users.contains_key(&message.author);

                            if !exists {
                                let user = if let Some(webhook) = &message.webhook {
                                    v0::User {
                                        id: message.author.clone(),
                                        username: webhook.name.clone(),
                                        discriminator: "0000".to_string(),
                                        display_name: None,
                                        avatar: webhook.avatar.clone().map(|id| v0::File {
                                            id,
                                            tag: Tag::Avatars.into(),
                                            filename: String::new(),
                                            metadata: v0::Metadata::File,
                                            content_type: String::new(),
                                            size: 0,
                                            deleted: None,
                                            reported: None,
                                            message_id: None,
                                            user_id: None,
                                            server_id: None,
                                            object_id: None,
                                        }),
                                        relations: Vec::new(),
                                        badges: 0,
                                        status: None,
                                        flags: 0,
                                        privileged: false,
                                        bot: None,
                                        relationship: v0::RelationshipStatus::None,
                                        online: false,
                                        pronouns: None,
                                    }
                                } else if let Ok(user) = http().fetch_user(&message.author).await {
                                    user
                                } else {
                                    v0::User {
                                        id: message.author.clone(),
                                        username: format!("Unknown User {}", message.author),
                                        discriminator: "0000".to_string(),
                                        display_name: None,
                                        avatar: None,
                                        relations: Vec::new(),
                                        badges: 0,
                                        status: None,
                                        flags: 0,
                                        privileged: false,
                                        bot: None,
                                        relationship: v0::RelationshipStatus::None,
                                        online: false,
                                        pronouns: None,
                                    }
                                };

                                user_radio.write().users.insert(user.id.clone(), user);
                            };
                        };

                        let user = user_radio
                            .slice_current({
                                let user_id = message.author.clone();
                                move |state| state.users.get(&user_id).unwrap()
                            })
                            .into_readable();

                        let member = if let Some(member) = message.member.clone()
                            && let Some(server) = server.clone()
                        {
                            member_radio
                                .write()
                                .members
                                .entry(member.id.server.clone())
                                .or_default()
                                .insert(member.id.user.clone(), member);

                            let members: Readable<HashMap<String, v0::Member>> = member_radio
                                .slice_current(move |state| {
                                    state.members.get(&server.read().id).unwrap()
                                })
                                .into_readable();

                            Some(map_readable(members, {
                                let user_id = message.author.clone();
                                move |members| members.get(&user_id).unwrap()
                            }))
                        } else if message.author.as_str() != "00000000000000000000000000"
                            && message.webhook.is_none()
                            && let Some(server) = server.clone()
                        {
                            let exists = member_radio
                                .read()
                                .members
                                .get(&server.read().id)
                                .is_some_and(|members| members.contains_key(&message.author));

                            let fetched = if !exists {
                                let server_id = server.read().id.clone();

                                if let Some(member) = http()
                                    .fetch_member(&server_id, &message.author.clone())
                                    .await
                                    .ok()
                                {
                                    member_radio
                                        .write()
                                        .members
                                        .entry(member.id.server.clone())
                                        .or_default()
                                        .insert(member.id.user.clone(), member);

                                    true
                                } else {
                                    false
                                }
                            } else {
                                true
                            };

                            if fetched {
                                let members: Readable<HashMap<String, v0::Member>> = member_radio
                                    .slice_current(move |state| {
                                        state.members.get(&server.read().id).unwrap()
                                    })
                                    .into_readable();

                                Some(map_readable(members, {
                                    let user_id = message.author.clone();
                                    move |members| members.get(&user_id).unwrap()
                                }))
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        new_message_models.push(MessageModel {
                            message,
                            user,
                            member,
                            // replies,
                        });
                    }

                    messages_models.set_if_modified(new_message_models);
                })));
            });
        };

        let groups = {
            let messages_models = messages_models.clone();

            use_memo(move || {
                let mut groups: Vec<Vec<MessageModel>> = Vec::new();

                for model in messages_models.read().iter().cloned().rev() {
                    if let Some(group) = groups.last_mut() {
                        let last = group.last().unwrap();

                        if last.user.peek().id != model.user.peek().id
                            || model
                                .message
                                .replies
                                .as_ref()
                                .is_some_and(|r| !r.is_empty())
                        {
                            groups.push(vec![model]);
                            continue;
                        };

                        let last_datetime = Timestamp::try_from(
                            ulid::Ulid::from_string(&last.message.id)
                                .unwrap()
                                .datetime(),
                        )
                        .unwrap()
                        .to_zoned(TimeZone::system());

                        let current_datetime = Timestamp::try_from(
                            ulid::Ulid::from_string(&model.message.id)
                                .unwrap()
                                .datetime(),
                        )
                        .unwrap()
                        .to_zoned(TimeZone::system());

                        let diff = (current_datetime.timestamp().as_second()
                            - last_datetime.timestamp().as_second())
                        .abs();

                        if last_datetime.date() != current_datetime.date() || diff >= 420 {
                            groups.push(vec![model]);
                            continue;
                        }

                        let last_msg = &last.message;

                        if last_msg.system.is_some() {
                            groups.push(vec![model]);
                            continue;
                        }

                        let current_msg = &model.message;

                        if current_msg.system.is_some()
                            || current_msg.masquerade != last_msg.masquerade
                        {
                            groups.push(vec![model]);
                            continue;
                        }

                        group.push(model);
                    } else {
                        groups.push(vec![model]);
                    }
                }

                groups
            })
        };

        let message_views = use_memo({
            let replies = self.replies.clone();
            let channel = self.channel.clone();
            move || {
                groups
                    .read()
                    .iter()
                    .cloned()
                    .map(|messages| {
                        let first = messages.first().unwrap();

                        let mut elements = vec![
                            MessageActions::new(replies, channel.clone(), first.clone())
                                .margin((12., 0., 0., 0.))
                                .padding((2., 0.))
                                .child(Message {
                                    channel: channel.clone(),
                                    message: first.clone(),
                                })
                                .into_element(),
                        ];

                        for message in &messages[1..] {
                            elements.push(
                                MessageActions::new(replies, channel.clone(), message.clone())
                                    .margin((0., 0., 0., 0.))
                                    .padding((2., 0.))
                                    .child(TrailingMessage {
                                        channel: channel.clone(),
                                        message: message.clone(),
                                    })
                                    .into_element(),
                            );
                        }
                        elements
                    })
                    .flatten()
                    .collect::<Vec<_>>()
            }
        });

        let permit_fetching =
            map_readable::<Option<FetchDirection>, bool>(fetching.into_readable(), |fetching| {
                if fetching.is_none() { &true } else { &false }
            });

        rect().padding((0., 8.)).child(
            MessageList::new(
                move |_| {
                    case_fetch_upwards();
                },
                move |_| {
                    case_fetch_downwards();
                },
                at_start.clone().into_readable(),
                at_end.into_readable(),
                at_bottom.into_writable(),
                permit_fetching,
                scroll_controller,
            )
            .child(
                Deferred::new().child(
                    rect()
                        .padding((16., 0., 0., 0.))
                        .maybe_child(at_start.read().then(|| {
                            rect()
                                .margin((18., 16., 10., 16.))
                                .child(label().font_size(32.).line_height(1.5).text(
                                    match &*self.channel.read() {
                                        v0::Channel::DirectMessage { recipients, .. } => {
                                            let user_id =
                                                radio.peek_state().user_id.clone().unwrap();

                                            let other = recipients
                                                .iter()
                                                .find(|&id| id != &*user_id)
                                                .unwrap()
                                                .clone();

                                            let user = radio
                                                .slice(AppChannel::Users, move |state| {
                                                    state.users.get(&other).unwrap()
                                                });

                                            Cow::Owned(user.read().username.clone())
                                        }
                                        v0::Channel::Group { name, .. }
                                        | v0::Channel::TextChannel { name, .. } => {
                                            Cow::Owned(name.clone())
                                        }
                                        v0::Channel::SavedMessages { .. } => {
                                            Cow::Borrowed("Saved Messages")
                                        }
                                    },
                                ))
                                .child(
                                    label()
                                        .font_size(16.)
                                        .font_weight(550)
                                        .line_height(1.5)
                                        .text("This is the start of your conversation."),
                                )
                        }))
                        .child(rect().children(message_views.read().iter().cloned()))
                        .child(ChannelTyping {
                            channel: self.channel.clone(),
                            server: self.server.clone(),
                        }),
                ),
            ),
        )
    }

    fn render_key(&self) -> DiffKey {
        (&self.channel.peek().id().to_string()).into()
    }
}
