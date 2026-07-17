use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel, SizeExt,
    components::{
        Avatar, MaterialIcon, StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip,
        material::{filled::cake, outlined::edit},
    },
    consume_material_theme,
    types::Account,
};

#[derive(PartialEq)]
pub struct AccountSettings {}

impl Component for AccountSettings {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());
        let user = radio.slice(AppChannel::Users, move |state| {
            state.users.get(&*user_id.read()).unwrap()
        });

        let theme = consume_material_theme();

        let account = use_state(|| None::<Account>);
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
                            .content(Content::Flex)
                            .horizontal()
                            .height(Size::px(58.))
                            .spacing(15.)
                            .cross_align(Alignment::Center)
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
                            )
                            .child(
                                StoatButton::new().corner_radius(12.).child(
                                    rect()
                                        .background(theme.md.primary.as_argb_u32())
                                        .color(theme.md.on_primary.as_argb_u32())
                                        .padding(8.)
                                        .child(MaterialIcon::new(edit()).size(Size::px(24.))),
                                ),
                            ),
                    )
                    .child(
                        rect().horizontal().child(
                            rect().margin((0., 0., 0., 73.)).child(
                                StoatTooltip::new(
                                    label()
                                        .max_lines(1)
                                        .font_size(11.)
                                        .text("Account created some time long ago."),
                                )
                                .position(AttachedPosition::Top)
                                .child(
                                    rect()
                                        .background(theme.md.primary.as_argb_u32())
                                        .color(theme.md.on_primary.as_argb_u32())
                                        .corner_radius(12.)
                                        .padding(8.)
                                        .child(MaterialIcon::new(cake()).size(Size::px(14.))),
                                ),
                            ),
                        ),
                    ),
            )
            .child(rect())
    }
}
