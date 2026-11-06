use std::rc::Rc;

use freya::{prelude::*, radio::use_radio};
use itertools::Itertools;
use stoat_models::v0;
use stoat_permissions::PermissionValue;

use crate::{
    AppChannel,
    components::{
        ProfileBadges, ProfileBanner, ProfileBio, ProfileButtons, ProfileJoined,
        ProfilePronouns, ProfileRoles,
        ProfileStatus, StoatButton, StoatButtonLayoutThemePartialExt, row, use_floating,
    },
    consume_material_theme, http,
};

#[derive(PartialEq)]
pub struct UserCard {
    pub user: Readable<v0::User>,
    pub member: Option<Readable<v0::Member>>,
    pub permissions: PermissionValue,
}

impl Component for UserCard {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserProfile);
        let open_profile = radio.slice_mut_current(|state| &mut state.user_profile);
        let theme = consume_material_theme();

        let user = use_memo({
            let user = self.user.clone();
            move || user.read().clone()
        });

        let mut profile = use_state(|| None);

        use_future({
            let user_id = self.user.peek().id.clone();

            move || {
                let user_id = user_id.clone();

                async move {
                    if let Ok(user_profile) = http().fetch_user_profile(&user_id).await {
                        profile.set(Some(user_profile));
                    }
                }
            }
        });

        let mut floating = use_floating();

        let mut tiles = Vec::new();

        let user_ref = user.read();
        let member_ref = self.member.as_ref().map(|m| m.read());


        if let Some(member) = &self.member
            && member_ref.as_ref().is_some_and(|m| !m.roles.is_empty())
        {
            tiles.push(
                ProfileRoles {
                    member: member.clone(),
                }
                .into_element(),
            );
        }

        if user_ref.badges != 0 {
            tiles.push(
                ProfileBadges {
                    badges: user_ref.badges,
                }
                .into_element(),
            )
        }

        if let Some(text) = user_ref.status.as_ref().and_then(|s| s.text.clone()) {
            tiles.push(ProfileStatus { text, member: self.member.clone() }.into_element());
        }

        tiles.push(
            ProfileJoined {
                user: user.into_readable(),
                member: self.member.clone(),
            }
            .into_element(),
        );

        if user_ref.pronouns.is_some() || member_ref.as_ref().is_some_and(|m| m.pronouns.is_some())
        {
            tiles.push(
                ProfilePronouns {
                    user: user.into_readable(),
                    member: self.member.clone(),
                }
                .into_element(),
            )
        }

        rect()
            .corner_radius(28.)
            .overflow(Overflow::Clip)
            .background(theme.md.surface_container_high.as_argb_u32())
            .shadow(Shadow::new().blur(3.).color(theme.md.shadow.as_argb_u32()))
            .width(Size::px(340.))
            .height(Size::px(400.))
            .child(
                ScrollView::new().show_scrollbar(false).child(
                    rect()
                        .padding(8.)
                        .spacing(8.)
                        .content(Content::Flex)
                        .child(
                            StoatButton::new()
                                .corner_radius(28.)
                                .child(rect().interactive(false).child(ProfileBanner {
                                    user: self.user.clone(),
                                    member: self.member.clone(),
                                    profile: profile.into_readable(),
                                }))
                                .on_press({
                                    let member = self.member.clone();
                                    move |_| {
                                        floating.set(None);
                                        let user_id = user.read().id.clone();
                                        open_profile.clone().set(Some((
                                            user_id,
                                            member.as_ref().map(|m| m.read().id.server.clone()),
                                        )));
                                    }
                                }),
                        )
                        .child(ProfileButtons {
                            user: self.user.clone(),
                            member: self.member.clone(),
                            permissions: self.permissions,
                            close: Rc::new(move || floating.clone().set(None)),
                        })
                        .children(
                            tiles
                                .into_iter()
                                .chunks(2)
                                .into_iter()
                                .map(|tiles| row().children(tiles)),
                        )
                        .maybe_child(
                            profile
                                .read()
                                .as_ref()
                                .and_then(|profile| profile.content.clone())
                                .map(|bio| ProfileBio { bio }),
                        ),
                ),
            )
    }
}
