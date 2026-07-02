use bytes::Bytes;
use freya::{
    icons::lucide::{
        banknote, bot_message_square, circle_user_round, cloud, cpu, flag, flask_conical, globe,
        info, list, mail, message_square_diff, mic, palette, shield_check, smile, user_x,
    },
    prelude::State,
    radio::{RadioChannel, RadioStation},
};
// use livekit::{PlatformAudio, Room};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    rc::Rc,
    sync::Arc,
};

use stoat_models::v0::{
    AppendMessage, Channel, Emoji, FieldsChannel, FieldsMessage, FieldsServer, FieldsUser, Member,
    Message, PartialMessage, RelationshipStatus, Server, User, UserSettings,
};
use stoat_result::ErrorType;

use crate::{Config, components::SelectedRole, http, types::EventV1};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connected,
    Reconnecting,
    Reconnected,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Selection {
    #[default]
    Home,
    Server(String),
    Discover,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum SettingsPage {
    #[default]
    Account,
    Profile,
    Sessions,
    MyBots,
    Feedback,
    Voice,
    Appearance,
    Language,
    SourceCode,
    Advanced,
    Donate,
}

impl SettingsPage {
    pub fn title(&self) -> &'static str {
        match self {
            SettingsPage::Account => "My Account",
            SettingsPage::Profile => "Profile",
            SettingsPage::Sessions => "Sessions",
            SettingsPage::MyBots => "My Bots",
            SettingsPage::Feedback => "Feedback",
            SettingsPage::Voice => "Voice",
            SettingsPage::Appearance => "Appearance",
            SettingsPage::Language => "Language",
            SettingsPage::SourceCode => "Source Code",
            SettingsPage::Advanced => "Advanced",
            SettingsPage::Donate => "Donate",
        }
    }

    pub fn icon(&self) -> Bytes {
        match self {
            SettingsPage::Account => Bytes::new(),
            SettingsPage::Profile => circle_user_round(),
            SettingsPage::Sessions => shield_check(),
            SettingsPage::MyBots => bot_message_square(),
            SettingsPage::Feedback => message_square_diff(),
            SettingsPage::Voice => mic(),
            SettingsPage::Appearance => palette(),
            SettingsPage::Language => globe(),
            SettingsPage::SourceCode => cpu(),
            SettingsPage::Advanced => flask_conical(),
            SettingsPage::Donate => banknote(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum ServerSettingsPage {
    #[default]
    Overview,
    Emojis,
    Roles(Option<SelectedRole>),
    Invites,
    Bans,
}

impl ServerSettingsPage {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Emojis => "Emojis",
            Self::Roles(_) => "Roles",
            Self::Invites => "Invites",
            Self::Bans => "Bans",
        }
    }

    pub fn icon(&self) -> Bytes {
        match self {
            Self::Overview => info(),
            Self::Emojis => smile(),
            Self::Roles(_) => flag(),
            Self::Invites => mail(),
            Self::Bans => user_x(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ChannelSettingsPage {
    #[default]
    Overview,
    Permissions,
    Webhooks,
}

impl ChannelSettingsPage {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Permissions => "Permissions",
            Self::Webhooks => "Webhooks",
        }
    }

    pub fn icon(&self) -> Bytes {
        match self {
            Self::Overview => info(),
            Self::Permissions => list(),
            Self::Webhooks => cloud(),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct OrderingSettings {
    pub servers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationState {
    All,
    Mention,
    None,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct MuteState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<u128>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct NotificationsSettings {
    pub server: HashMap<String, NotificationState>,
    pub channel: HashMap<String, NotificationState>,
    pub server_mutes: HashMap<String, MuteState>,
    pub channel_mutes: HashMap<String, MuteState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationBadge {
    Unread,
    Mentions(usize),
}

#[derive(Debug, Default)]
pub struct SettingsState {
    pub ordering: Option<OrderingSettings>,
    pub notifications: Option<NotificationsSettings>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChannelUnread {
    pub last_id: Option<String>,
    pub mentions: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct Ready {
    pub events: bool,
    pub settings: bool,
}

impl Ready {
    pub fn is_ready(&self) -> bool {
        self.events && self.settings
    }
}

#[derive(Debug, Clone)]
pub struct ChannelState {
    pub messages: VecDeque<Message>,
    pub at_start: bool,
    pub at_end: bool,
    pub scroll_pos: Option<i32>,
}

#[derive(Clone)]
pub struct MessageHandlers {
    pub on_message: Rc<dyn Fn(Message)>,
    pub on_message_delete: Rc<dyn Fn(String, String)>,
    pub on_message_update: Rc<dyn Fn(String, String, PartialMessage, Vec<FieldsMessage>)>,
    pub on_message_react: Rc<dyn Fn(String, String, String, String)>,
    pub on_message_unreact: Rc<dyn Fn(String, String, String, String)>,
    pub on_message_remove_reaction: Rc<dyn Fn(String, String, String)>,
    pub on_message_append: Rc<dyn Fn(String, String, AppendMessage)>,
}

impl Debug for MessageHandlers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageHandlers").finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct EditingMessage {
    pub id: String,
    pub content: String,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub state: ConnectionState,
    pub ready: Ready,
    pub selection: Selection,
    pub selected_channel: Option<String>,
    pub user_id: Option<String>,
    pub users: HashMap<String, User>,
    pub servers: HashMap<String, Server>,
    pub members: HashMap<String, HashMap<String, Member>>,
    pub channels: HashMap<String, Channel>,
    pub emojis: HashMap<String, Emoji>,
    pub channel_states: HashMap<String, ChannelState>,
    pub channel_message_cache: HashMap<String, HashMap<String, Message>>,
    pub channel_unreads: HashMap<String, ChannelUnread>,
    pub settings_page: Option<SettingsPage>,
    pub user_profile: Option<String>,
    pub settings: SettingsState,
    pub message_handlers: Option<MessageHandlers>,
    pub editing_message: Option<EditingMessage>,
    pub server_settings_page: Option<(String, ServerSettingsPage)>,
    pub channel_settings_page: Option<(String, ChannelSettingsPage)>,
    // pub current_room: Option<(Arc<Room>, PlatformAudio)>,
}

impl AppState {
    pub fn new() -> Self {
        let mut this = Self::default();

        this.users.insert(
            "00000000000000000000000000".to_string(),
            User {
                id: "00000000000000000000000000".to_string(),
                username: "Stoat".to_string(),
                discriminator: "0000".to_string(),
                display_name: None,
                avatar: None,
                relations: Vec::new(),
                badges: 0,
                status: None,
                flags: 0,
                privileged: false,
                bot: None,
                relationship: RelationshipStatus::None,
                online: false,
            },
        );

        this
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum AppChannel {
    State,
    Selection,
    Ready,
    SelectedChannel,
    UserId,
    Users,
    Servers,
    Members,
    Channels,
    Emojis,
    ChannelStates,
    ChannelUnreads,
    ChannelMessageCache,
    SettingsPage,
    Settings(&'static str),
    ServerSettingsPage,
    UserProfile,
    MessageHandlers,
    EditingMessage,
    ChannelSettingsPage,
    CurrentRoom,
}

impl RadioChannel<AppState> for AppChannel {}

pub type AppStation = RadioStation<AppState, AppChannel>;

macro_rules! set_enum_varient_values {
    ($enum: ident, $key: ident, $value: expr, ($($varient: path),+)) => {
        match $enum {
            $($varient { $key, .. } )|+ => { *$key = $value },
            _ => {}
        }
    };
}

macro_rules! update_enum_partial {
    ($value: ident, $data: expr, $key: ident, ($($varient: path),+)) => {
        if let Some(new_value) = $data.$key {
            set_enum_varient_values!($value, $key, new_value, ($($varient),+))
        }
    };

    (optional, $value: ident, $data: expr, $key: ident, ($($varient: path),+)) => {
        set_enum_varient_values!($value, $key, $data.$key, ($($varient),+))
    };
}

macro_rules! update_multi_enum_partial {
    ($value: ident, $data: expr, ($( $( $(@$optional:tt)? optional )? ($key: ident, ($($varient: path),+))),+ $(,)?)) => {
        $(update_enum_partial!($( $($optional)? optional,)? $value, $data, $key, ($($varient),+)));+
    };
}

pub fn set_state(state: ConnectionState, mut station: AppStation) {
    station.write_channel(AppChannel::State).state = state;
}

pub fn set_current_user_id(user_id: String, mut station: AppStation) {
    station.write_channel(AppChannel::UserId).user_id = Some(user_id);
}

pub fn insert_user(user: User, mut station: AppStation) {
    station
        .write_channel(AppChannel::Users)
        .users
        .insert(user.id.clone(), user);
}

pub fn insert_server(server: Server, mut station: AppStation) {
    station
        .write_channel(AppChannel::Servers)
        .servers
        .insert(server.id.clone(), server);
}

pub fn insert_channel(channel: Channel, mut station: AppStation) {
    station
        .write_channel(AppChannel::Channels)
        .channels
        .insert(channel.id().to_string(), channel);
}

pub fn insert_message(message: Message, mut station: AppStation) {
    if let Some(channel_state) = station
        .write_channel(AppChannel::ChannelStates)
        .channel_states
        .get_mut(&message.channel)
    {
        if channel_state.at_end {
            channel_state.messages.push_front(message.clone());

            if channel_state.messages.len() > 50 {
                channel_state.messages.resize_with(50, || unreachable!());
            }
        }
    } else if let Some(handle) = &station.read().message_handlers {
        (handle.on_message)(message)
    }
}

pub fn insert_member(member: Member, mut station: AppStation) {
    station
        .write_channel(AppChannel::Members)
        .members
        .entry(member.id.server.clone())
        .or_default()
        .insert(member.id.user.clone(), member);
}

pub fn update_server(server_id: &str, mut station: AppStation, f: impl FnOnce(&mut Server)) {
    if let Some(server) = station
        .write_channel(AppChannel::Servers)
        .servers
        .get_mut(server_id)
    {
        f(server)
    }
}

pub fn update_channel(channel_id: &str, mut station: AppStation, f: impl FnOnce(&mut Channel)) {
    if let Some(channel) = station
        .write_channel(AppChannel::Channels)
        .channels
        .get_mut(channel_id)
    {
        f(channel)
    }
}

pub fn set_selection(selection: Selection, mut station: AppStation) {
    station.write_channel(AppChannel::Selection).selection = selection;
}

pub fn set_selected_channel(channel_id: Option<String>, mut station: AppStation) {
    station
        .write_channel(AppChannel::SelectedChannel)
        .selected_channel = channel_id;
}

pub fn update_settings(settings: UserSettings, mut station: AppStation) {
    for (key, (_ts, payload)) in settings.into_iter() {
        match key.as_str() {
            "ordering" => {
                if let Ok(value) = serde_json::from_str(&payload) {
                    station
                        .write_channel(AppChannel::Settings("ordering"))
                        .settings
                        .ordering = Some(value)
                }
            }
            "notifications" => {
                if let Ok(value) = Ok::<_, ()>(serde_json::from_str(&payload).unwrap()) {
                    station
                        .write_channel(AppChannel::Settings("notifications"))
                        .settings
                        .notifications = Some(value)
                }
            }
            _ => {}
        }
    }
}

pub fn insert_channel_unread(id: String, unread: ChannelUnread, mut station: AppStation) {
    station
        .write_channel(AppChannel::ChannelUnreads)
        .channel_unreads
        .insert(id, unread);
}

pub fn ack_message(channel_id: &str, message_id: String, mut station: AppStation) {
    if let Some(unread) = station
        .write_channel(AppChannel::ChannelUnreads)
        .channel_unreads
        .get_mut(channel_id)
    {
        unread.mentions.retain(|id| id > &message_id);
        unread.last_id = Some(message_id);
    }
}

pub fn update_user(user_id: &str, mut station: AppStation, f: impl FnOnce(&mut User)) {
    if let Some(user) = station
        .write_channel(AppChannel::Users)
        .users
        .get_mut(user_id)
    {
        f(user);
    }
}

pub fn insert_emoji(emoji: Emoji, mut station: AppStation) {
    station
        .write_channel(AppChannel::Emojis)
        .emojis
        .insert(emoji.id.clone(), emoji);
}

pub fn remove_emoji(emoji_id: &str, mut station: AppStation) {
    station
        .write_channel(AppChannel::Emojis)
        .emojis
        .remove(emoji_id);
}

pub async fn update_state(
    event: EventV1,
    mut config: State<Config>,
    mut station: RadioStation<AppState, AppChannel>,
) {
    match event {
        EventV1::Bulk { v } => {
            for e in v {
                Box::pin(update_state(e, config, station)).await;
            }
        }
        EventV1::Authenticated => {}
        EventV1::Logout => {
            config.write().token = None;
        }
        EventV1::Error { data } => match &data.error_type {
            ErrorType::InvalidSession => {
                config.write().token = None;
            }
            _ => {
                log::error!("Error: {data:?}")
            }
        },
        EventV1::Pong { data: _ } => {}
        EventV1::Ready {
            users,
            servers,
            channels,
            members,
            emojis,
            user_settings,
            channel_unreads,
            policy_changes: _,
            voice_states: _,
        } => {
            for user in users.into_iter().flatten() {
                if user.relationship == RelationshipStatus::User {
                    set_current_user_id(user.id.clone(), station);
                };

                insert_user(user, station);
            }

            for server in servers.into_iter().flatten() {
                insert_server(server, station);
            }

            for channel in channels.into_iter().flatten() {
                insert_channel(channel, station);
            }

            for member in members.into_iter().flatten() {
                insert_member(member, station);
            }

            for channel_unread in channel_unreads.into_iter().flatten() {
                insert_channel_unread(
                    channel_unread.id.channel,
                    ChannelUnread {
                        last_id: channel_unread.last_id,
                        mentions: channel_unread.mentions.into_iter().collect(),
                    },
                    station,
                );
            }

            if let Some(settings) = user_settings {
                update_settings(settings, station);
            }

            // for voice_state in voice_states.into_iter().flatten() {
            //     context.cache.insert_voice_state(voice_state);
            // }

            for emoji in emojis.into_iter().flatten() {
                insert_emoji(emoji, station);
            }

            {
                let mut state = station.write_channel(AppChannel::State);

                state.state = if state.state == ConnectionState::Reconnecting && state.ready.events
                {
                    ConnectionState::Reconnected
                } else {
                    ConnectionState::Connected
                };
            }

            station.write_channel(AppChannel::Ready).ready.events = true;
        }
        EventV1::Message(message) => {
            insert_message(message.clone(), station);

            update_channel(&message.channel, station, |channel| {
                if let Channel::TextChannel {
                    last_message_id, ..
                }
                | Channel::Group {
                    last_message_id, ..
                }
                | Channel::DirectMessage {
                    last_message_id, ..
                } = channel
                {
                    *last_message_id = Some(message.id.clone());
                }
            });

            if message
                .mentions
                .as_ref()
                .is_some_and(|m| m.contains(station.peek().user_id.as_ref().unwrap()))
            {
                station
                    .write_channel(AppChannel::ChannelUnreads)
                    .channel_unreads
                    .entry(message.channel.clone())
                    .or_default()
                    .mentions
                    .insert(message.id);
            }
        }
        EventV1::ServerCreate {
            id,
            server,
            channels,
            emojis,
            voice_states,
        } => {
            insert_server(server, station);

            for channel in channels.into_iter() {
                insert_channel(channel, station);
            }

            for emoji in emojis.into_iter() {
                insert_emoji(emoji, station);
            }
            let user_id = station.read().user_id.clone().unwrap();
            let member = http().fetch_member(&id, &user_id).await.unwrap();
            insert_member(member, station);
        }
        EventV1::ServerUpdate { id, data, clear } => {
            update_server(&id, station, |server| {
                server.apply_options(data);

                for field in &clear {
                    match field {
                        FieldsServer::Description => server.description = None,
                        FieldsServer::Categories => server.categories = None,
                        FieldsServer::SystemMessages => server.system_messages = None,
                        FieldsServer::Icon => server.icon = None,
                        FieldsServer::Banner => server.banner = None,
                    }
                }
            });
        }
        EventV1::ChannelUpdate { id, data, clear } => {
            update_channel(&id, station, |channel| {
                update_multi_enum_partial!(
                    channel,
                    data.clone(),
                    (
                        (name, (Channel::TextChannel)),
                        (owner, (Channel::Group)),
                        optional(description, (Channel::Group, Channel::TextChannel)),
                        optional(icon, (Channel::Group, Channel::TextChannel)),
                        (nsfw, (Channel::Group, Channel::TextChannel)),
                        (active, (Channel::DirectMessage)),
                        optional(permissions, (Channel::Group)),
                        (role_permissions, (Channel::TextChannel)),
                        optional(default_permissions, (Channel::TextChannel)),
                        optional(
                            last_message_id,
                            (Channel::DirectMessage, Channel::Group, Channel::TextChannel)
                        )
                    )
                );

                for field in &clear {
                    match field {
                        FieldsChannel::Description => set_enum_varient_values!(
                            channel,
                            description,
                            None,
                            (Channel::Group, Channel::TextChannel)
                        ),
                        FieldsChannel::Icon => set_enum_varient_values!(
                            channel,
                            icon,
                            None,
                            (Channel::Group, Channel::TextChannel)
                        ),
                        FieldsChannel::DefaultPermissions => set_enum_varient_values!(
                            channel,
                            default_permissions,
                            None,
                            (Channel::TextChannel)
                        ),
                        FieldsChannel::Voice => {
                            set_enum_varient_values!(channel, voice, None, (Channel::TextChannel))
                        }
                    }
                }
            });
        }
        EventV1::MessageUpdate {
            id,
            channel,
            data,
            clear,
        } => {
            if let Some(channel_state) = station
                .write_channel(AppChannel::ChannelStates)
                .channel_states
                .get_mut(&channel)
            {
                if let Some(message) = channel_state.messages.iter_mut().find(|m| m.id == id) {
                    message.apply_options(data.clone());

                    for field in &clear {
                        match field {
                            FieldsMessage::Pinned => message.pinned = None,
                        }
                    }
                }
            } else if let Some(handle) = &station.read().message_handlers {
                (handle.on_message_update)(channel, id, data, clear)
            }
        }
        EventV1::MessageDelete { id, channel } => {
            if let Some(channel_state) = station
                .write_channel(AppChannel::ChannelStates)
                .channel_states
                .get_mut(&channel)
            {
                channel_state.messages.retain(|m| m.id != id);
            } else if let Some(handle) = &station.read().message_handlers {
                (handle.on_message_delete)(channel, id)
            }
        }
        EventV1::MessageReact {
            id,
            channel_id,
            user_id,
            emoji_id,
        } => {
            if let Some(channel_state) = station
                .write_channel(AppChannel::ChannelStates)
                .channel_states
                .get_mut(&channel_id)
            {
                if let Some(message) = channel_state.messages.iter_mut().find(|m| m.id == id) {
                    message
                        .reactions
                        .entry(emoji_id.clone())
                        .or_default()
                        .insert(user_id.clone());
                }
            } else if let Some(handle) = &station.read().message_handlers {
                (handle.on_message_react)(channel_id, id, emoji_id, user_id)
            }
        }
        EventV1::MessageUnreact {
            id,
            channel_id,
            user_id,
            emoji_id,
        } => {
            if let Some(channel_state) = station
                .write_channel(AppChannel::ChannelStates)
                .channel_states
                .get_mut(&channel_id)
            {
                if let Some(message) = channel_state.messages.iter_mut().find(|m| m.id == id) {
                    if let Some(users) = message.reactions.get_mut(&emoji_id) {
                        users.swap_remove(&user_id);

                        if users.is_empty() {
                            message.reactions.swap_remove(&emoji_id);
                        };
                    }
                }
            } else if let Some(handle) = &station.read().message_handlers {
                (handle.on_message_unreact)(channel_id, id, emoji_id, user_id)
            }
        }
        EventV1::MessageRemoveReaction {
            id,
            channel_id,
            emoji_id,
        } => {
            if let Some(channel_state) = station
                .write_channel(AppChannel::ChannelStates)
                .channel_states
                .get_mut(&channel_id)
            {
                if let Some(message) = channel_state.messages.iter_mut().find(|m| m.id == id) {
                    message.reactions.swap_remove(&emoji_id);
                }
            } else if let Some(handle) = &station.read().message_handlers {
                (handle.on_message_remove_reaction)(channel_id, id, emoji_id)
            }
        }
        EventV1::ChannelCreate(channel) => {
            insert_channel(channel, station);
        }
        EventV1::UserSettingsUpdate { id: _id, update } => {
            update_settings(update, station);
        }
        EventV1::ChannelAck {
            id,
            user: _,
            message_id,
        } => {
            ack_message(&id, message_id, station);
        }
        EventV1::MessageAppend {
            id,
            channel,
            append,
        } => {
            if let Some(channel_state) = station
                .write_channel(AppChannel::ChannelStates)
                .channel_states
                .get_mut(&channel)
            {
                if let Some(message) = channel_state.messages.iter_mut().find(|m| m.id == id) {
                    if let Some(embeds) = append.embeds.clone() {
                        message.embeds.get_or_insert_default().extend(embeds);
                    }
                }
            } else if let Some(handle) = &station.read().message_handlers {
                (handle.on_message_append)(channel, id, append)
            }
        }
        EventV1::UserUpdate {
            id,
            data,
            clear,
            event_id: _,
        } => {
            update_user(&id, station, |user| {
                user.apply_options(data);

                for field in clear {
                    match field {
                        FieldsUser::Avatar => user.avatar = None,
                        FieldsUser::StatusText => {
                            if let Some(status) = &mut user.status {
                                status.text = None
                            }
                        }
                        FieldsUser::StatusPresence => {
                            if let Some(status) = &mut user.status {
                                status.presence = None
                            }
                        }
                        FieldsUser::ProfileContent => {}
                        FieldsUser::ProfileBackground => {}
                        FieldsUser::DisplayName => user.display_name = None,
                        FieldsUser::Internal => {}
                    }
                }
            });
        }
        EventV1::EmojiCreate(emoji) => {
            insert_emoji(emoji, station);
        }
        EventV1::EmojiDelete { id } => {
            remove_emoji(&id, station);
        }
        // EventV1::ChannelStartTyping { id, user } => {
        //     context
        //         .notifiers
        //         .invoke_typing_start_waiters(&(id.clone(), user.clone()))
        //         .await;

        //     handle_event!(handler, context, typing_start, (id, user))
        // }
        // EventV1::ChannelStopTyping { id, user } => {
        //     context
        //         .notifiers
        //         .invoke_typing_stop_waiters(&(id.clone(), user.clone()))
        //         .await;

        //     handle_event!(handler, context, typing_stop, (id, user))
        // }
        _ => {}
    }
}
