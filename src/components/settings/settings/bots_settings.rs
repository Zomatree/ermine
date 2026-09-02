use std::collections::HashMap;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, SettingsPage, SizeExt, Tag,
    components::{
        Avatar, MaterialIcon, ModalValue, SingleLineEntry, StoatButton,
        StoatButtonColorsThemePartialExt, StoatButtonLayoutThemePartialExt, file_image,
        material::outlined::{
            chevron_right, clear, content_copy, delete, key, launch, library_books, link,
            lock_reset, person_add, smart_toy,
        },
        use_modals,
    },
    consume_material_theme, http, prompt_image_upload, use_initial,
};

#[derive(PartialEq)]
pub struct BotsSettings {
    pub selected_bot: Option<(String, String)>,
}

impl Component for BotsSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let radio = use_radio(AppChannel::SettingsPage);
        let mut modals = use_modals();

        let set_selected_bot = {
            move |bot| radio.clone().write().settings_page = Some(SettingsPage::MyBots(Some(bot)))
        };

        let mut bots = use_state(HashMap::new);

        use_hook(|| {
            spawn(async move {
                if let Ok(resp) = http().get_me_bots().await {
                    let mut bots = bots.write();

                    for bot in resp.bots {
                        let user = resp.users.iter().find(|u| u.id == bot.id).unwrap().clone();

                        bots.insert(bot.id.clone(), (bot, user));
                    }
                }
            })
        });

        if let Some((id, _name)) = self.selected_bot.clone() {
            let bot = bots.into_writable().map(
                {
                    let id = id.clone();
                    move |bots| bots.get(&id).unwrap()
                },
                {
                    let id = id.clone();
                    move |bots| bots.get_mut(&id).unwrap()
                },
            );

            SelectedBotsSettings { id, bot, bots }.into_element()
        } else {
            rect()
                .spacing(15.)
                .child(
                    rect()
                        .corner_radius(28.)
                        .overflow(Overflow::Clip)
                        .spacing(2.)
                        .child(
                            StoatButton::new()
                                .corner_radius(12.)
                                .on_press(move |_| {
                                    modals.write().push_modal(ModalValue::CreateBot {
                                        callback: EventHandler::new({
                                            let set_selected_bot = set_selected_bot.clone();

                                            move |response: v0::BotWithUserResponse| {
                                                set_selected_bot((response.bot.id.clone(), response.user.username.clone()));
                                                bots.write().insert(response.bot.id.clone(), (response.bot, response.user));
                                            }
                                        })
                                    });
                                })
                                .child(
                                    rect()
                                        .padding(13.)
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
                                                        .child(
                                                            MaterialIcon::new(smart_toy()).size(Size::px(22.)),
                                                        ),
                                                )
                                                .child(
                                                    rect()
                                                        .width(Size::flex(1.))
                                                        .child(
                                                            label()
                                                                .font_size(14.)
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .line_height(1.5)
                                                                .text("Create Bot"),
                                                        )
                                                        .child(
                                                            label()
                                                                .font_size(12.)
                                                                .line_height(1.5)
                                                                .text("You agree that your bot is subject to the Acceptable Usage Policy."),
                                                        ),
                                                )
                                                .child(
                                                    MaterialIcon::new(chevron_right()).size(Size::px(18.)),
                                                ),
                                        ),
                                ),
                        )
                        .child(
                            StoatButton::new()
                                .corner_radius(12.)
                                .on_press(move |_| {
                                    open::that_detached("https://developers.stoat.chat/").unwrap();
                                })
                                .child(
                                    rect()
                                        .corner_radius(12.)
                                        .padding(13.)
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
                                                        .child(
                                                            MaterialIcon::new(library_books())
                                                                .size(Size::px(22.)),
                                                        ),
                                                )
                                                .child(
                                                    rect()
                                                        .width(Size::flex(1.))
                                                        .child(
                                                            label()
                                                                .font_size(14.)
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .line_height(1.5)
                                                                .text("Developer Documentation"),
                                                        )
                                                        .child(
                                                            label()
                                                                .font_size(12.)
                                                                .line_height(1.5)
                                                                .text("Learn more about how to create bots on Stoat."),
                                                        ),
                                                )
                                                .child(
                                                    MaterialIcon::new(launch()).size(Size::px(18.)),
                                                ),
                                        )
                                )
                        )
                )
                .child(
                    rect()
                        .corner_radius(28.)
                        .overflow(Overflow::Clip)
                        .spacing(2.)
                        .children(bots.read().values().map(|(bot, user)| {
                            StoatButton::new()
                                .corner_radius(12.)
                                .on_press({
                                    let id = bot.id.clone();
                                    let name = user.username.clone();

                                    let set_selected_bot = set_selected_bot.clone();
                                    move |_| set_selected_bot((id.clone(), name.clone()))
                                })
                                .child(
                                    rect()
                                        .padding(13.)
                                        .background(theme.md.secondary_container.as_argb_u32())
                                        .color(theme.md.on_secondary_container.as_argb_u32())
                                        .horizontal()
                                        .spacing(16.)
                                        .cross_align(Alignment::Center)
                                        .content(Content::Flex)
                                        .child(rect()
                                            .corner_radius(36.)
                                            .width(Size::px(36.))
                                            .height(Size::px(36.))
                                            .background(theme.md.surface_dim.as_argb_u32())
                                            .color(theme.md.on_surface.as_argb_u32())
                                            .center()
                                            .child(Avatar::new(user.clone().into_readable(), None, 24.))
                                        )
                                        .child(label().width(Size::flex(1.)).text(user.username.clone()))
                                        .child(
                                            MaterialIcon::new(chevron_right())
                                                .width(Size::px(18.))
                                                .height(Size::px(18.)),
                                        ),
                                )
                        }))
                )
                .into_element()
        }
    }
}

#[derive(PartialEq)]
pub struct SelectedBotsSettings {
    pub id: String,
    pub bot: Writable<(v0::Bot, v0::User)>,
    pub bots: State<HashMap<String, (v0::Bot, v0::User)>>,
}

impl Component for SelectedBotsSettings {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::SettingsPage);
        let settings = radio.slice_mut_current(|state| &mut state.settings_page);
        let mut modals = use_modals();

        let bot = self.bot.map(|(bot, _)| bot, |(bot, _)| bot);
        let user = self.bot.map(|(_, user)| user, |(_, user)| user);

        let mut error = use_state(|| None);

        let edit_user = {
            let user_id = self.id.clone();

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

        let remove_user_field = {
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

        let edit_bot = {
            let user_id = self.id.clone();

            move |payload| {
                let user_id = user_id.clone();

                async move {
                    match http().edit_bot(&user_id, &payload).await {
                        Ok(user) => Some(user),
                        Err(e) => {
                            error.set(Some(e));
                            None
                        }
                    }
                }
            }
        };

        let remove_bot_field = {
            let edit_bot = edit_bot.clone();

            move |field| {
                let edit_bot = edit_bot.clone();

                async move {
                    edit_bot(v0::DataEditBot {
                        name: None,
                        public: None,
                        analytics: None,
                        interactions_url: None,
                        remove: vec![field],
                    })
                    .await
                }
            }
        };

        let current_user = user.read();

        let mut username = use_initial(|| current_user.username.clone());
        let mut display_name =
            use_initial(|| current_user.display_name.clone().unwrap_or_default());

        rect()
            .spacing(15.)
            .child(
                rect()
                    .spacing(8.)
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
                                                if let Some(id) = prompt_image_upload(Tag::Icons).await
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
                                            .maybe_child(current_user.avatar.as_ref().map(|icon| {
                                                rect()
                                                    .layer(Layer::Relative(1))
                                                    .width(Size::Fill)
                                                    .height(Size::Fill)
                                                    .child(file_image(icon))
                                            })),
                                    ),
                            )
                            .child(
                                StoatButton::new()
                                    .corner_radius(16.)
                                    .on_press({
                                        let remove_user_field = remove_user_field.clone();

                                        move |_| {
                                            let remove_user_field = remove_user_field.clone();

                                            spawn(async move {
                                                remove_user_field(v0::FieldsUser::Avatar).await;
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
                    // .child(label().text("Banner").font_size(12.))
                    // .child(
                    //     rect()
                    //         .horizontal()
                    //         .spacing(15.)
                    //         .cross_align(Alignment::Center)
                    //         .child(
                    //             StoatButton::new()
                    //                 .corner_radius(16.)
                    //                 .on_press({
                    //                     let prompt_image_upload = prompt_image_upload.clone();
                    //                     let edit_user = edit_user.clone();
                    //                     move |_| {
                    //                         let prompt_image_upload = prompt_image_upload.clone();
                    //                         let edit_user = edit_user.clone();
                    //                         spawn(async move {
                    //                             if let Some(id) =
                    //                                 prompt_image_upload(Tag::Banners).await
                    //                             {
                    //                                 let profile = profile.read().cloned();
                    //                                 edit_user(v0::DataEditUser {
                    //                                     display_name: None,
                    //                                     pronouns: None,
                    //                                     avatar: None,
                    //                                     status: None,
                    //                                     profile: Some(v0::DataUserProfile {
                    //                                         content: profile.content,
                    //                                         background: Some(id),
                    //                                     }),
                    //                                     badges: None,
                    //                                     flags: None,
                    //                                     remove: Vec::new(),
                    //                                 })
                    //                                 .await;
                    //                             };
                    //                         });
                    //                     }
                    //                 })
                    //                 .child(
                    //                     rect()
                    //                         .width(Size::px((96. / 100.) * 232.))
                    //                         .height(Size::px(96.))
                    //                         .background(theme.md.surface_dim.as_argb_u32())
                    //                         .maybe_child(profile.read().background.as_ref().map(
                    //                             |banner| {
                    //                                 rect()
                    //                                     .layer(Layer::Relative(1))
                    //                                     .width(Size::Fill)
                    //                                     .height(Size::Fill)
                    //                                     .child(
                    //                                         file_image(banner)
                    //                                             .aspect_ratio(AspectRatio::Max),
                    //                                     )
                    //                             },
                    //                         )),
                    //                 ),
                    //         )
                    //         .child(
                    //             StoatButton::new()
                    //                 .corner_radius(16.)
                    //                 .on_press({
                    //                     let remove_field = remove_field.clone();
                    //                     move |_| {
                    //                         let remove_field = remove_field.clone();
                    //                         spawn(async move {
                    //                             remove_field(v0::FieldsUser::ProfileBackground).await;
                    //                         });
                    //                     }
                    //                 })
                    //                 .child(
                    //                     rect()
                    //                         .width(Size::px(36.))
                    //                         .height(Size::px(36.))
                    //                         .center()
                    //                         .child(
                    //                             MaterialIcon::new(clear())
                    //                                 .size(Size::px(24.))
                    //                                 .color(theme.md.primary.as_argb_u32()),
                    //                         ),
                    //                 ),
                    //         ),
                    // )
                    .child(SingleLineEntry::new("Username", username))
                    .child(SingleLineEntry::new("Display Name", display_name))
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
                                        username.reset();
                                        display_name.reset();
                                    }),
                            )
                            .child(
                                StoatButton::new()
                                    .color(theme.md.on_primary.as_argb_u32())
                                    .background(theme.md.primary.as_argb_u32())
                                    .corner_radius(40.)
                                    .on_press({
                                        let edit_user = edit_user.clone();
                                        let edit_bot = edit_bot.clone();
                                        let bot = bot.clone();
                                        let settings = settings.clone();

                                        move |_| {
                                            let edit_user = edit_user.clone();
                                            let edit_bot = edit_bot.clone();
                                            let mut settings = settings.clone();
                                            let mut bot = bot.clone();
                                            let mut user = user.clone();

                                            spawn({
                                                async move {
                                                    if let Some(new_display_name) =
                                                        display_name.get_if_different()
                                                    {
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

                                                        if new_display_name.is_empty() {
                                                            payload
                                                                .remove
                                                                .push(v0::FieldsUser::DisplayName);
                                                        } else {
                                                            payload.display_name =
                                                                Some(new_display_name);
                                                        };

                                                        if let Some(new_user) = edit_user(payload).await {
                                                            display_name.set_new(
                                                                new_user.display_name.clone().unwrap_or_default(),
                                                            );
                                                            user.set(new_user);
                                                        };
                                                    };

                                                    if let Some(new_username) =
                                                        username.get_if_different()
                                                    {
                                                        let payload = v0::DataEditBot {
                                                            name: Some(new_username),
                                                            public: None,
                                                            analytics: None,
                                                            interactions_url: None,
                                                            remove: Vec::new(),
                                                        };

                                                        if let Some(response) = edit_bot(payload).await
                                                        {
                                                            username.set_new(response.user.username.clone());
                                                            *settings.write() = Some(SettingsPage::MyBots(Some((response.bot.id.clone(), response.user.username.clone()))));
                                                            bot.set(response.bot);
                                                            user.set(response.user)
                                                        }
                                                    }
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
            .child(rect()
                .corner_radius(28.)
                .overflow(Overflow::Clip)
                .spacing(2.)
                .child(
                    StoatButton::new()
                        .corner_radius(12.)
                        .on_press({
                            let id = self.id.clone();
                            let name = current_user.username.clone();
                            let bot = bot.clone();

                            move |_| {
                                modals.write().push_modal(ModalValue::ResetBotToken {
                                    id: id.clone(),
                                    name: name.clone(),
                                    callback: EventHandler::new({
                                        let mut bot = bot.clone();

                                        move |response: v0::BotWithUserResponse| {
                                            bot.write().token = response.bot.token;
                                        }
                                    })
                                });
                            }
                        })
                        .child(
                            rect()
                                .padding(13.)
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
                                                .child(
                                                    MaterialIcon::new(lock_reset()).size(Size::px(22.)),
                                                ),
                                        )
                                        .child(
                                            rect()
                                                .width(Size::flex(1.))
                                                .child(
                                                    label()
                                                        .font_size(14.)
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .line_height(1.5)
                                                        .text("Reset Token"),
                                                )
                                                .child(
                                                    label()
                                                        .font_size(12.)
                                                        .line_height(1.5)
                                                        .text("Generate a new token if it gets lost or compromised"),
                                                ),
                                        )
                                        .child(
                                            MaterialIcon::new(chevron_right()).size(Size::px(18.)),
                                        ),
                                ),
                        ),
                )
                .child(
                    StoatButton::new()
                        .corner_radius(12.)
                        .on_press({
                            let id = self.id.clone();

                            move |_| {
                                let id = id.clone();

                                spawn(async move {
                                    if let Ok(bot) = http().fetch_public_bot(&id).await {
                                        modals.write().push_modal(ModalValue::InviteBot { bot });
                                    }
                                });
                            }
                        })
                        .child(
                            rect()
                                .padding(13.)
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
                                                .child(
                                                    MaterialIcon::new(person_add()).size(Size::px(22.)),
                                                ),
                                        )
                                        .child(
                                            label()
                                                .width(Size::flex(1.))
                                                .font_size(14.)
                                                .font_weight(FontWeight::MEDIUM)
                                                .line_height(1.5)
                                                .text("Invite Bot")
                                        )
                                        .child(
                                            MaterialIcon::new(chevron_right()).size(Size::px(18.)),
                                        ),
                                ),
                        ),
                )
                .child(
                    StoatButton::new()
                        .corner_radius(12.)
                        .on_press({
                            let id = self.id.clone();

                            move |_| {
                                Clipboard::set(format!("https://stoat.chat/bot/{id}")).unwrap();
                            }
                        })
                        .child(
                            rect()
                                .padding(13.)
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
                                                .child(
                                                    MaterialIcon::new(link()).size(Size::px(22.)),
                                                ),
                                        )
                                        .child(
                                            label()
                                                .width(Size::flex(1.))
                                                .font_size(14.)
                                                .font_weight(FontWeight::MEDIUM)
                                                .line_height(1.5)
                                                .text("Copy Invite URL")
                                        )
                                        .child(
                                            MaterialIcon::new(content_copy()).size(Size::px(18.)),
                                        ),
                                ),
                        ),
                )
                .child(
                    StoatButton::new()
                        .corner_radius(12.)
                        .on_press({
                            let id = self.id.clone();

                            move |_| {
                                Clipboard::set(id.clone()).unwrap();
                            }
                        })
                        .child(
                            rect()
                                .padding(13.)
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
                                                .child(
                                                    MaterialIcon::new(content_copy()).size(Size::px(22.)),
                                                ),
                                        )
                                        .child(
                                            label()
                                                .width(Size::flex(1.))
                                                .font_size(14.)
                                                .font_weight(FontWeight::MEDIUM)
                                                .line_height(1.5)
                                                .text("Copy ID")
                                        )
                                        .child(
                                            MaterialIcon::new(content_copy()).size(Size::px(18.)),
                                        ),
                                ),
                        ),
                )
                .child(
                    StoatButton::new()
                        .corner_radius(12.)
                        .on_press({
                            let bot = bot.clone();

                            move |_| {
                                Clipboard::set(bot.read().token.clone()).unwrap();
                            }
                        })
                        .child(
                            rect()
                                .padding(13.)
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
                                                .child(
                                                    MaterialIcon::new(key()).size(Size::px(22.)),
                                                ),
                                        )
                                        .child(
                                            label()
                                                .width(Size::flex(1.))
                                                .font_size(14.)
                                                .font_weight(FontWeight::MEDIUM)
                                                .line_height(1.5)
                                                .text("Copy Token")
                                        )
                                        .child(
                                            MaterialIcon::new(content_copy()).size(Size::px(18.)),
                                        ),
                                ),
                        ),
                )
                .child(
                    StoatButton::new()
                        .corner_radius(12.)
                        .on_press({
                            let id = self.id.clone();
                            let name = current_user.username.clone();
                            let settings = settings.clone();
                            let mut bots = self.bots;

                            move |_| {
                                modals.write().push_modal(ModalValue::DeleteBot {
                                    id: id.clone(),
                                    name: name.clone(),
                                    callback: EventHandler::new({
                                        let mut settings = settings.clone();
                                        let id = id.clone();

                                        move |_| {
                                            bots.write().remove(&id);
                                            *settings.write() = Some(SettingsPage::MyBots(None));
                                        }
                                    })
                                });
                            }
                        })
                        .child(
                            rect()
                                .padding(13.)
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
                                                .color(theme.md.error.as_argb_u32())
                                                .center()
                                                .child(
                                                    MaterialIcon::new(delete()).size(Size::px(22.)),
                                                ),
                                        )
                                        .child(
                                            label()
                                                .width(Size::flex(1.))
                                                .font_size(14.)
                                                .font_weight(FontWeight::MEDIUM)
                                                .line_height(1.5)
                                                .text("Delete Bot")
                                        )
                                        .child(
                                            MaterialIcon::new(chevron_right()).size(Size::px(18.)),
                                        ),
                                ),
                        ),
                )
            )
    }
}
