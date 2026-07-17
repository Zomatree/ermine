use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel, SizeExt,
    components::{
        Avatar, MaterialIcon,
        material::{filled::expand_more, outlined::groups},
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct ProfileSettings {}

impl Component for ProfileSettings {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());
        let user = radio.slice(AppChannel::Users, move |state| {
            state.users.get(&*user_id.read()).unwrap()
        });

        let theme = consume_material_theme();

        let user_value = user.read();

        rect()
            .spacing(15.)
            .child(
                rect()
                    .padding(15.)
                    .corner_radius(28.)
                    .background(theme.md.primary_container.as_argb_u32())
                    .child(
                        rect()
                            .horizontal()
                            .height(Size::px(58.))
                            .spacing(15.)
                            .cross_align(Alignment::Center)
                            .content(Content::Flex)
                            .child(Avatar::new(user.clone().into_readable(), None, 58.))
                            .child(
                                rect()
                                    .color(theme.md.on_secondary_container.as_argb_u32())
                                    .height(Size::Fill)
                                    .width(Size::flex(1.))
                                    .main_align(Alignment::SpaceAround)
                                    .child(
                                        label()
                                            .font_size(18.)
                                            .font_weight(FontWeight::SEMI_BOLD)
                                            .line_height(1.5)
                                            .text(
                                                user_value
                                                    .display_name
                                                    .as_ref()
                                                    .unwrap_or(&user_value.username)
                                                    .clone(),
                                            ),
                                    )
                                    .child(label().font_size(14.).line_height(1.5).text(format!(
                                        "{}#{}",
                                        user_value.username, user_value.discriminator
                                    ))),
                            ),
                    ),
            )
            .child(
                rect()
                    .padding(13.)
                    .corner_radius(28.)
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
                                    .child(MaterialIcon::new(groups()).size(Size::px(22.))),
                            )
                            .child(
                                rect()
                                    .color(theme.md.on_secondary_container.as_argb_u32())
                                    .width(Size::flex(1.))
                                    .child(
                                        label()
                                            .font_size(14.)
                                            .font_weight(FontWeight::SEMI_BOLD)
                                            .line_height(1.5)
                                            .text("Server Identities"),
                                    )
                                    .child(
                                        label()
                                            .font_size(12.)
                                            .line_height(1.5)
                                            .text("Change your profile per-server"),
                                    ),
                            )
                            .child(MaterialIcon::new(expand_more()).size(Size::px(18.))),
                    ),
            )
    }
}
