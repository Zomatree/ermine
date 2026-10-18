use freya::{prelude::*, radio::use_radio};
use jiff::{Timestamp, tz::TimeZone};

use crate::{
    AppChannel, SettingsPage, SizeExt,
    components::{
        Avatar, MaterialIcon, StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip,
        material::{filled::cake, outlined::edit},
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct AccountSettings {}

impl Component for AccountSettings {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());
        let user = radio.slice(AppChannel::Users, move |state| {
            state.users.get(state.user_id.as_ref().unwrap()).unwrap()
        });
        let mut settings_page =
            radio.slice_mut(AppChannel::SettingsPage, |state| &mut state.settings_page);

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
                            .content(Content::Flex)
                            .horizontal()
                            .height(Size::px(58.))
                            .spacing(15.)
                            .cross_align(Alignment::Center)
                            .child(Avatar::new(user.clone().into_readable(), None, 58.))
                            .child(
                                rect()
                                    .color(theme.md.on_secondary_container.as_argb_u32())
                                    .width(Size::flex(1.))
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
                                StoatButton::new()
                                    .corner_radius(12.)
                                    .on_press(move |_| {
                                        settings_page.set(Some(SettingsPage::Profile));
                                    })
                                    .child(
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
                                StoatTooltip::new(label().max_lines(1).font_size(11.).text({
                                    let datetime = Timestamp::try_from(
                                        ulid::Ulid::from_string(&user_id.read())
                                            .unwrap()
                                            .datetime(),
                                    )
                                    .unwrap()
                                    .to_zoned(TimeZone::system());

                                    format!(
                                        "Account created {}.",
                                        datetime.strftime("%d/%m/%Y at %H:%M")
                                    )
                                }))
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
