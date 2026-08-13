use std::{
    hash::{DefaultHasher, Hasher},
    sync::{Arc, OnceLock, RwLock},
    time::Duration,
};

use bytes::Bytes;
use reqwest::{
    Body, Client, Method, Request, RequestBuilder, Response,
    multipart::{Form, Part},
};
use scc::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_string};
use stoat_models::v0::{
    AllMemberResponse, AuditLogQueryResponse, BanListResult, BulkMessageResponse, Channel,
    ChannelUnread, CreateServerLegacyResponse, CreateVoiceUserResponse, CreateWebhookBody,
    DataBanCreate, DataCreateEmoji, DataCreateRole, DataCreateServer, DataCreateServerChannel,
    DataDefaultChannelPermissions, DataEditChannel, DataEditMessage, DataEditRole,
    DataEditRoleRanks, DataEditServer, DataEditUser, DataEditWebhook, DataJoinCall, DataMemberEdit,
    DataMessageSearch, DataMessageSend, DataSendFriendRequest, DataSetRolePermissions,
    DataSetServerRolePermission, Emoji, FetchServerResponse, FlagResponse, Invite,
    InviteJoinResponse, MFAResponse, MFATicket, Member, Message, MultiFactorStatus, MutualResponse,
    NewRoleResponse, OptionsAuditLogQuery, OptionsBulkDelete, OptionsFetchAllMembers,
    OptionsFetchServer, OptionsFetchSettings, OptionsQueryMessages, OptionsServerDelete,
    OptionsUnreact, ResponseWebhook, Role, Server, ServerBan, SessionInfo, User, UserProfile,
    UserSettings, Webhook,
};
use stoat_permissions::DataPermissionsValue;
use tokio::time::sleep;

use crate::{
    Session,
    error::{Error, Result},
    types::{AutumnResponse, DataLogin, ResponseLogin, StoatConfig},
};

pub static HTTP: OnceLock<HttpClient> = OnceLock::new();

pub fn http() -> HttpClient {
    HTTP.get().expect("http client is unset").clone()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RatelimitEntry {
    pub remaining: u32,
    pub reset: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Service {
    Api,
    Autumn,
}

pub struct LocalFile {
    pub name: String,
    pub body: Body,
}

/// Handles all HTTP requests to the Stoat api.
///
/// All api requests are automatically handled for rate-limits and will pause executing the request until the ratelimit is over.
#[derive(Clone, Debug)]
pub struct HttpClient {
    pub base: String,
    pub api_config: Arc<StoatConfig>,
    pub session: Arc<RwLock<Option<Session>>>,
    pub inner: Client,
    pub ratelimits: Arc<HashMap<u64, RatelimitEntry>>,
}

impl AsRef<HttpClient> for HttpClient {
    fn as_ref(&self) -> &HttpClient {
        self
    }
}

impl AsRef<StoatConfig> for HttpClient {
    fn as_ref(&self) -> &StoatConfig {
        &self.api_config
    }
}

impl AsRef<StoatConfig> for StoatConfig {
    fn as_ref(&self) -> &StoatConfig {
        self
    }
}

impl HttpClient {
    pub async fn new(base: String, session: Option<Session>) -> Result<Self> {
        let client = Client::new();
        let ratelimits = Arc::new(HashMap::new());

        let api_config = HttpRequest {
            ratelimits: ratelimits.clone(),
            service: Service::Api,
            builder: client.get(&base),
            query: None,
        }
        .response()
        .await?;

        Ok(HttpClient {
            base,
            api_config: Arc::new(api_config),
            session: Arc::new(RwLock::new(session)),
            inner: client,
            ratelimits,
        })
    }

    /// Creates a raw http request for a specific method and route.
    pub fn request(&self, method: Method, route: impl AsRef<str>) -> HttpRequest {
        let mut builder = self
            .inner
            .request(method, format!("{}{}", &self.base, route.as_ref()))
            .header("Accept", "application/json");

        if let Some(session) = &*self.session.read().unwrap() {
            builder = builder.header("x-session-token", session.token.clone());
        }

        HttpRequest {
            ratelimits: self.ratelimits.clone(),
            service: Service::Api,
            builder,
            query: None,
        }
    }

    /// Creates a raw autumn http request for a specific method and route.
    pub fn autumn_request(&self, method: Method, route: impl AsRef<str>) -> HttpRequest {
        let mut builder = self
            .inner
            .request(
                method,
                format!("{}{}", &self.api_config.features.autumn.url, route.as_ref()),
            )
            .header("Accept", "application/json");

        if let Some(session) = &*self.session.read().unwrap() {
            builder = builder.header("x-session-token", session.token.clone());
        }

        HttpRequest {
            ratelimits: self.ratelimits.clone(),
            service: Service::Autumn,
            builder,
            query: None,
        }
    }

    /// Formats a url string for a file hosted on Stoat.
    pub fn format_file_url(&self, tag: &str, id: &str, filename: Option<&str>) -> String {
        let autumn_url = &self.api_config.features.autumn.url;

        let mut url = format!("{autumn_url}/{}/{}", tag, id);

        if let Some(filename) = filename {
            url.push('/');
            url.push_str(filename);
        };

        url
    }

    pub fn format_default_avatar_url(&self, user_id: &str) -> String {
        format!("{}/users/{user_id}/default_avatar", &self.base)
    }

    pub async fn get_root(&self) -> Result<StoatConfig> {
        self.request(Method::GET, "/").response().await
    }

    pub async fn login(&self, data: &DataLogin) -> Result<ResponseLogin> {
        self.request(Method::POST, "/auth/session/login")
            .body(data)
            .response()
            .await
    }

    pub async fn send_message(&self, channel_id: &str, data: &DataMessageSend) -> Result<Message> {
        self.request(Method::POST, format!("/channels/{}/messages", channel_id))
            .body(data)
            .response()
            .await
    }

    pub async fn fetch_user(&self, user_id: &str) -> Result<User> {
        self.request(Method::GET, format!("/users/{user_id}"))
            .response()
            .await
    }

    pub async fn fetch_messages<'a>(
        &self,
        channel_id: &str,
        data: &OptionsQueryMessages,
    ) -> Result<BulkMessageResponse> {
        self.request(Method::GET, format!("/channels/{}/messages", channel_id))
            .query(data)
            .response()
            .await
    }

    pub async fn open_dm(&self, user_id: &str) -> Result<Channel> {
        self.request(Method::GET, format!("/users/{user_id}/dm"))
            .response()
            .await
    }

    pub async fn fetch_member(&self, server_id: &str, user_id: &str) -> Result<Member> {
        self.request(
            Method::GET,
            format!("/servers/{server_id}/members/{user_id}"),
        )
        .response()
        .await
    }

    pub async fn delete_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.request(
            Method::DELETE,
            format!("/channels/{channel_id}/messages/{message_id}"),
        )
        .send()
        .await
    }

    pub async fn edit_message(
        &self,
        channel_id: &str,
        message_id: &str,
        data: &DataEditMessage,
    ) -> Result<Message> {
        self.request(
            Method::PATCH,
            format!("/channels/{}/messages/{}", channel_id, message_id),
        )
        .body(&data)
        .response()
        .await
    }

    pub async fn join_call(
        &self,
        channel_id: &str,
        data: &DataJoinCall,
    ) -> Result<CreateVoiceUserResponse> {
        self.request(Method::POST, format!("/channels/{channel_id}/join_call"))
            .body(data)
            .response()
            .await
    }

    pub async fn fetch_message(&self, channel_id: &str, message_id: &str) -> Result<Message> {
        self.request(
            Method::GET,
            format!("/channels/{channel_id}/messages/{message_id}"),
        )
        .response()
        .await
    }

    pub async fn upload_file(&self, tag: &str, file: LocalFile) -> Result<AutumnResponse> {
        self.autumn_request(Method::POST, format!("/{tag}"))
            .multipart(Form::new().part("file", Part::stream(file.body).file_name(file.name)))
            .response()
            .await
    }

    pub async fn delete_channel(&self, channel_id: &str) -> Result<()> {
        self.request(Method::DELETE, format!("/channels/{channel_id}"))
            .send()
            .await
    }

    pub async fn edit_channel(&self, channel_id: &str, data: &DataEditChannel) -> Result<Channel> {
        self.request(Method::PATCH, format!("/channels/{channel_id}"))
            .body(data)
            .response()
            .await
    }

    pub async fn fetch_channel(&self, channel_id: &str) -> Result<Channel> {
        self.request(Method::GET, format!("/channels/{channel_id}"))
            .response()
            .await
    }

    pub async fn fetch_members(&self, channel_id: &str) -> Result<Vec<User>> {
        self.request(Method::GET, format!("/channels/{channel_id}/members"))
            .response()
            .await
    }

    pub async fn fetch_server_members(
        &self,
        server_id: &str,
        options: &OptionsFetchAllMembers,
    ) -> Result<AllMemberResponse> {
        self.request(Method::GET, format!("/servers/{server_id}/members"))
            .query(options)
            .response()
            .await
    }

    pub async fn delete_messages(
        &self,
        channel_id: &str,
        options: &OptionsBulkDelete,
    ) -> Result<()> {
        self.request(
            Method::DELETE,
            format!("/channels/{channel_id}/messages/bulk"),
        )
        .body(options)
        .send()
        .await
    }

    pub async fn clear_reactions(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.request(
            Method::DELETE,
            format!("/channels/{channel_id}/messages/{message_id}/reactions"),
        )
        .send()
        .await
    }

    pub async fn pin_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.request(
            Method::POST,
            format!("/channels/{channel_id}/messages/{message_id}/pin"),
        )
        .send()
        .await
    }

    pub async fn unpin_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.request(
            Method::DELETE,
            format!("/channels/{channel_id}/messages/{message_id}/pin"),
        )
        .send()
        .await
    }

    pub async fn react_message(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<()> {
        self.request(
            Method::PUT,
            format!("/channels/{channel_id}/messages/{message_id}/reactions/{emoji}"),
        )
        .send()
        .await
    }

    pub async fn unreact_message(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
        options: &OptionsUnreact,
    ) -> Result<()> {
        self.request(
            Method::DELETE,
            format!("/channels/{channel_id}/messages/{message_id}/reactions/{emoji}"),
        )
        .query(options)
        .send()
        .await
    }

    pub async fn set_default_channel_permissions(
        &self,
        channel_id: &str,
        data: &DataDefaultChannelPermissions,
    ) -> Result<Channel> {
        self.request(
            Method::PUT,
            format!("/channels/{channel_id}/permissions/default"),
        )
        .body(data)
        .response()
        .await
    }

    pub async fn set_role_channel_permissions(
        &self,
        channel_id: &str,
        role_id: &str,
        data: &DataSetRolePermissions,
    ) -> Result<Channel> {
        self.request(
            Method::PUT,
            format!("/channels/{channel_id}/permissions/{role_id}"),
        )
        .body(data)
        .response()
        .await
    }

    pub async fn create_webhook(
        &self,
        channel_id: &str,
        data: &CreateWebhookBody,
    ) -> Result<Webhook> {
        self.request(Method::POST, format!("/channels/{channel_id}/webhooks"))
            .body(data)
            .response()
            .await
    }

    pub async fn fetch_webhooks(&self, channel_id: &str) -> Result<Vec<Webhook>> {
        self.request(Method::GET, format!("/channels/{channel_id}/webhooks"))
            .response()
            .await
    }

    pub async fn delete_invite(&self, invite_id: &str) -> Result<()> {
        self.request(Method::DELETE, format!("/invites/{invite_id}"))
            .send()
            .await
    }

    pub async fn fetch_invite(&self, invite_id: &str) -> Result<Invite> {
        self.request(Method::GET, format!("/invites/{invite_id}"))
            .response()
            .await
    }

    pub async fn join_invite(&self, invite_id: &str) -> Result<InviteJoinResponse> {
        self.request(Method::POST, format!("/invites/{invite_id}"))
            .response()
            .await
    }

    pub async fn ban_member(
        &self,
        server_id: &str,
        user_id: &str,
        data: &DataBanCreate,
    ) -> Result<ServerBan> {
        self.request(Method::PUT, format!("/servers/{server_id}/bans/{user_id}"))
            .body(data)
            .response()
            .await
    }

    pub async fn fetch_bans(&self, server_id: &str) -> Result<BanListResult> {
        self.request(Method::GET, format!("/servers/{server_id}/bans"))
            .response()
            .await
    }

    pub async fn unban_member(&self, server_id: &str, user_id: &str) -> Result<()> {
        self.request(
            Method::DELETE,
            format!("/servers/{server_id}/bans/{user_id}"),
        )
        .send()
        .await
    }

    pub async fn create_channel(
        &self,
        server_id: &str,
        data: &DataCreateServerChannel,
    ) -> Result<Channel> {
        self.request(Method::POST, format!("/servers/{server_id}/channels"))
            .body(data)
            .response()
            .await
    }

    pub async fn fetch_emojis(&self, server_id: &str) -> Result<Vec<Emoji>> {
        self.request(Method::GET, format!("/servers/{server_id}/emojis"))
            .response()
            .await
    }

    pub async fn fetch_invites(&self, server_id: &str) -> Result<Vec<Invite>> {
        self.request(Method::GET, format!("/servers/{server_id}/invites"))
            .response()
            .await
    }

    pub async fn edit_member(
        &self,
        server_id: &str,
        user_id: &str,
        data: &DataMemberEdit,
    ) -> Result<Member> {
        self.request(
            Method::PATCH,
            format!("/servers/{server_id}/members/{user_id}"),
        )
        .body(data)
        .response()
        .await
    }

    pub async fn kick_member(&self, server_id: &str, user_id: &str) -> Result<()> {
        self.request(
            Method::DELETE,
            format!("/servers/{server_id}/members/{user_id}"),
        )
        .send()
        .await
    }

    pub async fn set_default_server_permissions(
        &self,
        server_id: &str,
        data: &DataPermissionsValue,
    ) -> Result<Server> {
        self.request(
            Method::PUT,
            format!("/servers/{server_id}/permissions/default"),
        )
        .body(data)
        .response()
        .await
    }

    pub async fn set_role_server_permissions(
        &self,
        server_id: &str,
        role_id: &str,
        data: &DataSetServerRolePermission,
    ) -> Result<Server> {
        self.request(
            Method::PUT,
            format!("/servers/{server_id}/permissions/{role_id}"),
        )
        .body(data)
        .response()
        .await
    }

    pub async fn create_role(
        &self,
        server_id: &str,
        data: &DataCreateRole,
    ) -> Result<NewRoleResponse> {
        self.request(Method::POST, format!("/servers/{server_id}/roles"))
            .body(data)
            .response()
            .await
    }

    pub async fn delete_role(&self, server_id: &str, role_id: &str) -> Result<()> {
        self.request(
            Method::DELETE,
            format!("/servers/{server_id}/roles/{role_id}"),
        )
        .send()
        .await
    }

    pub async fn edit_role_positions(
        &self,
        server_id: &str,
        data: &DataEditRoleRanks,
    ) -> Result<Server> {
        self.request(Method::PATCH, format!("/servers/{server_id}/roles/ranks"))
            .body(data)
            .response()
            .await
    }

    pub async fn edit_role(
        &self,
        server_id: &str,
        role_id: &str,
        data: &DataEditRole,
    ) -> Result<Role> {
        self.request(
            Method::PATCH,
            format!("/servers/{server_id}/roles/{role_id}"),
        )
        .body(data)
        .response()
        .await
    }

    pub async fn fetch_role(&self, server_id: &str, role_id: &str) -> Result<Role> {
        self.request(Method::GET, format!("/servers/{server_id}/roles/{role_id}"))
            .response()
            .await
    }

    pub async fn delete_server(
        &self,
        server_id: &str,
        options: &OptionsServerDelete,
    ) -> Result<()> {
        self.request(Method::DELETE, format!("/servers/{server_id}"))
            .query(options)
            .send()
            .await
    }

    pub async fn edit_server(&self, server_id: &str, data: &DataEditServer) -> Result<Server> {
        self.request(Method::PATCH, format!("/servers/{server_id}"))
            .body(data)
            .response()
            .await
    }

    pub async fn fetch_server(
        &self,
        server_id: &str,
        options: &OptionsFetchServer,
    ) -> Result<FetchServerResponse> {
        self.request(Method::GET, format!("/servers/{server_id}"))
            .query(options)
            .response()
            .await
    }

    pub async fn edit_user(&self, user_id: &str, data: &DataEditUser) -> Result<User> {
        self.request(Method::PATCH, format!("/users/{user_id}"))
            .body(data)
            .response()
            .await
    }

    pub async fn fetch_dms(&self) -> Result<Vec<Channel>> {
        self.request(Method::GET, "/users/dms").response().await
    }

    pub async fn fetch_user_profile(&self, user_id: &str) -> Result<UserProfile> {
        self.request(Method::GET, format!("/users/{user_id}/profile"))
            .response()
            .await
    }

    pub async fn fetch_self(&self) -> Result<User> {
        self.request(Method::GET, "/users/@me").response().await
    }

    pub async fn fetch_user_flags(&self, user_id: &str) -> Result<FlagResponse> {
        self.request(Method::GET, format!("/users/{user_id}/flags"))
            .response()
            .await
    }

    pub async fn fetch_user_mutuals(&self, user_id: &str) -> Result<MutualResponse> {
        self.request(Method::GET, format!("/users/{user_id}/mutual"))
            .response()
            .await
    }

    pub async fn fetch_default_avatar(&self, user_id: &str) -> Result<Bytes> {
        self.request(Method::GET, format!("/users/{user_id}/default_avatar"))
            .execute()
            .await?
            .bytes()
            .await
            .map_err(Into::into)
    }

    pub async fn fetch_image_preview(&self, tag: &str, id: &str) -> Result<Bytes> {
        self.autumn_request(Method::GET, format!("{tag}/{id}"))
            .execute()
            .await?
            .bytes()
            .await
            .map_err(Into::into)
    }

    pub async fn fetch_image(&self, tag: &str, id: &str, filename: &str) -> Result<Bytes> {
        self.autumn_request(Method::GET, format!("{tag}/{id}/{filename}"))
            .execute()
            .await?
            .bytes()
            .await
            .map_err(Into::into)
    }

    pub async fn fetch_webhook(&self, webhook_id: &str) -> Result<ResponseWebhook> {
        self.request(Method::GET, format!("/webhooks/{webhook_id}"))
            .response()
            .await
    }

    pub async fn fetch_webhook_token(&self, webhook_id: &str, token: &str) -> Result<Webhook> {
        self.request(Method::GET, format!("/webhooks/{webhook_id}/{token}"))
            .response()
            .await
    }

    pub async fn edit_webhook(&self, webhook_id: &str, data: &DataEditWebhook) -> Result<Webhook> {
        self.request(Method::PATCH, format!("/webhooks/{webhook_id}"))
            .body(data)
            .response()
            .await
    }

    pub async fn edit_webhook_token(
        &self,
        webhook_id: &str,
        data: &DataEditWebhook,
        token: &str,
    ) -> Result<Webhook> {
        self.request(Method::PATCH, format!("/webhooks/{webhook_id}/{token}"))
            .body(data)
            .response()
            .await
    }

    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        self.request(Method::DELETE, format!("/webhooks/{webhook_id}"))
            .send()
            .await
    }

    pub async fn delete_webhook_token(&self, webhook_id: &str, token: &str) -> Result<()> {
        self.request(Method::DELETE, format!("/webhooks/{webhook_id}/{token}"))
            .send()
            .await
    }

    pub async fn execute_webhook_token(
        &self,
        webhook_id: &str,
        token: &str,
        data: &DataMessageSend,
    ) -> Result<Message> {
        self.request(Method::POST, format!("/webhooks/{webhook_id}/{token}"))
            .body(data)
            .response()
            .await
    }

    pub async fn fetch_settings(&self, data: &OptionsFetchSettings) -> Result<UserSettings> {
        self.request(Method::POST, format!("/sync/settings/fetch"))
            .body(data)
            .response()
            .await
    }

    pub async fn set_settings(
        &self,
        data: &std::collections::HashMap<String, Value>,
    ) -> Result<()> {
        self.request(Method::POST, format!("/sync/settings/set"))
            .body(
                &data
                    .into_iter()
                    .map(|(k, v)| (k, to_string(&v).unwrap()))
                    .collect::<std::collections::HashMap<_, _>>(),
            )
            .send()
            .await
    }

    pub async fn fetch_unreads(&self) -> Result<Vec<ChannelUnread>> {
        self.request(Method::GET, format!("/sync/unreads"))
            .response()
            .await
    }

    pub async fn add_friend_by_id(&self, user_id: &str) -> Result<User> {
        self.request(Method::PUT, format!("/users/{user_id}/friend"))
            .response()
            .await
    }

    pub async fn remove_friend(&self, user_id: &str) -> Result<User> {
        self.request(Method::DELETE, format!("/users/{user_id}/friend"))
            .response()
            .await
    }

    pub async fn add_friend(&self, data: &DataSendFriendRequest) -> Result<User> {
        self.request(Method::POST, format!("/users/friend"))
            .body(data)
            .response()
            .await
    }

    pub async fn create_emoji(&self, emoji_id: &str, data: &DataCreateEmoji) -> Result<Emoji> {
        self.request(Method::PUT, format!("/custom/emoji/{emoji_id}"))
            .body(data)
            .response()
            .await
    }

    pub async fn delete_emoji(&self, emoji_id: &str) -> Result<()> {
        self.request(Method::DELETE, format!("/custom/emoji/{emoji_id}"))
            .send()
            .await
    }

    pub async fn create_server(
        &self,
        data: &DataCreateServer,
    ) -> Result<CreateServerLegacyResponse> {
        self.request(Method::POST, format!("/servers/create"))
            .body(data)
            .response()
            .await
    }

    pub async fn search_channel(
        &self,
        channel_id: &str,
        data: &DataMessageSearch,
    ) -> Result<BulkMessageResponse> {
        self.request(Method::POST, format!("/channels/{channel_id}/search"))
            .body(data)
            .response()
            .await
    }

    pub async fn create_invite(&self, channel_id: &str) -> Result<Invite> {
        self.request(Method::POST, format!("/channels/{channel_id}/invites"))
            .response()
            .await
    }

    pub async fn ack_channel(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.request(
            Method::PUT,
            format!("/channels/{channel_id}/ack/{message_id}"),
        )
        .send()
        .await
    }

    pub async fn fetch_audit_logs(
        &self,
        server_id: &str,
        data: &OptionsAuditLogQuery,
    ) -> Result<AuditLogQueryResponse> {
        self.request(Method::GET, format!("/servers/{server_id}/audit_logs"))
            .query(data)
            .response()
            .await
    }

    pub async fn fetch_emoji(&self, emoji_id: &str) -> Result<Emoji> {
        self.request(Method::GET, format!("/custom/emoji/{emoji_id}"))
            .response()
            .await
    }

    pub async fn get_sessions(&self) -> Result<Vec<SessionInfo>> {
        self.request(Method::GET, "/auth/session/all")
            .response()
            .await
    }

    pub async fn fetch_mfa_status(&self) -> Result<MultiFactorStatus> {
        self.request(Method::GET, "/auth/mfa").response().await
    }

    pub async fn create_mfa_ticket(&self, data: &MFAResponse) -> Result<MFATicket> {
        self.request(Method::PUT, "/auth/mfa/ticket")
            .body(data)
            .response()
            .await
    }

    pub async fn revoke_all_sessions(&self, mfa_ticket: String, revoke_self: bool) -> Result<()> {
        self.request(Method::DELETE, "/auth/session/all")
            .query(&[("revoke_self", revoke_self)])
            .mfa_ticket(mfa_ticket)
            .send()
            .await
    }

    pub async fn revoke_session(&self, mfa_ticket: String, session_id: &str) -> Result<()> {
        self.request(Method::DELETE, format!("/auth/session/{session_id}"))
            .mfa_ticket(mfa_ticket)
            .send()
            .await
    }
}

pub struct HttpRequest {
    ratelimits: Arc<HashMap<u64, RatelimitEntry>>,
    service: Service,
    builder: RequestBuilder,
    query: Option<String>,
}

impl HttpRequest {
    fn resolve_bucket(service: Service, request: &Request) -> (&str, Option<&str>) {
        match service {
            Service::Api => {
                let mut segments = request.url().path_segments().unwrap();

                let segment = segments.next();
                let resource = segments.next();
                let extra = segments.next();

                if let Some(segment) = segment {
                    let method = request.method();

                    match (segment, resource, method) {
                        ("users", target, &Method::PATCH) => ("user_edit", target),
                        ("users", _, _) => {
                            if let Some("default_avatar") = extra {
                                return ("default_avatar", None);
                            }

                            ("users", None)
                        }
                        ("bots", _, _) => ("bots", None),
                        ("channels", Some(id), _) => {
                            if request.method() == &Method::POST {
                                if let Some("messages") = extra {
                                    return ("messaging", Some(id));
                                }
                            }

                            ("channels", Some(id))
                        }
                        ("servers", Some(id), _) => ("servers", Some(id)),
                        ("auth", _, _) => {
                            if request.method() == &Method::DELETE {
                                ("auth_delete", None)
                            } else {
                                ("auth", None)
                            }
                        }
                        ("swagger", _, _) => ("swagger", None),
                        ("safety", Some("report"), _) => ("safety_report", Some("report")),
                        ("safety", _, _) => ("safety", None),
                        _ => ("any", None),
                    }
                } else {
                    ("any", None)
                }
            }
            Service::Autumn => {
                let path = request.url().path_segments().unwrap().collect::<Vec<_>>();

                match (request.method(), path.as_slice()) {
                    (&Method::POST, &[tag]) => ("upload", Some(tag)),
                    _ => ("any", None),
                }
            }
        }
    }

    pub fn body<I: Serialize>(mut self, body: &I) -> HttpRequest {
        self.builder = self.builder.json(body);

        self
    }

    pub fn query<I: Serialize>(mut self, query: &I) -> HttpRequest {
        self.query = Some(
            serde_qs::Config::new()
                .array_format(serde_qs::ArrayFormat::Unindexed)
                .serialize_string(query)
                .unwrap(),
        );

        self
    }

    pub fn form<I: Serialize>(mut self, form: &I) -> HttpRequest {
        self.builder = self.builder.form(form);

        self
    }

    pub fn multipart(mut self, multipart: Form) -> HttpRequest {
        self.builder = self.builder.multipart(multipart);

        self
    }

    pub fn mfa_ticket(mut self, token: String) -> HttpRequest {
        self.builder = self.builder.header("X-Mfa-Ticket", token);

        self
    }

    pub async fn execute(self) -> Result<Response, Error> {
        let (client, req) = self.builder.build_split();

        let mut request = req?;

        if let Some(query) = self.query {
            request.url_mut().set_query(Some(&query));
        };

        log::debug!(
            "Sending http request {} {}?{}",
            request.method(),
            request.url().path(),
            request.url().query().unwrap_or_default()
        );

        let (bucket, resource) = Self::resolve_bucket(self.service, &request);
        let mut key = DefaultHasher::new();
        key.write(bucket.as_bytes());

        if let Some(resource) = resource {
            key.write(resource.as_bytes());
        };

        let key = key.finish();

        if let Some(entry) = self.ratelimits.get_async(&key).await {
            if entry.remaining == 0 {
                let duration = Duration::from_millis(entry.reset as u64);

                log::warn!(
                    "Ratelimit limit reached on: {:?} {}: sleeping for {:.3}s",
                    request.method(),
                    request.url().path(),
                    duration.as_secs_f32()
                );

                // TODO: switch to queue system to avoid re-hitting the ratelimit after the bucket is reset and too many requests are waiting
                sleep(duration).await;
            }
        }

        let response = client.execute(request).await?;

        let remaining = response
            .headers()
            .get("X-RateLimit-Remaining")
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();

        let reset = response
            .headers()
            .get("X-RateLimit-Reset-After")
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();

        self.ratelimits
            .upsert_async(key, RatelimitEntry { remaining, reset })
            .await;

        if response.status().as_u16() == 429 {
            let failure = response.json().await?;
            log::error!("{failure:?}");
            return Err(Error::RatelimitReached(failure));
        }

        if response.status().is_client_error() || response.status().is_server_error() {
            let error = response.json().await?;
            log::error!("{error:?}");
            Err(Error::HttpError(error))
        } else {
            Ok(response)
        }
    }

    pub async fn response<O: for<'a> Deserialize<'a>>(self) -> Result<O, Error> {
        self.execute()
            .await?
            .json()
            .await
            .map_err(Into::into)
            .inspect_err(|e| log::error!("{e:?}"))
    }

    pub async fn send(self) -> Result<(), Error> {
        self.execute().await?;

        Ok(())
    }
}
