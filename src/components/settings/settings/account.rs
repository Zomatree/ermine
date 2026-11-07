use std::borrow::Cow;

use freya::{prelude::*, radio::use_radio};
use jiff::{Timestamp, tz::TimeZone};
use stoat_models::v0;

use crate::{
    AppChannel, AppState, Config, SettingsPage, SizeExt,
    components::{
        Avatar, CategoryButton, CategoryButtonGroup, MaterialIcon, ModalValue, StoatButton,
        StoatButtonLayoutThemePartialExt, StoatTooltip, file_image,
        material::{
            filled::cake,
            outlined::{
                alternate_email, block, delete, edit, email, lock, password, privacy_tip,
                verified_user,
            },
        },
        stoat_label, use_modals,
    },
    consume_material_theme, http, use_config,
};

#[derive(PartialEq)]
pub struct AccountSettings {}

impl Component for AccountSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut config = use_config();
        let mut modals = use_modals();

        let mut radio = use_radio(AppChannel::UserId);
        let servers = radio.slice(AppChannel::Servers, |state| &state.servers);
        let user = radio.slice(AppChannel::Users, move |state| {
            state.users.get(state.user_id.as_ref().unwrap()).unwrap()
        });
        let mut settings_page =
            radio.slice_mut(AppChannel::SettingsPage, |state| &mut state.settings_page);

        let mut profile = use_state(|| None);
        let mut mfa = use_state(|| v0::MultiFactorStatus::default());
        let account_email = use_state(|| None::<String>);

        let user_value = user.read();

        use_hook(|| {
            let user_id = user_value.id.clone();

            spawn(async move {
                if let Ok(user_profile) = http().fetch_user_profile(&user_id).await {
                    profile.set(Some(user_profile));
                }
            });

            spawn(async move {
                if let Ok(status) = http().fetch_mfa_status().await {
                    mfa.set(status);
                }
            });
        });

        let does_own_servers = servers.read().values().any(|s| &s.owner == &user_value.id);

        rect()
            .spacing(15.)
            .child(
                rect()
                    .overflow(Overflow::Clip)
                    .corner_radius(28.)
                    .height(Size::px(118.))
                    .width(Size::Fill)
                    .background(theme.md.primary_container)
                    .maybe_child(
                        profile
                            .read()
                            .as_ref()
                            .and_then(|p| p.background.as_ref())
                            .map(|background| {
                                rect()
                                    .height(Size::px(118.))
                                    .width(Size::Fill)
                                    .child(
                                        file_image(background)
                                            .aspect_ratio(AspectRatio::Max)
                                            .image_cover(ImageCover::Center)
                                            .expanded(),
                                    )
                                    .child(
                                        rect()
                                            .layer(2)
                                            .position(Position::new_absolute().top(0.).left(0.))
                                            .height(Size::px(118.))
                                            .width(Size::Fill)
                                            .background(
                                                Color::from(theme.md.surface_container_low)
                                                    .with_af32(0.7),
                                            ),
                                    )
                            }),
                    )
                    .child(
                        rect()
                            .position(Position::new_absolute().top(0.).left(0.))
                            .padding(15.)
                            .layer(Layer::Relative(6))
                            .width(Size::Fill)
                            .height(Size::px(118.))
                            .child(
                                rect()
                                    .content(Content::Flex)
                                    .horizontal()
                                    .height(Size::px(58.))
                                    .spacing(15.)
                                    .cross_align(Alignment::Center)
                                    .child(Avatar::new(user.clone().into_readable(), None, 58.))
                                    .child(
                                        rect()
                                            .color(theme.md.on_secondary_container)
                                            .width(Size::flex(1.))
                                            .child(
                                                stoat_label().font_size(18.).font_weight(600).text(
                                                    user_value
                                                        .display_name
                                                        .as_ref()
                                                        .unwrap_or(&user_value.username)
                                                        .clone(),
                                                ),
                                            )
                                            .child(stoat_label().font_size(14.).text(format!(
                                                "{}#{}",
                                                user_value.username, user_value.discriminator
                                            ))),
                                    )
                                    .child(
                                        StoatButton::new()
                                            .corner_radius(12.)
                                            .on_press(move |_| {
                                                settings_page.set(Some(SettingsPage::Profile));
                                            })
                                            .child(
                                                rect()
                                                    .background(theme.md.primary)
                                                    .color(theme.md.on_primary)
                                                    .padding(8.)
                                                    .child(
                                                        MaterialIcon::new(edit())
                                                            .size(Size::px(24.)),
                                                    ),
                                            ),
                                    ),
                            )
                            .child(
                                rect().horizontal().child(
                                    rect().margin((0., 0., 0., 73.)).child(
                                        StoatTooltip::new(
                                            label().max_lines(1).font_size(11.).text({
                                                let datetime = Timestamp::try_from(
                                                    ulid::Ulid::from_string(&user_value.id)
                                                        .unwrap()
                                                        .datetime(),
                                                )
                                                .unwrap()
                                                .to_zoned(TimeZone::system());

                                                format!(
                                                    "Account created {}.",
                                                    datetime.strftime("%d/%m/%Y at %H:%M")
                                                )
                                            }),
                                        )
                                        .position(AttachedPosition::Top)
                                        .child(
                                            rect()
                                                .background(theme.md.secondary)
                                                .color(theme.md.on_secondary)
                                                .corner_radius(12.)
                                                .padding(8.)
                                                .child(
                                                    MaterialIcon::new(cake()).size(Size::px(14.)),
                                                ),
                                        ),
                                    ),
                                ),
                            ),
                    ),
            )
            .child(
                CategoryButtonGroup::new()
                    .child(
                        CategoryButton::new(alternate_email())
                            .chevron()
                            .on_press(move |_| {
                                modals.write().push_modal(ModalValue::ChangeUsername {});
                            })
                            .title(stoat_label().text("Username"))
                            .description(stoat_label().text(user_value.username.clone())),
                    )
                    .child(
                        CategoryButton::new(email())
                            .chevron()
                            .on_press(move |_| {
                                modals.write().push_modal(ModalValue::ChangeEmail {});
                            })
                            .title(stoat_label().text("Email"))
                            .description(
                                rect()
                                    .horizontal()
                                    .spacing(8.)
                                    .child(stoat_label().font_size(12.).text(
                                        if let Some(email) = account_email.read().cloned() {
                                            Cow::Owned(email)
                                        } else {
                                            "•••••••••••@•••••••••••".into()
                                        },
                                    ))
                                    .maybe_child(account_email.read().is_none().then(|| {
                                        RevealEmail {
                                            account_email,
                                        }
                                    })),
                            ),
                    )
                    .child(
                        CategoryButton::new(password())
                            .chevron()
                            .on_press(move |_| {
                                modals.write().push_modal(ModalValue::ChangePassword {});
                            })
                            .title(stoat_label().text("Password"))
                            .description(stoat_label().text("•••••••••")),
                    ),
            )
            .child({
                let mfa_ref = mfa.read();

                CategoryButtonGroup::new()
                    .with(|this| {
                        if mfa_ref.recovery_active {
                            this
                                .child(
                                    CategoryButton::new(privacy_tip())
                                        .chevron()
                                        .on_press(move |_| {
                                            modals.clone().write().push_modal(ModalValue::MFA {
                                                callback: EventHandler::new(move |ticket: v0::MFATicket| {
                                                    spawn_in_window(async move {
                                                        if let Ok(codes) = http().fetch_recovery_codes(ticket.token).await {
                                                            modals.write().push_modal(ModalValue::ViewRecoveryCodes {
                                                                codes
                                                            });
                                                        }
                                                    });
                                                })
                                            });
                                        })
                                        .title(stoat_label().text("View Recovery Codes"))
                                        .description(stoat_label().text("Get active recovery codes"))
                                )
                                .child(
                                    CategoryButton::new(verified_user())
                                        .chevron()
                                        .on_press(move |_| {
                                            modals.clone().write().push_modal(ModalValue::MFA {
                                                callback: EventHandler::new(move |ticket: v0::MFATicket| {
                                                    spawn_in_window(async move {
                                                        if let Ok(codes) = http().generate_recovery_codes(ticket.token).await {
                                                            modals.write().push_modal(ModalValue::ViewRecoveryCodes {
                                                                codes
                                                            });
                                                        }
                                                    });
                                                })
                                            });
                                        })
                                        .title(stoat_label().text("Reset Recovery Codes"))
                                        .description(stoat_label().text("Get a new set of recovery codes"))
                                )
                        } else {
                            this
                                .child(
                                    CategoryButton::new(verified_user())
                                        .chevron()
                                        .on_press(move |_| {
                                            modals.clone().write().push_modal(ModalValue::MFA {
                                                callback: EventHandler::new(move |ticket: v0::MFATicket| {
                                                    spawn_in_window(async move {
                                                        if let Ok(codes) = http().generate_recovery_codes(ticket.token).await {
                                                            modals.write().push_modal(ModalValue::ViewRecoveryCodes {
                                                                codes
                                                            });
                                                        }
                                                    });
                                                })
                                            });
                                        })
                                        .title(stoat_label().text("Generate Recovery Codes"))
                                        .description(stoat_label().text("Setup recovery codes"))
                                )
                        }
                    })
                    .child(if mfa_ref.totp_mfa {
                        CategoryButton::new(lock())
                            .chevron()
                            .on_press(move |_| {
                                modals.clone().write().push_modal(ModalValue::MFA {
                                    callback: EventHandler::new(move |ticket: v0::MFATicket| {
                                        spawn_in_window(async move {
                                            if http().disable_totp(ticket.token).await.is_ok() {
                                                mfa.write().totp_mfa = false;
                                            }
                                        });
                                    })
                                });
                            })
                            .title(stoat_label().text("Remove Authenticator"))
                            .description(stoat_label().text("Disable one-time password authenticator"))
                    } else {
                        CategoryButton::new(lock())
                            .chevron()
                            .on_press(move |_| {
                                modals.clone().write().push_modal(ModalValue::MFA {
                                    callback: EventHandler::new(move |ticket: v0::MFATicket| {
                                        spawn_in_window(async move {
                                            if let Ok(secret) = http().generate_totp_secret(ticket.token).await {
                                                modals.write().push_modal(ModalValue::EnableTotp {
                                                    secret: secret.secret,
                                                    callback: EventHandler::new(move |_| {
                                                        mfa.write().totp_mfa = true;
                                                    })
                                                });
                                            }
                                        });
                                    })
                                });
                            })
                            .title(stoat_label().text("Enable Authenticator"))
                            .description(stoat_label().text("Setup one-time password authenticator"))
                    })
            })
            .child(
                CategoryButtonGroup::new()
                    .child(
                        CategoryButton::new(block())
                            .chevron()
                            .icon_color(theme.md.error)
                            .on_press(move |_| {
                                modals.clone().write().push_modal(ModalValue::MFA {
                                    callback: EventHandler::new(move |ticket: v0::MFATicket| {
                                        spawn_in_window(async move {
                                            if http().disable_account(Some(ticket.token)).await.is_ok() {
                                                config.set(Config::default());
                                                radio.set(AppState::new());

                                                http().logout().await.unwrap();
                                            }
                                        });
                                    })
                                });
                            })
                            .title(stoat_label().text("Disable Account"))
                            .description(stoat_label().text("You won't be able to access your account unless you contact support - however, your data will not be deleted."))
                    )
                    .child(
                        CategoryButton::new(delete())
                            .chevron()
                            .icon_color(theme.md.error)
                            .enabled(!does_own_servers)
                            .on_press(move |_| {
                                modals.clone().write().push_modal(ModalValue::MFA {
                                    callback: EventHandler::new(move |ticket: v0::MFATicket| {
                                        spawn_in_window(async move {
                                            if http().delete_account(Some(ticket.token)).await.is_ok() {
                                                config.set(Config::default());
                                                radio.set(AppState::new());

                                                http().logout().await.unwrap();
                                            }
                                        });
                                    })
                                });
                            })
                            .title(
                                stoat_label()
                                    .text(if does_own_servers {
                                        "Cannot delete account until your servers are deleted or transferred"
                                    } else {
                                        "Delete Account"
                                    })
                            )
                            .description(stoat_label().text("Your account and all of your data (including your messages and friends list) will be queued for deletion. A confirmation email will be sent - you can cancel this within 7 days by contacting support."))
                    ),
            )
    }
}

#[derive(PartialEq)]
struct RevealEmail {
    pub account_email: State<Option<String>>,
}

impl Component for RevealEmail {
    fn render(&self) -> impl IntoElement {
        let mut account_email = self.account_email;

        let mut hovering = use_state(|| false);

        stoat_label()
            .font_size(12.)
            .cursor(CursorIcon::Pointer)
            .text_decoration(if hovering() {
                TextDecoration::Underline
            } else {
                TextDecoration::None
            })
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .on_press(move |evt: Event<PressEventData>| {
                evt.stop_propagation();

                spawn(async move {
                    if let Ok(account) = http().fetch_account().await {
                        account_email.set(Some(account.email));
                    }
                });
            })
            .text("Reveal")
    }
}
