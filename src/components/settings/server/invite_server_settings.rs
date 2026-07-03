use std::borrow::Cow;

use crate::{
    AppChannel,
    components::{Avatar, ModalValue, StoatButton, StoatButtonLayoutThemePartialExt, use_modals},
    http, use_material_theme,
};
use freya::{
    icons::lucide::{copy, trash},
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;

struct Invite {
    code: String,
    server: String,
    creator: String,
    channel: String,
}

#[derive(PartialEq)]
pub struct InviteServerSettings {
    pub server: Readable<v0::Server>,
}

impl Component for InviteServerSettings {
    fn render(&self) -> impl IntoElement {
        let theme = use_material_theme();
        let mut modals = use_modals();
        let radio = use_radio(AppChannel::Users);
        let users = radio.slice_current(|state| &state.users);
        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);

        let mut invites = use_state(Vec::new);

        use_hook(move || {
            let server_id = self.server.read().id.clone();

            spawn(async move {
                if let Ok(fetched_invites) = http().fetch_invites(&server_id).await {
                    for invite in fetched_invites {
                        if let v0::Invite::Server {
                            code,
                            server,
                            creator,
                            channel,
                        } = invite
                        {
                            invites.write().push(Invite {
                                code,
                                server,
                                creator,
                                channel,
                            });
                        }
                    }
                }
            })
        });

        rect()
            .spacing(8.)
            .child(
                StoatButton::new().child(
                    rect()
                        .corner_radius(20.)
                        .height(Size::px(40.))
                        .width(Size::Fill)
                        .center()
                        .font_size(14.)
                        .background(theme.md.secondary_container.as_argb_u32())
                        .color(theme.md.on_secondary_container.as_argb_u32())
                        .child("Create invite"),
                ),
            )
            .child(
                rect()
                    .corner_radius(12.)
                    .border(
                        Border::new()
                            .width(1.)
                            .fill(theme.md.outline_variant.as_argb_u32()),
                    )
                    .child(
                        rect()
                            .horizontal()
                            .content(Content::Flex)
                            .width(Size::Fill)
                            .color(theme.md.on_surface_variant.as_argb_u32())
                            .border(
                                Border::new()
                                    .width(BorderWidth {
                                        top: 0.,
                                        right: 0.,
                                        bottom: if invites.read().len() != 0 { 1. } else { 0. },
                                        left: 0.,
                                    })
                                    .fill(theme.md.outline_variant.as_argb_u32()),
                            )
                            .child(rect().width(Size::flex(1.)).padding(15.).child("Inviter"))
                            .child(
                                rect()
                                    .width(Size::flex(1.))
                                    .padding(15.)
                                    .child("Invite Code"),
                            )
                            .child(rect().padding(15.).child(rect().width(Size::px(80.)))),
                    )
                    .child({
                        let users = users.read();
                        let channels = channels.read();
                        let invites = invites.read();

                        rect().children(invites.iter().enumerate().map(|(idx, invite)| {
                            let user = users.get(&invite.creator);
                            let channel = channels.get(&invite.channel).unwrap();

                            rect()
                                .horizontal()
                                .cross_align(Alignment::Center)
                                .content(Content::Flex)
                                .width(Size::Fill)
                                .border(
                                    Border::new()
                                        .width(BorderWidth {
                                            top: 0.,
                                            right: 0.,
                                            bottom: if idx != invites.len() - 1 { 1. } else { 0. },
                                            left: 0.,
                                        })
                                        .fill(theme.md.outline_variant.as_argb_u32()),
                                )
                                .child(
                                    rect()
                                        .horizontal()
                                        .width(Size::flex(1.))
                                        .padding(15.)
                                        .spacing(8.)
                                        .cross_align(Alignment::Center)
                                        .child(
                                            rect().child(match user {
                                                Some(user) => Avatar::new(
                                                    user.clone().into_readable(),
                                                    None,
                                                    32.,
                                                )
                                                .into_element(),
                                                None => rect()
                                                    .width(Size::px(32.))
                                                    .height(Size::px(32.))
                                                    .into_element(),
                                            }),
                                        )
                                        .child(
                                            rect()
                                                .child(label().font_size(16.).text(match user {
                                                    Some(user) => Cow::Owned(user.username.clone()),
                                                    None => Cow::Borrowed("Unknown user"),
                                                }))
                                                .child(label().font_size(12.).text(format!(
                                                    "#{}",
                                                    channel.name().unwrap().to_string()
                                                ))),
                                        ),
                                )
                                .child(
                                    rect()
                                        .width(Size::flex(1.))
                                        .padding(15.)
                                        .child(invite.code.clone()),
                                )
                                .child(
                                    rect()
                                        .padding(15.)
                                        .spacing(8.)
                                        .horizontal()
                                        .child(
                                            StoatButton::new()
                                                .corner_radius(18.)
                                                .on_press({
                                                    let code = invite.code.clone();

                                                    move |_| {
                                                        Clipboard::set(format!(
                                                            "https://stt.gg/{code}"
                                                        ))
                                                        .unwrap();
                                                    }
                                                })
                                                .child(
                                                    rect()
                                                        .width(Size::px(36.))
                                                        .height(Size::px(36.))
                                                        .background(theme.md.secondary_container.as_argb_u32())
                                                        .color(theme.md.on_secondary_container.as_argb_u32())
                                                        .center()
                                                        .child(
                                                            svg(copy())
                                                                .width(Size::px(24.))
                                                                .height(Size::px(24.)),
                                                        ),
                                                ),
                                        )
                                        .child(
                                            StoatButton::new()
                                                .corner_radius(18.)
                                                .on_press({
                                                    let code = invite.code.clone();

                                                    move |_| {
                                                        modals.write().push_modal(
                                                            ModalValue::DeleteInvite {
                                                                invite: code.clone(),
                                                            },
                                                        )
                                                    }
                                                })
                                                .child(
                                                    rect()
                                                        .width(Size::px(36.))
                                                        .height(Size::px(36.))
                                                        .background(theme.md.primary.as_argb_u32())
                                                        .color(theme.md.on_primary.as_argb_u32())
                                                        .center()
                                                        .child(
                                                            svg(trash())
                                                                .width(Size::px(24.))
                                                                .height(Size::px(24.)),
                                                        ),
                                                ),
                                        ),
                                )
                                .into_element()
                        }))
                    }),
            )
    }
}
