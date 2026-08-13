use freya::prelude::*;

use crate::{
    Config, SizeExt,
    components::{
        MaterialIcon, ModalValue, SingleLineEntry, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt,
        material::filled::{clear, dark_mode, edit, info},
        use_modals,
    },
    consume_material_theme, format_error, http,
    types::{DataLogin, MFAResponse, ResponseLogin},
};

#[derive(PartialEq)]
pub struct Login {}

impl Component for Login {
    fn render(&self) -> impl IntoElement {
        let mut config = use_consume::<State<Config>>();
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let email = use_state(String::new);
        let password = use_state(String::new);
        let mut error = use_state(|| None::<String>);

        let mut mfa_ticket = use_state(|| None::<String>);
        let mut mfa_value = use_state(String::new);
        let mut mfa_error = use_state(|| None::<String>);

        let submit = {
            move || {
                let email = email.read().clone();
                let password = password.read().clone();

                if email.is_empty() || password.is_empty() {
                    return;
                };

                spawn(async move {
                    match http()
                        .login(&DataLogin::Email {
                            email,
                            password,
                            friendly_name: Some("Ermine".to_string()),
                        })
                        .await
                    {
                        Ok(response) => match response {
                            ResponseLogin::Success(session) => {
                                config.write().session = Some(session)
                            }
                            ResponseLogin::MFA { ticket, .. } => {
                                mfa_ticket.set(Some(ticket));
                            }
                            ResponseLogin::Disabled { .. } => {
                                error.set(Some("Disabled Account".to_string()))
                            }
                        },
                        Err(e) => error.set(Some(format_error(&e, "Account"))),
                    }
                });
            }
        };

        rect()
            .width(Size::Fill)
            .height(Size::Fill)
            .background(theme.md.surface.as_argb_u32())
            .center()
            .child(
                rect()
                    .width(Size::Fill)
                    .height(Size::Fill)
                    .padding((40., 35., 35., 35.))
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::SpaceBetween)
                    .child(
                        rect()
                            .width(Size::Fill)
                            .horizontal()
                            .main_align(Alignment::End)
                            .child(
                                StoatButton::new()
                                    .background(theme.md.secondary_container.as_argb_u32())
                                    .color(theme.md.on_secondary_container.as_argb_u32())
                                    .corner_radius(40.)
                                    .on_press(move |_| config.write().theme.scheme.toggle())
                                    .child(
                                        rect().size(Size::px(40.)).center().child(
                                            MaterialIcon::new(dark_mode()).size(Size::px(24.)),
                                        ),
                                    ),
                            ),
                    )
                    .child(
                        rect()
                            .max_width(Size::px(360.))
                            .max_height(Size::px(600.))
                            .padding((45., 40.))
                            .corner_radius(32.)
                            .background(theme.md.surface_container.as_argb_u32())
                            .spacing(15.)
                            .child(
                                rect()
                                    .spacing(8.)
                                    .child(label().text("👋 Welcome!").font_size(22))
                                    .child(label().text("Sign into Stoat").font_size(16)),
                            )
                            .child(
                                rect()
                                    .spacing(15.)
                                    .cross_align(Alignment::Center)
                                    .child(
                                        SingleLineEntry::new("Email", email)
                                            .placeholder("Please enter your email.")
                                            .width(Size::px(280.))
                                            .on_submit(move |_| submit()),
                                    )
                                    .child(
                                        SingleLineEntry::new("Password", password)
                                            .placeholder("Enter your current password.")
                                            .mode(InputMode::Hidden('•'))
                                            .width(Size::px(280.))
                                            .on_submit(move |_| submit()),
                                    ),
                            )
                            .child(
                                rect()
                                    .width(Size::px(280.))
                                    .cross_align(Alignment::Center)
                                    .color(theme.md.primary.as_argb_u32())
                                    .spacing(32.)
                                    .child(
                                        StoatButton::new()
                                            .color(theme.md.primary.as_argb_u32())
                                            .corner_radius(40.)
                                            .child(
                                                rect()
                                                    .height(Size::px(40.))
                                                    .padding((0., 16.))
                                                    .center()
                                                    .child("Reset password"),
                                            ),
                                    )
                                    .child(
                                        StoatButton::new()
                                            .color(theme.md.primary.as_argb_u32())
                                            .corner_radius(40.)
                                            .child(
                                                rect()
                                                    .height(Size::px(40.))
                                                    .padding((0., 16.))
                                                    .center()
                                                    .child("Resend verification"),
                                            ),
                                    ),
                            )
                            .child(
                                rect()
                                    .width(Size::px(280.))
                                    .horizontal()
                                    .spacing(8.)
                                    .main_align(Alignment::Center)
                                    .child(
                                        StoatButton::new()
                                            .color(theme.md.primary.as_argb_u32())
                                            .corner_radius(40.)
                                            .child(
                                                rect()
                                                    .padding((0., 16.))
                                                    .height(Size::px(40.))
                                                    .horizontal()
                                                    .center()
                                                    .spacing(4.)
                                                    .child(
                                                        MaterialIcon::new(clear())
                                                            .size(Size::px(12.)),
                                                    )
                                                    .child("Exit"),
                                            )
                                            .on_press(|_| {
                                                let platform = Platform::get();
                                                Platform::get().with_window(None, move |window| {
                                                    platform.close_window(window.id());
                                                });
                                            }),
                                    )
                                    .child(
                                        StoatButton::new()
                                            .child(
                                                rect()
                                                    .height(Size::px(40.))
                                                    .padding((0., 16.))
                                                    .center()
                                                    .child("Login"),
                                            )
                                            .background(theme.md.primary.as_argb_u32())
                                            .color(theme.md.on_primary.as_argb_u32())
                                            .corner_radius(40.)
                                            .on_press(move |_| submit()),
                                    ),
                            )
                            .maybe_child(error.read().clone().map(|error| {
                                rect()
                                    .width(Size::Fill)
                                    .horizontal()
                                    .center()
                                    .spacing(4.)
                                    .color(theme.md.error.as_argb_u32())
                                    .child(MaterialIcon::new(info()).size(Size::px(16.)))
                                    .child(label().font_size(11.).text(error))
                            })),
                    )
                    .child(
                        rect()
                            .width(Size::Fill)
                            .horizontal()
                            .main_align(Alignment::SpaceBetween)
                            .cross_align(Alignment::End)
                            .color(theme.md.on_surface_variant.as_argb_u32())
                            .child("Developed by Zomatree")
                            .child(
                                StoatButton::new()
                                    .background(theme.md.secondary_container.as_argb_u32())
                                    .color(theme.md.on_secondary_container.as_argb_u32())
                                    .corner_radius(40.)
                                    .on_press(move |_| {
                                        modals.write().push_modal(ModalValue::EditApi)
                                    })
                                    .child(
                                        rect()
                                            .size(Size::px(40.))
                                            .center()
                                            .child(MaterialIcon::new(edit()).size(Size::px(24.))),
                                    ),
                            ),
                    ),
            )
            .maybe_child(mfa_ticket.read().cloned().map(|ticket| {
                let submit = move || {
                    let value = mfa_value.read().clone();
                    let ticket = ticket.clone();

                    spawn(async move {
                        match http()
                            .login(&DataLogin::MFA {
                                mfa_ticket: ticket,
                                mfa_response: Some(MFAResponse::Totp { totp_code: value }),
                                friendly_name: Some("Ermine".to_string()),
                            })
                            .await
                        {
                            Ok(response) => match response {
                                ResponseLogin::Success(session) => {
                                    mfa_ticket.set(None);
                                    mfa_value.set(String::new());
                                    mfa_error.set(None);
                                    config.write().session = Some(session)
                                }
                                _ => unreachable!(),
                            },
                            Err(e) => mfa_error.set(Some(format_error(&e, "2FA"))),
                        }
                    });
                };

                Popup::new()
                    .background(theme.md.surface_container_high.as_argb_u32())
                    .color(theme.md.on_surface.as_argb_u32())
                    .width(Size::px(370.))
                    .on_close_request(move |_| {
                        mfa_ticket.set(None);
                        mfa_value.set(String::new());
                        mfa_error.set(None);
                    })
                    .child(
                        rect().font_size(24.).padding(8.).child(
                            label()
                                .a11y_role(AccessibilityRole::TitleBar)
                                .width(Size::fill())
                                .text("Confirm action"),
                        ),
                    )
                    .child(
                        rect()
                            .spacing(8.)
                            .padding(8.)
                            .child(
                                label()
                                    .font_size(14.)
                                    .color(theme.md.on_surface_variant.as_argb_u32())
                                    .text("Please confirm this action using the selected method."),
                            )
                            .child(
                                SingleLineEntry::new("Authenticator App", mfa_value).on_submit({
                                    let submit = submit.clone();
                                    move |_| submit()
                                }),
                            )
                            .maybe_child(mfa_error.read().clone().map(|error| {
                                rect()
                                    .width(Size::Fill)
                                    .horizontal()
                                    .center()
                                    .spacing(4.)
                                    .color(theme.md.error.as_argb_u32())
                                    .child(MaterialIcon::new(info()).size(Size::px(16.)))
                                    .child(label().font_size(11.).text(error))
                            })),
                    )
                    .child(
                        PopupButtons::new()
                            .child(
                                StoatButton::new()
                                    .corner_radius(40.)
                                    .child(
                                        rect()
                                            .center()
                                            .color(theme.md.primary.as_argb_u32())
                                            .child("Back")
                                            .height(Size::px(40.))
                                            .padding((0., 16.)),
                                    )
                                    .on_press(move |_| {
                                        mfa_ticket.set(None);
                                        mfa_value.set(String::new());
                                        mfa_error.set(None);
                                    }),
                            )
                            .child(
                                StoatButton::new()
                                    .corner_radius(40.)
                                    .child(
                                        rect()
                                            .center()
                                            .color(theme.md.primary.as_argb_u32())
                                            .child("Confirm")
                                            .height(Size::px(40.))
                                            .padding((0., 16.)),
                                    )
                                    .on_press(move |_| submit()),
                            ),
                    )
            }))
    }
}
