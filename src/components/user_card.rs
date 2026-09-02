use std::rc::Rc;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{
        ProfileBadges, ProfileBanner, ProfileBio, ProfileButtons, ProfileJoined, ProfilePronouns,
        ProfileRoles, ProfileStatus, StoatButton, StoatButtonLayoutThemePartialExt, use_floating,
    },
    consume_material_theme, http,
};

#[derive(PartialEq)]
pub struct UserCard {
    pub user: Readable<v0::User>,
    pub member: Option<Readable<v0::Member>>,
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

        let profile = use_state(|| None);

        use_future({
            let user_id = self.user.peek().id.clone();

            move || {
                let user_id = user_id.clone();
                let mut profile = profile.clone();

                async move {
                    if let Ok(user_profile) = http().fetch_user_profile(&user_id).await {
                        profile.set(Some(user_profile));
                    }
                }
            }
        });

        let mut floating = use_floating();

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
                                .on_press(move |_| {
                                    floating.set(None);
                                    let user_id = user.read().id.clone();
                                    open_profile.clone().set(Some(user_id));
                                }),
                        )
                        .child(ProfileButtons {
                            user: self.user.clone(),
                            close: Rc::new(move || floating.clone().set(None)),
                        })
                        .maybe_child({
                            (self
                                .member
                                .as_ref()
                                .is_some_and(|member| !member.read().roles.is_empty())
                                || self.user.read().badges != 0)
                                .then(|| {
                                    rect()
                                        .horizontal()
                                        .spacing(8.)
                                        .content(Content::Flex)
                                        .maybe_child(
                                            self.member
                                                .clone()
                                                .filter(|member| !member.read().roles.is_empty())
                                                .map(|member| ProfileRoles { member }),
                                        )
                                        .maybe_child({
                                            let badges = self.user.read().badges;

                                            (badges != 0).then(|| ProfileBadges { badges })
                                        })
                                })
                        })
                        .child(
                            rect()
                                .horizontal()
                                .spacing(8.)
                                .content(Content::Flex)
                                .maybe_child(
                                    user.read()
                                        .status
                                        .as_ref()
                                        .and_then(|status| status.text.clone())
                                        .map(|text| ProfileStatus { text }),
                                )
                                .child(ProfileJoined {
                                    user: self.user.clone(),
                                    member: self.member.clone(),
                                }),
                        )
                        .maybe_child(
                            (self
                                .member
                                .as_ref()
                                .is_some_and(|m| m.read().pronouns.is_some())
                                || self.user.read().pronouns.is_some())
                            .then(|| {
                                ProfilePronouns {
                                    user: self.user.clone(),
                                    member: self.member.clone(),
                                }
                                .into_element()
                            }),
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
