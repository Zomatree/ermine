use std::fmt::Display;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use stoat_models::v0;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CaptchaFeature {
    pub enabled: bool,
    pub key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Feature {
    pub enabled: bool,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct VoiceNode {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub public_url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct VoiceFeature {
    pub enabled: bool,
    pub nodes: Vec<VoiceNode>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct StoatFeatures {
    pub captcha: CaptchaFeature,
    pub email: bool,
    pub invite_only: bool,
    pub autumn: Feature,
    pub january: Feature,
    pub livekit: VoiceFeature,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BuildInformation {
    pub commit_sha: String,
    pub commit_timestamp: String,
    pub semver: String,
    pub origin_url: String,
    pub timestamp: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct StoatConfig {
    pub revolt: String,
    pub features: StoatFeatures,
    pub ws: String,
    pub app: String,
    pub vapid: String,
    pub build: BuildInformation,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct AutumnResponse {
    pub id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct RatelimitFailure {
    pub retry_after: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "lowercase")]
pub enum Tag {
    Attachments,
    Avatars,
    Backgrounds,
    Icons,
    Banners,
    Emojis,
}

impl Tag {
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    pub fn to_string(self) -> String {
        self.as_str().to_string()
    }
}

impl Into<&'static str> for Tag {
    fn into(self) -> &'static str {
        match self {
            Tag::Attachments => "attachments",
            Tag::Avatars => "avatars",
            Tag::Backgrounds => "backgrounds",
            Tag::Icons => "icons",
            Tag::Banners => "banners",
            Tag::Emojis => "emojis",
        }
    }
}

impl Into<String> for Tag {
    fn into(self) -> String {
        self.to_string()
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataLogin {
    Email {
        /// Email
        email: String,
        /// Password
        password: String,
        /// Friendly name used for the session
        friendly_name: Option<String>,
    },
    MFA {
        /// Unvalidated or authorised MFA ticket
        ///
        /// Used to resolve the correct account
        mfa_ticket: String,
        /// Valid MFA response
        ///
        /// This will take precedence over the `password` field where applicable
        mfa_response: Option<MFAResponse>,
        /// Friendly name used for the session
        friendly_name: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result")]
pub enum ResponseLogin {
    Success(Session),
    MFA {
        ticket: String,
        allowed_methods: Vec<MFAMethod>,
    },
    Disabled {
        user_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MFAResponse {
    Password { password: String },
    Recovery { recovery_code: String },
    Totp { totp_code: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    #[serde(rename = "_id")]
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MFAMethod {
    Password,
    Recovery,
    Totp,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Ping {
    Binary(Vec<u8>),
    Number(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventV1 {
    /// Multiple events
    Bulk {
        v: Vec<EventV1>,
    },
    /// Error event
    Error {
        data: stoat_result::Error,
    },

    /// Successfully authenticated
    Authenticated,
    /// Logged out
    Logout,
    /// Basic data to cache
    Ready {
        #[serde(skip_serializing_if = "Option::is_none")]
        users: Option<Vec<v0::User>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        servers: Option<Vec<v0::Server>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        channels: Option<Vec<v0::Channel>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        members: Option<Vec<v0::Member>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        emojis: Option<Vec<v0::Emoji>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        voice_states: Option<Vec<v0::ChannelVoiceState>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        user_settings: Option<v0::UserSettings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_unreads: Option<Vec<v0::ChannelUnread>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        policy_changes: Option<Vec<v0::PolicyChange>>,
    },

    /// Ping response
    Pong {
        data: Ping,
    },
    /// New message
    Message(v0::Message),

    /// Update existing message
    MessageUpdate {
        id: String,
        channel: String,
        data: v0::PartialMessage,
        #[serde(default)]
        clear: Vec<v0::FieldsMessage>,
    },

    /// Append information to existing message
    MessageAppend {
        id: String,
        channel: String,
        append: v0::AppendMessage,
    },

    /// Delete message
    MessageDelete {
        id: String,
        channel: String,
    },

    /// New reaction to a message
    MessageReact {
        id: String,
        channel_id: String,
        user_id: String,
        emoji_id: String,
    },

    /// Remove user's reaction from message
    MessageUnreact {
        id: String,
        channel_id: String,
        user_id: String,
        emoji_id: String,
    },

    /// Remove a reaction from message
    MessageRemoveReaction {
        id: String,
        channel_id: String,
        emoji_id: String,
    },

    /// Bulk delete messages
    BulkMessageDelete {
        channel: String,
        ids: Vec<String>,
    },

    /// New server
    ServerCreate {
        id: String,
        server: v0::Server,
        channels: Vec<v0::Channel>,
        emojis: Vec<v0::Emoji>,
        voice_states: Vec<v0::ChannelVoiceState>,
    },

    /// Update existing server
    ServerUpdate {
        id: String,
        data: v0::PartialServer,
        #[serde(default)]
        clear: Vec<v0::FieldsServer>,
    },

    /// Delete server
    ServerDelete {
        id: String,
    },

    /// Update existing server member
    ServerMemberUpdate {
        id: v0::MemberCompositeKey,
        data: v0::PartialMember,
        #[serde(default)]
        clear: Vec<v0::FieldsMember>,
    },

    /// User joins server
    ServerMemberJoin {
        id: String,
        // Deprecated: use member.id.user
        #[deprecated = "Use member.id.user instead"]
        user: String,
        member: v0::Member,
    },

    /// User left server
    ServerMemberLeave {
        id: String,
        user: String,
        reason: v0::RemovalIntention,
    },

    /// Server role created or updated
    ServerRoleUpdate {
        id: String,
        role_id: String,
        data: v0::PartialRole,
        #[serde(default)]
        clear: Vec<v0::FieldsRole>,
    },

    /// Server role deleted
    ServerRoleDelete {
        id: String,
        role_id: String,
    },

    /// Server roles ranks updated
    ServerRoleRanksUpdate {
        id: String,
        ranks: Vec<String>,
    },

    /// Update existing user
    UserUpdate {
        id: String,
        data: v0::PartialUser,
        #[serde(default)]
        clear: Vec<v0::FieldsUser>,
        event_id: Option<String>,
    },

    /// Relationship with another user changed
    UserRelationship {
        id: String,
        user: v0::User,
    },
    /// Settings updated remotely
    UserSettingsUpdate {
        id: String,
        update: v0::UserSettings,
    },

    /// User has been platform banned or deleted their account
    ///
    /// Clients should remove the following associated data:
    /// - Messages
    /// - DM Channels
    /// - Relationships
    /// - Server Memberships
    ///
    /// User flags are specified to explain why a wipe is occurring though not all reasons will necessarily ever appear.
    UserPlatformWipe {
        user_id: String,
        flags: i32,
    },
    /// New emoji
    EmojiCreate(v0::Emoji),

    /// Update existing emoji
    EmojiUpdate {
        id: String,
        data: v0::PartialEmoji,
    },

    /// Delete emoji
    EmojiDelete {
        id: String,
    },

    /// New report
    ReportCreate(v0::Report),
    /// New channel
    ChannelCreate(v0::Channel),

    /// Update existing channel
    ChannelUpdate {
        id: String,
        data: v0::PartialChannel,
        #[serde(default)]
        clear: Vec<v0::FieldsChannel>,
    },

    /// Delete channel
    ChannelDelete {
        id: String,
    },

    /// User joins a group
    ChannelGroupJoin {
        id: String,
        user: String,
    },

    /// User leaves a group
    ChannelGroupLeave {
        id: String,
        user: String,
    },

    /// User started typing in a channel
    ChannelStartTyping {
        id: String,
        user: String,
    },

    /// User stopped typing in a channel
    ChannelStopTyping {
        id: String,
        user: String,
    },

    /// User acknowledged message in channel
    ChannelAck {
        id: String,
        user: String,
        message_id: String,
    },

    /// New webhook
    WebhookCreate(v0::Webhook),

    /// Update existing webhook
    WebhookUpdate {
        id: String,
        data: v0::PartialWebhook,
        remove: Vec<v0::FieldsWebhook>,
    },

    /// Delete webhook
    WebhookDelete {
        id: String,
    },

    /// Voice events
    VoiceChannelJoin {
        id: String,
        state: v0::UserVoiceState,
    },
    VoiceChannelLeave {
        id: String,
        user: String,
    },
    VoiceChannelMove {
        user: String,
        from: String,
        to: String,
        state: v0::UserVoiceState,
    },
    UserVoiceStateUpdate {
        id: String,
        channel_id: String,
        data: v0::PartialUserVoiceState,
    },
    UserMoveVoiceChannel {
        node: String,
        from: String,
        to: String,
        token: String,
    },
    /// User's active slowmodes
    UserSlowmodes {
        slowmodes: Vec<v0::ChannelSlowmode>,
    },
    VoiceCallUpdate {
        initiator_id: String,
        channel_id: String,
        started_at: Option<Timestamp>,
        ended: bool,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Authenticate { token: String },
    BeginTyping { channel: String },
    EndTyping { channel: String },
    Subscribe { server_id: String },
    Ping { data: Ping },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Account {
    pub id: String,
    pub email: String,
}
