const MANAGE_CHANNEL: (&str, &str, i64) = ("Manage Channel", "Edit and delete channel", 1 << 0);
const MANAGE_SERVER: (&str, &str, i64) = (
    "Manage Server",
    "Edit the server's information and settings",
    1 << 1,
);
const MANAGE_PERMISSIONS: (&str, &str, i64) = (
    "Manage Permissions",
    "Edit channel-specific role and default permissions",
    1 << 2,
);
const MANAGE_ROLES: (&str, &str, i64) = ("Manage Roles", "Create and edit server roles", 1 << 3);
const MANAGE_CUSTOMISATION: (&str, &str, i64) =
    ("Manage Customisation", "Create server emoji", 1 << 4);

const KICK_MEMBERS: (&str, &str, i64) = (
    "Kick Members",
    "Kick lower-ranking members from the server",
    1 << 6,
);
const BAN_MEMBERS: (&str, &str, i64) = (
    "Ban Members",
    "Ban lower-ranking members from the server",
    1 << 7,
);
const TIMEOUT_MEMBERS: (&str, &str, i64) = (
    "Timeout Members",
    "Temporarily prevent lower-ranking members from interacting",
    1 << 8,
);
const ASSIGN_ROLES: (&str, &str, i64) = (
    "Assign Roles",
    "Assign lower-ranked roles to lower-ranking members",
    1 << 9,
);
const CHANGE_NICKNAME: (&str, &str, i64) = ("Change Nickname", "Change own nickname", 1 << 10);
const MANAGE_NICKNAMES: (&str, &str, i64) = (
    "Manage Nicknames",
    "Change other members' nicknames",
    1 << 11,
);
const CHANGE_AVATAR: (&str, &str, i64) = ("Change Avatar", "Change own avatar", 1 << 12);
const REMOVE_AVATARS: (&str, &str, i64) =
    ("Remove Avatars", "Remove other members' avatars", 1 << 13);
const VIEW_CHANNEL: (&str, &str, i64) = (
    "View Channel",
    "Able to access channels on this server",
    1 << 20,
);
const READ_MESSAGE_HISTORY: (&str, &str, i64) = (
    "Read Message History",
    "Read past messages sent in channels",
    1 << 21,
);
const SEND_MESSAGES: (&str, &str, i64) = ("Send Messages", "Send messages in channels", 1 << 22);
const MANAGE_MESSAGES: (&str, &str, i64) = (
    "Manage Messages",
    "Delete and pin messages sent by other members",
    1 << 23,
);
const MANAGE_WEBHOOKS: (&str, &str, i64) = ("Manage Webhooks", "Create and edit webhooks", 1 << 24);
const INVITE_OTHERS: (&str, &str, i64) =
    ("Invite Others", "Create invites for others to use", 1 << 25);

const SEND_EMBEDS: (&str, &str, i64) = (
    "Send Embeds",
    "Send embedded content such as link embeds or custom embeds",
    1 << 26,
);
const UPLOAD_FILES: (&str, &str, i64) = ("Upload Files", "Send attachments to chat", 1 << 27);
const MASQUERADE: (&str, &str, i64) = (
    "Masquerade",
    "Allow members to change name and avatar per-message",
    1 << 28,
);
const REACT: (&str, &str, i64) = ("React", "React to messages with emoji", 1 << 29);
const BYPASS_SLOWMODE: (&str, &str, i64) =
    ("Bypass Slowmode", "Bypasses slowmode in channels", 1 << 39);
const CONNECT: (&str, &str, i64) = ("Connect", "Connect to voice channel", 1 << 30);
const SPEAK: (&str, &str, i64) = ("Speak", "Able to speak in voice call", 1 << 31);
const VIDEO: (&str, &str, i64) = ("Video", "Share camera or screen in voice call", 1 << 32);
const MUTE_MEMBERS: (&str, &str, i64) = (
    "Mute Members",
    "Mute lower-ranking members in voice call",
    1 << 33,
);
const DEAFEN_MEMBERS: (&str, &str, i64) = (
    "Deafen Members",
    "Deafen lower-ranking members in voice call",
    1 << 34,
);
const MOVE_MEMBERS: (&str, &str, i64) = (
    "Move Members",
    "Move members between voice channels",
    1 << 35,
);
const LISTEN: (&str, &str, i64) = ("Listen", "Hear other people and see their video", 1 << 36);

const MENTION_EVERYONE: (&str, &str, i64) = (
    "Mention Everyone",
    "Mention everyone and online members inside the server",
    1 << 37,
);
const MENTION_ROLES: (&str, &str, i64) = ("Mention Roles", "Mention specific roles", 1 << 38);

pub static SERVER_PERMISSIONS: &[(&'static str, &[(&'static str, &'static str, i64)])] = &[
    (
        "Admin",
        &[
            MANAGE_CHANNEL,
            MANAGE_SERVER,
            MANAGE_PERMISSIONS,
            MANAGE_ROLES,
            MANAGE_CUSTOMISATION,
        ],
    ),
    (
        "Members",
        &[
            KICK_MEMBERS,
            BAN_MEMBERS,
            TIMEOUT_MEMBERS,
            ASSIGN_ROLES,
            CHANGE_NICKNAME,
            MANAGE_NICKNAMES,
            CHANGE_AVATAR,
            REMOVE_AVATARS,
        ],
    ),
    (
        "Channels",
        &[
            VIEW_CHANNEL,
            READ_MESSAGE_HISTORY,
            SEND_MESSAGES,
            MANAGE_MESSAGES,
            MANAGE_WEBHOOKS,
            INVITE_OTHERS,
        ],
    ),
    (
        "Messaging",
        &[
            SEND_EMBEDS,
            UPLOAD_FILES,
            MASQUERADE,
            REACT,
            BYPASS_SLOWMODE,
        ],
    ),
    (
        "Voice",
        &[
            CONNECT,
            SPEAK,
            VIDEO,
            MUTE_MEMBERS,
            DEAFEN_MEMBERS,
            MOVE_MEMBERS,
            LISTEN,
        ],
    ),
    ("Mentions", &[MENTION_EVERYONE, MENTION_ROLES]),
];

pub static SERVER_CHANNEL_PERMISSIONS: &[(&'static str, &[(&'static str, &'static str, i64)])] = &[
    ("Admin", &[MANAGE_CHANNEL, MANAGE_PERMISSIONS]),
    (
        "Channels",
        &[
            VIEW_CHANNEL,
            READ_MESSAGE_HISTORY,
            SEND_MESSAGES,
            MANAGE_MESSAGES,
            MANAGE_WEBHOOKS,
            INVITE_OTHERS,
        ],
    ),
    (
        "Messaging",
        &[
            SEND_EMBEDS,
            UPLOAD_FILES,
            MASQUERADE,
            REACT,
            BYPASS_SLOWMODE,
        ],
    ),
    (
        "Voice",
        &[
            CONNECT,
            SPEAK,
            VIDEO,
            MUTE_MEMBERS,
            DEAFEN_MEMBERS,
            MOVE_MEMBERS,
            LISTEN,
        ],
    ),
    ("Mentions", &[MENTION_EVERYONE, MENTION_ROLES]),
];

pub static GROUP_CHANNEL_PERMISSIONS: &[(&'static str, &[(&'static str, &'static str, i64)])] = &[
    (
        "Admin",
        &[
            MANAGE_CHANNEL,
            MANAGE_PERMISSIONS,
            SEND_MESSAGES,
            MANAGE_MESSAGES,
            MANAGE_WEBHOOKS,
            INVITE_OTHERS,
        ],
    ),
    ("Messaging", &[SEND_EMBEDS, UPLOAD_FILES, MASQUERADE, REACT]),
    ("Mentions", &[MENTION_EVERYONE, MENTION_ROLES]),
];
