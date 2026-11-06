use freya::{prelude::*, radio::use_radio};
use qrcode_generator::{
    Renderer,
    qr::{Encoder, ErrorCorrection},
};
use stoat_models::v0;

use crate::{
    AppChannel, SizeExt,
    components::{Dialog, SingleLineEntry, stoat_label, use_modals},
    consume_material_theme, format_error, http,
};

#[derive(PartialEq)]
pub struct EnableTotp {
    pub secret: String,
    pub callback: EventHandler<()>,
}

impl Component for EnableTotp {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let radio = use_radio(AppChannel::Users);

        let user =
            radio.slice_current(|state| state.users.get(state.user_id.as_ref().unwrap()).unwrap());

        let code = use_state(String::new);
        let mut error = use_state(|| None);

        let qr_code = use_hook(|| {
            let uri = format!(
                "otpauth://totp/Stoat:{}?secret={}&issuer=Stoat",
                user.read().username,
                self.secret
            );

            let symbol = Encoder::new(ErrorCorrection::Medium)
                .encode_text(&uri)
                .unwrap();

            let svg = Renderer::new(&symbol, 156)
                .quiet_zone(0)
                .svg_xml_declaration(false)
                .to_svg_string(None::<&str>)
                .unwrap();

            Bytes::from(svg)
        });

        Dialog::new()
            .title(label().line_height(1.5).text("Enable authenticator app"))
            .body(
                rect()
                    .spacing(8.)
                    .child(
                        rect()
                            .spacing(8.)
                            .width(Size::Fill)
                            .cross_align(Alignment::Center)
                            .child(stoat_label().text(
                                "Please scan or use the token below in your authenticator app.",
                            ))
                            .child(
                                rect()
                                    .padding(4.)
                                    .corner_radius(4.)
                                    .background(Color::WHITE)
                                    .child(
                                        SvgViewer::new((&self.secret, qr_code))
                                            .size(Size::px(156.)),
                                    ),
                            )
                            .child(
                                SelectableText::new()
                                    .line_height(1.5)
                                    .text_height(TextHeightBehavior::All)
                                    .font_family("Fira Code Medium")
                                    .letter_spacing(0.25)
                                    .span(self.secret.clone()),
                            ),
                    )
                    .child(
                        SingleLineEntry::new("Enter Code", code)
                            .placeholder("Enter Code from your authenticator app..."),
                    )
                    .maybe_child(
                        error
                            .read()
                            .cloned()
                            .map(|error| stoat_label().color(theme.md.error).text(error)),
                    ),
            )
            .default_action("Cancel")
            .action("Continue", {
                let callback = self.callback.clone();

                move || {
                    let code = code.read().cloned();
                    let callback = callback.clone();

                    spawn(async move {
                        match http()
                            .enable_totp(&v0::MFAResponse::Totp { totp_code: code })
                            .await
                        {
                            Ok(_) => {
                                callback.call(());
                                modals.write().pop_modal();
                            }
                            Err(e) => {
                                error.set(Some(format_error(&e, "mfa")));
                            }
                        }
                    });

                    false
                }
            })
    }
}
