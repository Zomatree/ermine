use freya::prelude::*;
use stoat_permissions::OverrideField;

use crate::{
    SizeExt,
    components::{
        MaterialIcon,
        checkbox::StoatCheckbox,
        material::{
            filled::{check, clear},
            outlined::remove,
        },
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct PermissionsEditor {
    overrite: bool,
    allow: Writable<i64>,
    deny: Writable<i64>,
}

impl PermissionsEditor {
    pub fn new_value(permission: impl IntoWritable<i64>) -> Self {
        Self {
            overrite: false,
            allow: permission.into_writable(),
            deny: Writable::from_state(State::create(0)),
        }
    }

    pub fn new_overrite(overrite: impl IntoWritable<OverrideField>) -> Self {
        let writable = overrite.into_writable();

        Self {
            overrite: true,
            allow: writable.map(|v| &v.a, |v| &mut v.a),
            deny: writable.map(|v| &v.d, |v| &mut v.d),
        }
    }
}

static PERMISSION_VALUES: &[(&'static str, &[(&'static str, &'static str, i64)])] = &[
    (
        "Admin",
        &[
            ("Manage Channel", "Edit and delete channel", 1 << 0),
            (
                "Manage Server",
                "Edit the server's information and settings",
                1 << 1,
            ),
            (
                "Manage Permissions",
                "Edit channel-specific role and default permissions",
                1 << 2,
            ),
            ("Manage Roles", "Create and edit server roles", 1 << 3),
            ("Manage Customisation", "Create server emoji", 1 << 4),
        ],
    ),
    (
        "Members",
        &[
            (
                "Kick Members",
                "Kick lower-ranking members from the server",
                1 << 6,
            ),
            (
                "Ban Members",
                "Ban lower-ranking members from the server",
                1 << 7,
            ),
            (
                "Timeout Members",
                "Temporarily prevent lower-ranking members from interacting",
                1 << 8,
            ),
            (
                "Assign Roles",
                "Assign lower-ranked roles to lower-ranking members",
                1 << 9,
            ),
            ("Change Nickname", "Change own nickname", 1 << 10),
            (
                "Manage Nicknames",
                "Change other members' nicknames",
                1 << 11,
            ),
            ("Change Avatar", "Change own avatar", 1 << 12),
            ("Remove Avatars", "Remove other members' avatars", 1 << 13),
        ],
    ),
    (
        "Channels",
        &[
            (
                "View Channel",
                "Able to access channels on this server",
                1 << 20,
            ),
            (
                "Read Message History",
                "Read past messages sent in channels",
                1 << 21,
            ),
            ("Send Messages", "Send messages in channels", 1 << 22),
            (
                "Manage Messages",
                "Delete and pin messages sent by other members",
                1 << 23,
            ),
            ("Manage Webhooks", "Create and edit webhooks", 1 << 24),
            ("Invite Others", "Create invites for others to use", 1 << 25),
        ],
    ),
    (
        "Messaging",
        &[
            (
                "Send Embeds",
                "Send embedded content such as link embeds or custom embeds",
                1 << 26,
            ),
            ("Upload Files", "Send attachments to chat", 1 << 27),
            (
                "Masquerade",
                "Allow members to change name and avatar per-message",
                1 << 28,
            ),
            ("React", "React to messages with emoji", 1 << 29),
            ("Bypass Slowmode", "Bypasses slowmode in channels", 1 << 39),
        ],
    ),
    (
        "Voice",
        &[
            ("Connect", "Connect to voice channel", 1 << 30),
            ("Speak", "Able to speak in voice call", 1 << 31),
            ("Video", "Share camera or screen in voice call", 1 << 32),
            (
                "Mute Members",
                "Mute lower-ranking members in voice call",
                1 << 33,
            ),
            (
                "Deafen Members",
                "Deafen lower-ranking members in voice call",
                1 << 34,
            ),
            (
                "Move Members",
                "Move members between voice channels",
                1 << 35,
            ),
            ("Listen", "Hear other people and see their video", 1 << 36),
        ],
    ),
    (
        "Mentions",
        &[
            (
                "Mention Everyone",
                "Mention everyone and online members inside the server",
                1 << 37,
            ),
            ("Mention Roles", "Mention specific roles", 1 << 38),
        ],
    ),
];

impl Component for PermissionsEditor {
    fn render(&self) -> impl IntoElement {
        rect()
            .spacing(8.)
            .children(
                PERMISSION_VALUES
                    .iter()
                    .copied()
                    .map(|(header, permissions)| {
                        rect()
                            .child(
                                rect()
                                    .height(Size::px(24.))
                                    .main_align(Alignment::Center)
                                    .child(
                                        label()
                                            .font_size(12.)
                                            .font_weight(FontWeight::MEDIUM)
                                            .text(header),
                                    ),
                            )
                            .child(rect().children(permissions.iter().copied().map(
                                |(title, description, bit)| {
                                    if self.overrite {
                                        PermissionOverrite {
                                            allow: self.allow.clone(),
                                            deny: self.deny.clone(),
                                            title,
                                            description,
                                            bit,
                                        }
                                        .into_element()
                                    } else {
                                        PermissionToggle {
                                            value: self.allow.clone(),
                                            title,
                                            description,
                                            bit,
                                        }
                                        .into_element()
                                    }
                                },
                            )))
                            .into_element()
                    }),
            )
    }
}

#[derive(PartialEq)]
struct PermissionToggle {
    pub value: Writable<i64>,
    pub title: &'static str,
    pub description: &'static str,
    pub bit: i64,
}

impl Component for PermissionToggle {
    fn render(&self) -> impl IntoElement {
        let toggle = use_state(|| (*self.value.read() & self.bit) == self.bit);

        use_side_effect({
            let mut value = self.value.clone();
            let bit = self.bit;
            move || {
                let mut v = value.write();

                if toggle() {
                    *v |= bit;
                } else {
                    *v ^= bit;
                }
            }
        });

        StoatCheckbox::new(toggle).child(
            rect()
                .padding((10., 0.))
                .child(label().font_size(16.).line_height(1.5).text(self.title))
                .child(
                    label()
                        .font_size(14.)
                        .line_height(1.25)
                        .text(self.description),
                ),
        )
    }
}

#[derive(PartialEq)]
struct PermissionOverrite {
    pub allow: Writable<i64>,
    pub deny: Writable<i64>,
    pub title: &'static str,
    pub description: &'static str,
    pub bit: i64,
}

impl Component for PermissionOverrite {
    fn render(&self) -> impl IntoElement {
        let current = if (*self.allow.read() & self.bit) == self.bit {
            Some(true)
        } else if (*self.deny.read() & self.bit) == self.bit {
            Some(false)
        } else {
            None
        };

        rect()
            .content(Content::Flex)
            .horizontal()
            .spacing(8.)
            .cross_align(Alignment::Center)
            .child(
                rect()
                    .width(Size::flex(1.))
                    .padding((10., 0.))
                    .child(label().font_size(16.).line_height(1.5).text(self.title))
                    .child(
                        label()
                            .font_size(14.)
                            .line_height(1.25)
                            .text(self.description),
                    ),
            )
            .child(
                rect()
                    .corner_radius(12.)
                    .horizontal()
                    .overflow(Overflow::Clip)
                    .child(PermissionOverriteSwitchOverride {
                        allow: self.allow.clone(),
                        deny: self.deny.clone(),
                        bit: self.bit,
                        current,
                        value: Some(true),
                    })
                    .child(PermissionOverriteSwitchOverride {
                        allow: self.allow.clone(),
                        deny: self.deny.clone(),
                        bit: self.bit,
                        current,
                        value: None,
                    })
                    .child(PermissionOverriteSwitchOverride {
                        allow: self.allow.clone(),
                        deny: self.deny.clone(),
                        bit: self.bit,
                        current,
                        value: Some(false),
                    }),
            )
    }
}

#[derive(PartialEq)]
struct PermissionOverriteSwitchOverride {
    pub allow: Writable<i64>,
    pub deny: Writable<i64>,
    pub bit: i64,
    pub current: Option<bool>,
    pub value: Option<bool>,
}

impl Component for PermissionOverriteSwitchOverride {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut hover = use_state(|| false);
        let a11y_id = use_a11y();

        rect()
            .width(Size::px(36.))
            .height(Size::px(36.))
            .a11y_id(a11y_id)
            .a11y_role(AccessibilityRole::CheckBox)
            .center()
            .on_press({
                let value = self.value.clone();
                let bit = self.bit;
                let mut allow = self.allow.clone();
                let mut deny = self.deny.clone();

                move |_| {
                    a11y_id.request_focus();

                    match value {
                        Some(true) => {
                            {
                                let mut allow = allow.write();
                                *allow |= bit;
                            };
                            {
                                let mut deny = deny.write();
                                *deny &= !bit;
                            };
                        }
                        None => {
                            {
                                let mut allow = allow.write();
                                *allow &= !bit;
                            }
                            {
                                let mut deny = deny.write();
                                *deny &= !bit;
                            };
                        }
                        Some(false) => {
                            {
                                let mut allow = allow.write();
                                *allow &= !bit;
                            };
                            {
                                let mut deny = deny.write();
                                *deny |= bit;
                            };
                        }
                    };
                }
            })
            .on_pointer_over(move |_| {
                hover.set(true);
            })
            .on_pointer_out(move |_| hover.set_if_modified(false))
            .on_pointer_enter(move |_| {
                Cursor::set(CursorIcon::Pointer);
            })
            .on_pointer_leave(move |_| {
                Cursor::set(CursorIcon::default());
            })
            .background(if self.current == self.value {
                match self.value {
                    Some(true) => theme.md.on_primary_container.as_argb_u32(),
                    None => theme.md.on_secondary.as_argb_u32(),
                    Some(false) => theme.md.on_error_container.as_argb_u32(),
                }
            } else {
                theme.md.surface_container_high.as_argb_u32()
            })
            .color(if self.current == self.value {
                match self.value {
                    Some(true) => theme.md.primary_container.as_argb_u32(),
                    None => theme.md.secondary.as_argb_u32(),
                    Some(false) => theme.md.error_container.as_argb_u32(),
                }
            } else {
                theme.md.on_surface.as_argb_u32()
            })
            .child(
                MaterialIcon::new(match self.value {
                    Some(true) => check(),
                    None => remove(),
                    Some(false) => clear(),
                })
                .size(Size::px(24.)),
            )
            .maybe_child(hover().then(|| {
                rect()
                    .width(Size::px(36.))
                    .height(Size::px(36.))
                    .position(Position::new_absolute().left(0.).top(0.))
                    .background(theme.md.on_surface.as_argb_u32())
                    .opacity(0.08)
            }))
    }
}
