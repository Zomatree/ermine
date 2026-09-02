use freya::prelude::*;
use jiff::{Timestamp, tz::TimeZone};
use stoat_models::v0;
use ulid::Ulid;

use crate::{
    SizeExt,
    components::{
        MaterialIcon, ModalValue, StoatButton, StoatButtonLayoutThemePartialExt,
        material::outlined::{chevron_right, delete, logout, question_mark, star},
        use_modals,
    },
    consume_material_theme, http, use_config,
};

#[derive(PartialEq)]
pub struct SessionsSettings {}

impl Component for SessionsSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();
        let config = use_config();

        let current_session = use_hook(|| config.read().session.clone().unwrap());
        let current_session_icon = use_hook(|| session_icon(&current_session.name));

        let mut sessions = use_state(Vec::new);

        let mut future = use_future(move || async move {
            if let Ok(mut resp) = http().get_sessions().await {
                resp.sort_by(|a, b| b.id.cmp(&a.id));
                sessions.set(resp);
            }
        });

        let refetch = use_state(|| ());
        use_side_effect(move || {
            refetch.read();
            future.start()
        });

        rect().spacing(15.).child(
            rect()
                .corner_radius(28.)
                .overflow(Overflow::Clip)
                .spacing(2.)
                .child(
                    rect()
                        .corner_radius(12.)
                        .padding(13.)
                        .background(theme.md.secondary_container.as_argb_u32())
                        .color(theme.md.on_secondary_container.as_argb_u32())
                        .child(
                            rect()
                                .horizontal()
                                .spacing(16.)
                                .cross_align(Alignment::Center)
                                .content(Content::Flex)
                                .child(
                                    rect()
                                        .corner_radius(36.)
                                        .width(Size::px(36.))
                                        .height(Size::px(36.))
                                        .background(theme.md.surface_dim.as_argb_u32())
                                        .color(theme.md.on_surface.as_argb_u32())
                                        .center()
                                        .child(
                                            MaterialIcon::new(current_session_icon)
                                                .size(Size::px(22.)),
                                        ),
                                )
                                .child(
                                    rect()
                                        .width(Size::flex(1.))
                                        .child(
                                            label()
                                                .font_size(14.)
                                                .font_weight(FontWeight::MEDIUM)
                                                .line_height(1.5)
                                                .text("Current Session"),
                                        )
                                        .child(
                                            label()
                                                .font_size(12.)
                                                .line_height(1.5)
                                                .text(current_session.name),
                                        ),
                                )
                        ),
                )
                .child(
                    StoatButton::new()
                        .corner_radius(12.)
                        .on_press({
                            move |_| {
                                modals.write().push_modal(ModalValue::LogoutOtherSessions {
                                    callback: EventHandler::new(move |_| {
                                        future.start();
                                    })
                                });
                            }
                        })
                        .child(
                            rect()
                                .padding(13.)
                                .background(theme.md.secondary_container.as_argb_u32())
                                .color(theme.md.on_secondary_container.as_argb_u32())
                                .child(
                                    rect()
                                        .horizontal()
                                        .spacing(16.)
                                        .cross_align(Alignment::Center)
                                        .content(Content::Flex)
                                        .child(
                                            rect()
                                                .corner_radius(36.)
                                                .width(Size::px(36.))
                                                .height(Size::px(36.))
                                                .background(theme.md.surface_dim.as_argb_u32())
                                                .color(theme.md.error.as_argb_u32())
                                                .center()
                                                .child(
                                                    MaterialIcon::new(logout()).size(Size::px(22.)),
                                                ),
                                        )
                                        .child(
                                            rect()
                                                .width(Size::flex(1.))
                                                .child(
                                                    label()
                                                        .font_size(14.)
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .line_height(1.5)
                                                        .text("Log Out Other Sessions"),
                                                )
                                                .child(
                                                    label()
                                                        .font_size(12.)
                                                        .line_height(1.5)
                                                        .text("Logs you out of all sessions except this device."),
                                                ),
                                        )
                                        .child(
                                            MaterialIcon::new(chevron_right()).size(Size::px(18.)),
                                        ),
                                ),
                        ),
                ),
        ).child(
            rect()
                .corner_radius(28.)
                .overflow(Overflow::Clip)
                .spacing(2.)
                .children(sessions.read().iter().cloned().map(|info|
                    Session { info, refetch }.into_element()
                ))
        )
    }
}

#[derive(PartialEq)]
struct Session {
    pub info: v0::SessionInfo,
    pub refetch: State<()>,
}

impl Component for Session {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let created_at = use_hook(|| {
            Timestamp::try_from(Ulid::from_string(&self.info.id).unwrap().datetime())
                .unwrap()
                .to_zoned(TimeZone::system())
                .strftime("%d/%m/%y %H:%M")
                .to_string()
        });

        let icon = use_hook(|| session_icon(&self.info.name));

        rect()
            .corner_radius(12.)
            .padding(13.)
            .background(theme.md.secondary_container.as_argb_u32())
            .color(theme.md.on_secondary_container.as_argb_u32())
            .child(
                rect()
                    .horizontal()
                    .spacing(16.)
                    .cross_align(Alignment::Center)
                    .content(Content::Flex)
                    .child(
                        rect()
                            .corner_radius(36.)
                            .width(Size::px(36.))
                            .height(Size::px(36.))
                            .background(theme.md.surface_dim.as_argb_u32())
                            .color(theme.md.on_surface.as_argb_u32())
                            .center()
                            .child(MaterialIcon::new(icon).size(Size::px(22.))),
                    )
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .child(
                                label()
                                    .font_size(14.)
                                    .font_weight(FontWeight::MEDIUM)
                                    .line_height(1.5)
                                    .text(self.info.name.clone()),
                            )
                            .child(
                                label()
                                    .font_size(12.)
                                    .line_height(1.5)
                                    .text(format!("Created on {created_at}")),
                            ),
                    )
                    .child(
                        StoatButton::new()
                            .corner_radius(18.)
                            .on_press({
                                let session_id = self.info.id.clone();
                                let mut refetch = self.refetch;

                                move |_| {
                                    let session_id = session_id.clone();

                                    modals.write().push_modal(ModalValue::MFA {
                                        callback: EventHandler::new(
                                            move |ticket: v0::MFATicket| {
                                                let session_id = session_id.clone();

                                                spawn_forever(async move {
                                                    if http()
                                                        .revoke_session(ticket.token, &session_id)
                                                        .await
                                                        .is_ok()
                                                    {
                                                        refetch.set(());
                                                    };
                                                });
                                            },
                                        ),
                                    });
                                }
                            })
                            .child(
                                rect()
                                    .size(Size::px(36.))
                                    .background(theme.md.surface_dim.as_argb_u32())
                                    .color(theme.md.error.as_argb_u32())
                                    .center()
                                    .child(MaterialIcon::new(delete()).size(Size::px(24.))),
                            ),
                    ),
            )
    }
}

fn session_icon(name: &str) -> Bytes {
    let name = name.to_lowercase();

    if name.contains("android") {
        Bytes::from_static(include_bytes!("../../../assets/icons/android.svg"))
    } else if name.contains("windows") {
        Bytes::from_static(include_bytes!("../../../assets/icons/windows.svg"))
    } else if name.contains("mac") || name.contains("ios") || name.contains("ipad") {
        Bytes::from_static(include_bytes!("../../../assets/icons/apple.svg"))
    } else if name.contains("linux") {
        Bytes::from_static(include_bytes!("../../../assets/icons/linux.svg"))
    } else if name.contains("ermine") {
        star()
    } else {
        question_mark()
    }
}
