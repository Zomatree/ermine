use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, SizeExt, Tag,
    components::{
        Avatar, MaterialIcon, SingleLineEntry, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt, file_image,
        material::{
            filled::expand_more,
            outlined::{clear, groups},
        },
    },
    consume_material_theme, http, prompt_image_upload, use_initial,
};

#[derive(PartialEq)]
pub struct ProfileSettings {}

impl Component for ProfileSettings {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let user_id = radio
            .slice_current(|state| state.user_id.as_ref().unwrap())
            .read()
            .cloned();
        let user = radio.slice(AppChannel::Users, {
            let user_id = user_id.clone();
            move |state| state.users.get(&user_id).unwrap()
        });

        let theme = consume_material_theme();

        let mut error = use_state(|| None);

        let edit_user = {
            let user_id = user_id.clone();

            move |payload| {
                let user_id = user_id.clone();
                async move {
                    match http().edit_user(&user_id, &payload).await {
                        Ok(user) => Some(user),
                        Err(e) => {
                            error.set(Some(e));
                            None
                        }
                    }
                }
            }
        };

        let remove_field = {
            let edit_user = edit_user.clone();

            move |field| {
                let edit_user = edit_user.clone();

                async move {
                    edit_user(v0::DataEditUser {
                        display_name: None,
                        pronouns: None,
                        avatar: None,
                        status: None,
                        profile: None,
                        badges: None,
                        flags: None,
                        remove: vec![field],
                    })
                    .await
                }
            }
        };

        let current_user = user.read();

        let mut display_name =
            use_initial(|| current_user.display_name.clone().unwrap_or_default());
        let mut pronouns = use_initial(|| current_user.pronouns.clone().unwrap_or_default());
        let mut bio = use_initial(String::new);

        let mut profile = use_state(|| v0::UserProfile {
            content: None,
            background: None,
        });

        use_hook({
            let user_id = user_id.clone();

            move || {
                spawn(async move {
                    if let Ok(user_profile) = http().fetch_user_profile(&user_id).await {
                        bio.set_new(user_profile.content.clone().unwrap_or_default());
                        profile.set(user_profile);
                    };
                })
            }
        });

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
                                                current_user
                                                    .display_name
                                                    .as_ref()
                                                    .unwrap_or(&current_user.username)
                                                    .clone(),
                                            ),
                                    )
                                    .child(label().font_size(14.).line_height(1.5).text(format!(
                                        "{}#{}",
                                        current_user.username, current_user.discriminator
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
            .child(
                rect()
                    .spacing(8.)
                    .child(label().font_size(22.).text("Edit Global Profile"))
                    .child(label().text("Avatar").font_size(12.))
                    .child(
                        rect()
                            .horizontal()
                            .spacing(15.)
                            .cross_align(Alignment::Center)
                            .child(
                                StoatButton::new()
                                    .corner_radius(48.)
                                    .on_press({
                                        let edit_user = edit_user.clone();

                                        move |_| {
                                            let edit_user = edit_user.clone();

                                            spawn(async move {
                                                if let Some(id) =
                                                    prompt_image_upload(Tag::Icons).await
                                                {
                                                    edit_user(v0::DataEditUser {
                                                        display_name: None,
                                                        pronouns: None,
                                                        avatar: Some(id),
                                                        status: None,
                                                        profile: None,
                                                        badges: None,
                                                        flags: None,
                                                        remove: Vec::new(),
                                                    })
                                                    .await;
                                                };
                                            });
                                        }
                                    })
                                    .child(
                                        rect()
                                            .width(Size::px(96.))
                                            .height(Size::px(96.))
                                            .background(theme.md.surface_dim.as_argb_u32())
                                            .maybe_child(current_user.avatar.as_ref().map(
                                                |icon| {
                                                    rect()
                                                        .layer(Layer::Relative(1))
                                                        .width(Size::Fill)
                                                        .height(Size::Fill)
                                                        .child(file_image(icon))
                                                },
                                            )),
                                    ),
                            )
                            .child(
                                StoatButton::new()
                                    .corner_radius(16.)
                                    .on_press({
                                        let remove_field = remove_field.clone();

                                        move |_| {
                                            let remove_field = remove_field.clone();

                                            spawn(async move {
                                                remove_field(v0::FieldsUser::Avatar).await;
                                            });
                                        }
                                    })
                                    .child(
                                        rect()
                                            .width(Size::px(36.))
                                            .height(Size::px(36.))
                                            .center()
                                            .child(
                                                MaterialIcon::new(clear())
                                                    .size(Size::px(24.))
                                                    .color(theme.md.primary.as_argb_u32()),
                                            ),
                                    ),
                            ),
                    )
                    .child(label().text("Banner").font_size(12.))
                    .child(
                        rect()
                            .horizontal()
                            .spacing(15.)
                            .cross_align(Alignment::Center)
                            .child(
                                StoatButton::new()
                                    .corner_radius(16.)
                                    .on_press({
                                        let prompt_image_upload = prompt_image_upload.clone();
                                        let edit_user = edit_user.clone();

                                        move |_| {
                                            let prompt_image_upload = prompt_image_upload.clone();
                                            let edit_user = edit_user.clone();

                                            spawn(async move {
                                                if let Some(id) =
                                                    prompt_image_upload(Tag::Banners).await
                                                {
                                                    let profile = profile.read().cloned();

                                                    edit_user(v0::DataEditUser {
                                                        display_name: None,
                                                        pronouns: None,
                                                        avatar: None,
                                                        status: None,
                                                        profile: Some(v0::DataUserProfile {
                                                            content: profile.content,
                                                            background: Some(id),
                                                        }),
                                                        badges: None,
                                                        flags: None,
                                                        remove: Vec::new(),
                                                    })
                                                    .await;
                                                };
                                            });
                                        }
                                    })
                                    .child(
                                        rect()
                                            .width(Size::px((96. / 100.) * 232.))
                                            .height(Size::px(96.))
                                            .background(theme.md.surface_dim.as_argb_u32())
                                            .maybe_child(profile.read().background.as_ref().map(
                                                |banner| {
                                                    rect()
                                                        .layer(Layer::Relative(1))
                                                        .width(Size::Fill)
                                                        .height(Size::Fill)
                                                        .child(
                                                            file_image(banner)
                                                                .aspect_ratio(AspectRatio::Max),
                                                        )
                                                },
                                            )),
                                    ),
                            )
                            .child(
                                StoatButton::new()
                                    .corner_radius(16.)
                                    .on_press({
                                        let remove_field = remove_field.clone();

                                        move |_| {
                                            let remove_field = remove_field.clone();

                                            spawn(async move {
                                                remove_field(v0::FieldsUser::ProfileBackground)
                                                    .await;
                                            });
                                        }
                                    })
                                    .child(
                                        rect()
                                            .width(Size::px(36.))
                                            .height(Size::px(36.))
                                            .center()
                                            .child(
                                                MaterialIcon::new(clear())
                                                    .size(Size::px(24.))
                                                    .color(theme.md.primary.as_argb_u32()),
                                            ),
                                    ),
                            ),
                    )
                    .child(SingleLineEntry::new("Display Name", display_name))
                    .child(SingleLineEntry::new("Pronouns", pronouns))
                    .child(SingleLineEntry::new("Profile Bio", bio))
                    .child(
                        rect()
                            .horizontal()
                            .spacing(8.)
                            .font_size(14)
                            .child(
                                StoatButton::new()
                                    .color(theme.md.primary.as_argb_u32())
                                    .corner_radius(40.)
                                    .child(
                                        rect()
                                            .height(Size::px(40.))
                                            .padding((0., 16.))
                                            .center()
                                            .child("Reset"),
                                    )
                                    .on_press(move |_| {
                                        display_name.reset();
                                        pronouns.reset();
                                        bio.reset();
                                    }),
                            )
                            .child(
                                StoatButton::new()
                                    .color(theme.md.on_primary.as_argb_u32())
                                    .background(theme.md.primary.as_argb_u32())
                                    .corner_radius(40.)
                                    .on_press({
                                        move |_| {
                                            let edit_user = edit_user.clone();

                                            spawn({
                                                async move {
                                                    let mut payload = v0::DataEditUser {
                                                        display_name: None,
                                                        pronouns: None,
                                                        avatar: None,
                                                        status: None,
                                                        profile: None,
                                                        badges: None,
                                                        flags: None,
                                                        remove: Vec::new(),
                                                    };

                                                    if let Some(display_name) =
                                                        display_name.get_if_different()
                                                    {
                                                        if display_name.is_empty() {
                                                            payload
                                                                .remove
                                                                .push(v0::FieldsUser::DisplayName);
                                                        } else {
                                                            payload.display_name =
                                                                Some(display_name);
                                                        };
                                                    };

                                                    if let Some(pronouns) =
                                                        pronouns.get_if_different()
                                                    {
                                                        if pronouns.is_empty() {
                                                            payload
                                                                .remove
                                                                .push(v0::FieldsUser::Pronouns);
                                                        } else {
                                                            payload.pronouns = Some(pronouns);
                                                        };
                                                    };

                                                    if let Some(user) = edit_user(payload).await {
                                                        display_name.set_new(
                                                            user.display_name.unwrap_or_default(),
                                                        );
                                                        pronouns.set_new(
                                                            user.pronouns.unwrap_or_default(),
                                                        );
                                                        bio.apply();
                                                    };
                                                }
                                            });
                                        }
                                    })
                                    .child(
                                        rect()
                                            .height(Size::px(40.))
                                            .padding((0., 16.))
                                            .center()
                                            .child("Save"),
                                    ),
                            ),
                    ),
            )
    }
}
