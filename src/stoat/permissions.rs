use freya::radio::Radio;
use stoat_models::v0::{Channel, Member, Server, User};
use stoat_permissions::{
    ALLOW_IN_TIMEOUT, ChannelPermission, ChannelType, DEFAULT_PERMISSION_DIRECT_MESSAGE,
    DEFAULT_PERMISSION_SAVED_MESSAGES, DEFAULT_PERMISSION_VIEW_ONLY, Override, PermissionValue,
    RelationshipStatus, UserPermission,
};

use crate::{AppChannel, AppState};

/// Stores all relavent info for a permission query.
#[derive(Clone)]
pub struct PermissionQuery {
    state: Radio<AppState, AppChannel>,

    perspective: User,
    user: Option<User>,
    channel: Option<Channel>,
    server: Option<Server>,
    member: Option<Member>,
}

impl PermissionQuery {
    /// Creates an instance of [`PermissionQuery`].
    ///
    /// You should use [`user_permissions_query`] over this in most cases.
    pub fn new(state: Radio<AppState, AppChannel>, perspective: User) -> Self {
        Self {
            state,
            perspective,
            user: None,
            channel: None,
            server: None,
            member: None,
        }
    }

    /// Use user
    pub fn user(mut self, user: User) -> Self {
        self.user = Some(user);

        self
    }

    /// Use channel
    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);

        self
    }

    /// Use server
    pub fn server(mut self, server: impl Into<Option<Server>>) -> Self {
        self.server = server.into();

        self
    }

    /// Use member
    pub fn member(mut self, member: impl Into<Option<Member>>) -> Self {
        self.member = member.into();

        self
    }

    fn are_we_privileged(&mut self) -> bool {
        self.perspective.privileged
    }

    /// Is our perspective user a bot?
    fn are_we_a_bot(&mut self) -> bool {
        self.perspective.bot.is_some()
    }

    /// Is our perspective user and the currently selected user the same?
    fn are_the_users_same(&mut self) -> bool {
        if let Some(other_user) = &self.user {
            self.perspective.id == other_user.id
        } else {
            false
        }
    }

    /// Get the relationship with have with the currently selected user
    fn user_relationship(&mut self) -> RelationshipStatus {
        if let Some(other_user) = &self.user {
            if self.perspective.id == other_user.id {
                return RelationshipStatus::User;
            } else if let Some(bot) = &other_user.bot {
                if self.perspective.id == bot.owner_id {
                    return RelationshipStatus::User;
                }
            }

            for entry in &self.perspective.relations {
                if entry.user_id == other_user.id {
                    return match entry.status {
                        stoat_models::v0::RelationshipStatus::None => RelationshipStatus::None,
                        stoat_models::v0::RelationshipStatus::User => RelationshipStatus::User,
                        stoat_models::v0::RelationshipStatus::Friend => RelationshipStatus::Friend,
                        stoat_models::v0::RelationshipStatus::Outgoing => {
                            RelationshipStatus::Outgoing
                        }
                        stoat_models::v0::RelationshipStatus::Incoming => {
                            RelationshipStatus::Incoming
                        }
                        stoat_models::v0::RelationshipStatus::Blocked => {
                            RelationshipStatus::Blocked
                        }
                        stoat_models::v0::RelationshipStatus::BlockedOther => {
                            RelationshipStatus::BlockedOther
                        }
                    };
                }
            }
        }

        RelationshipStatus::None
    }

    /// Whether the currently selected user is a bot
    fn user_is_bot(&mut self) -> bool {
        if let Some(other_user) = &self.user {
            other_user.bot.is_some()
        } else {
            false
        }
    }

    fn have_mutual_connection(&mut self) -> bool {
        true
    }

    // * For calculating server permission

    /// Is our perspective user the server's owner?
    fn are_we_server_owner(&mut self) -> bool {
        if let Some(server) = &self.server {
            server.owner == self.perspective.id
        } else {
            false
        }
    }

    /// Is our perspective user a member of the server?
    fn are_we_a_member(&mut self) -> bool {
        if let Some(server) = &self.server {
            let slice = self
                .state
                .slice(AppChannel::Members, |state| &state.members);

            if self.member.is_some() {
                true
            } else if let Some(member) = slice
                .read()
                .get(&server.id)
                .unwrap()
                .get(&self.perspective.id)
                .cloned()
            {
                self.member = Some(member);

                true
            } else {
                false
            }
        } else if let Some(member) = &self.member {
            let slice = self
                .state
                .slice(AppChannel::Servers, |state| &state.servers);

            if let Some(server) = slice.read().get(&member.id.server).cloned() {
                self.server = Some(server);

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get default server permission
    fn get_default_server_permissions(&mut self) -> u64 {
        if let Some(server) = &self.server {
            server.default_permissions as u64
        } else {
            0
        }
    }

    /// Get the ordered role overrides (from lowest to highest) for this member in this server
    fn get_our_server_role_overrides(&mut self) -> Vec<Override> {
        if let Some(server) = &self.server {
            let member_roles = self
                .member
                .as_ref()
                .map(|member| member.roles.clone())
                .unwrap_or_default();

            let mut roles = server
                .roles
                .iter()
                .filter(|(id, _)| member_roles.contains(id))
                .map(|(_, role)| {
                    let v: Override = role.permissions.into();
                    (role.rank, v)
                })
                .collect::<Vec<(i64, Override)>>();

            roles.sort_by(|a, b| b.0.cmp(&a.0));
            roles.into_iter().map(|(_, v)| v).collect()
        } else {
            vec![]
        }
    }

    /// Is our perspective user timed out on this server?
    fn are_we_timed_out(&mut self) -> bool {
        if let Some(member) = &self.member {
            member.timeout.is_some()
        } else {
            false
        }
    }

    /// Is the member muted?
    fn do_we_have_publish_overwrites(&mut self) -> bool {
        self.member.as_ref().is_none_or(|member| member.can_publish)
    }

    /// Is the member deafend?
    fn do_we_have_receive_overwrites(&mut self) -> bool {
        self.member.as_ref().is_none_or(|member| member.can_receive)
    }

    // * For calculating channel permission

    /// Get the type of the channel
    fn get_channel_type(&mut self) -> ChannelType {
        if let Some(channel) = &self.channel {
            match channel {
                Channel::DirectMessage { .. } => ChannelType::DirectMessage,
                Channel::Group { .. } => ChannelType::Group,
                Channel::SavedMessages { .. } => ChannelType::SavedMessages,
                Channel::TextChannel { .. } => ChannelType::ServerChannel,
            }
        } else {
            ChannelType::Unknown
        }
    }

    /// Get the default channel permissions
    /// Group channel defaults should be mapped to an allow-only override
    fn get_default_channel_permissions(&mut self) -> Override {
        if let Some(channel) = &self.channel {
            match channel {
                Channel::Group { permissions, .. } => Override {
                    allow: permissions.unwrap_or(*DEFAULT_PERMISSION_DIRECT_MESSAGE as i64) as u64,
                    deny: 0,
                },
                Channel::TextChannel {
                    default_permissions,
                    ..
                } => default_permissions.unwrap_or_default().into(),
                _ => Default::default(),
            }
        } else {
            Default::default()
        }
    }

    /// Get the ordered role overrides (from lowest to highest) for this member in this channel
    fn get_our_channel_role_overrides(&mut self) -> Vec<Override> {
        if let Some(channel) = &self.channel {
            match channel {
                Channel::TextChannel {
                    role_permissions, ..
                } => {
                    if let Some(server) = &self.server {
                        let member_roles = self
                            .member
                            .as_ref()
                            .map(|member| member.roles.clone())
                            .unwrap_or_default();

                        let mut roles = role_permissions
                            .iter()
                            .filter(|(id, _)| member_roles.contains(id))
                            .filter_map(|(id, permission)| {
                                server.roles.get(id).map(|role| {
                                    let v: Override = (*permission).into();
                                    (role.rank, v)
                                })
                            })
                            .collect::<Vec<(i64, Override)>>();

                        roles.sort_by(|a, b| b.0.cmp(&a.0));
                        roles.into_iter().map(|(_, v)| v).collect()
                    } else {
                        vec![]
                    }
                }
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Do we own this group or saved messages channel if it is one of those?
    fn do_we_own_the_channel(&mut self) -> bool {
        if let Some(channel) = &self.channel {
            match channel {
                Channel::Group { owner, .. } => owner == &self.perspective.id,
                Channel::SavedMessages { user, .. } => user == &self.perspective.id,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Are we a recipient of this channel?
    fn are_we_part_of_the_channel(&mut self) -> bool {
        if let Some(Channel::DirectMessage { recipients, .. } | Channel::Group { recipients, .. }) =
            &self.channel
        {
            recipients.contains(&self.perspective.id)
        } else {
            false
        }
    }

    /// Set the current user as the recipient of this channel
    /// (this will only ever be called for DirectMessage channels, use unimplemented!() for other code paths)
    fn set_recipient_as_user(&mut self) {
        if let Some(channel) = &self.channel {
            match channel {
                Channel::DirectMessage { recipients, .. } => {
                    let recipient_id = recipients
                        .iter()
                        .find(|recipient| recipient != &&self.perspective.id)
                        .expect("Missing recipient for DM");

                    let slice = self.state.slice(AppChannel::Users, |state| &state.users);

                    if let Some(user) = slice.read().get(recipient_id).cloned() {
                        self.user.replace(user);
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    /// Set the current server as the server owning this channel
    /// (this will only ever be called for server channels, use unimplemented!() for other code paths)
    fn set_server_from_channel(&mut self) {
        if let Some(channel) = &self.channel {
            match channel {
                Channel::TextChannel { server, .. } => {
                    if let Some(known_server) = self.server.as_ref() {
                        if server == &known_server.id {
                            // Already cached, return early.
                            return;
                        }
                    }

                    let slice = self
                        .state
                        .slice(AppChannel::Servers, |state| &state.servers);

                    if let Some(server) = slice.read().get(server).cloned() {
                        self.server.replace(server);
                    }
                }
                _ => unimplemented!(),
            }
        }
    }
}

pub fn user_permissions_query(state: Radio<AppState, AppChannel>) -> PermissionQuery {
    let user = state
        .slice(AppChannel::Users, |state| {
            state.users.get(state.user_id.as_ref().unwrap()).unwrap()
        })
        .read()
        .cloned();

    PermissionQuery::new(state, user)
}

/// Calculate permissions against a user
pub fn calculate_user_permissions(query: &mut PermissionQuery) -> PermissionValue {
    if query.are_we_privileged() {
        return u64::MAX.into();
    }

    if query.are_the_users_same() {
        return u64::MAX.into();
    }

    let mut permissions = 0_u64;
    match query.user_relationship() {
        RelationshipStatus::Friend => return u64::MAX.into(),
        RelationshipStatus::Blocked | RelationshipStatus::BlockedOther => {
            return (UserPermission::Access as u64).into();
        }
        RelationshipStatus::Incoming | RelationshipStatus::Outgoing => {
            permissions = UserPermission::Access as u64;
        }
        _ => {}
    }

    if query.have_mutual_connection() {
        permissions = UserPermission::Access as u64 + UserPermission::ViewProfile as u64;

        if query.user_is_bot() || query.are_we_a_bot() {
            permissions += UserPermission::SendMessage as u64;
        }

        permissions.into()
    } else {
        permissions.into()
    }

    // TODO: add boolean switch for permission for users to globally message a user
    // maybe an enum?
    // PrivacyLevel { Private, Friends, Mutual, Public, Global }

    // TODO: add boolean switch for permission for users to mutually DM a user
}

/// Calculate permissions against a server
pub fn calculate_server_permissions(query: &mut PermissionQuery) -> PermissionValue {
    if query.are_we_privileged() || query.are_we_server_owner() {
        return ChannelPermission::GrantAllSafe.into();
    }

    if !query.are_we_a_member() {
        return 0_u64.into();
    }

    let mut permissions: PermissionValue = query.get_default_server_permissions().into();

    for role_override in query.get_our_server_role_overrides() {
        permissions.apply(role_override);
    }

    if !query.do_we_have_publish_overwrites() {
        permissions.revoke(ChannelPermission::Speak as u64);
        permissions.revoke(ChannelPermission::Video as u64);
    }

    if !query.do_we_have_receive_overwrites() {
        permissions.revoke(ChannelPermission::Listen as u64);
    }

    if query.are_we_timed_out() {
        permissions.restrict(*ALLOW_IN_TIMEOUT);
    }

    permissions
}

/// Calculate permissions against a channel
pub fn calculate_channel_permissions(query: &mut PermissionQuery) -> PermissionValue {
    if query.are_we_privileged() {
        return ChannelPermission::GrantAllSafe.into();
    }

    match query.get_channel_type() {
        ChannelType::SavedMessages => {
            if query.do_we_own_the_channel() {
                DEFAULT_PERMISSION_SAVED_MESSAGES.into()
            } else {
                0_u64.into()
            }
        }
        ChannelType::DirectMessage => {
            if query.are_we_part_of_the_channel() {
                query.set_recipient_as_user();

                let permissions = calculate_user_permissions(query);
                if permissions.has_user_permission(UserPermission::SendMessage) {
                    (*DEFAULT_PERMISSION_DIRECT_MESSAGE).into()
                } else {
                    (*DEFAULT_PERMISSION_VIEW_ONLY).into()
                }
            } else {
                0_u64.into()
            }
        }
        ChannelType::Group => {
            if query.do_we_own_the_channel() {
                ChannelPermission::GrantAllSafe.into()
            } else if query.are_we_part_of_the_channel() {
                (*DEFAULT_PERMISSION_VIEW_ONLY | query.get_default_channel_permissions().allow)
                    .into()
            } else {
                0_u64.into()
            }
        }
        ChannelType::ServerChannel => {
            query.set_server_from_channel();

            if query.are_we_server_owner() {
                ChannelPermission::GrantAllSafe.into()
            } else if query.are_we_a_member() {
                let mut permissions = calculate_server_permissions(query);
                permissions.apply(query.get_default_channel_permissions());

                for role_override in query.get_our_channel_role_overrides() {
                    permissions.apply(role_override);
                }

                if query.are_we_timed_out() {
                    permissions.restrict(*ALLOW_IN_TIMEOUT);
                }

                if !permissions.has_channel_permission(ChannelPermission::ViewChannel) {
                    permissions.revoke_all();
                }

                permissions
            } else {
                0_u64.into()
            }
        }
        ChannelType::Unknown => 0_u64.into(),
    }
}
