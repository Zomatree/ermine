use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{FriendButton, FriendPage},
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct FriendsList {
    pub page: State<FriendPage>,
}

#[derive(PartialEq, Clone)]
enum ListItem {
    Header(&'static str, usize),
    User(Readable<v0::User>),
    Placeholder,
}

impl Component for FriendsList {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let radio = use_radio(AppChannel::UserId);
        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());

        let user = radio.slice(AppChannel::Users, move |state| {
            state.users.get(user_id.read().as_str()).unwrap()
        });

        let relations = use_memo({
            let page = self.page.clone();
            let radio = radio.clone();

            move || {
                let page = page();

                user.read()
                    .relations
                    .clone()
                    .into_iter()
                    .map::<Readable<v0::User>, _>(|relationship| {
                        radio
                            .slice(AppChannel::Users, {
                                let user_id = relationship.user_id.clone();
                                move |state| state.users.get(&user_id).unwrap()
                            })
                            .into_readable()
                    })
                    .filter(|user| {
                        let user = user.read();

                        match page {
                            FriendPage::Online => {
                                user.online && user.relationship == v0::RelationshipStatus::Friend
                            }
                            FriendPage::All => user.relationship == v0::RelationshipStatus::Friend,
                            FriendPage::Pending => [
                                v0::RelationshipStatus::Incoming,
                                v0::RelationshipStatus::Outgoing,
                            ]
                            .contains(&user.relationship),
                            FriendPage::Blocked => [
                                v0::RelationshipStatus::Blocked,
                                v0::RelationshipStatus::BlockedOther,
                            ]
                            .contains(&user.relationship),
                        }
                    })
                    .collect::<Vec<_>>()
            }
        });

        let groups = use_memo({
            let page = self.page.clone();

            move || {
                let page = page();
                let relations = relations.read();

                let mut groups = Vec::new();

                match page {
                    FriendPage::Online => {
                        groups.push(ListItem::Header("Online", relations.len()));

                        if relations.is_empty() {
                            groups.push(ListItem::Placeholder);
                        }

                        groups.extend(relations.iter().cloned().map(ListItem::User));
                    }
                    FriendPage::All => {
                        groups.push(ListItem::Header("All", relations.len()));

                        if relations.is_empty() {
                            groups.push(ListItem::Placeholder);
                        }

                        groups.extend(relations.iter().cloned().map(ListItem::User));
                    }
                    FriendPage::Pending => {
                        let mut incoming = Vec::new();
                        let mut outgoing = Vec::new();

                        for user in relations.iter().cloned() {
                            if user.read().relationship == v0::RelationshipStatus::Incoming {
                                incoming.push(ListItem::User(user));
                            } else {
                                outgoing.push(ListItem::User(user));
                            }
                        }

                        groups.push(ListItem::Header("Incoming", incoming.len()));

                        if incoming.is_empty() {
                            groups.push(ListItem::Placeholder);
                        }

                        groups.extend(incoming);

                        groups.push(ListItem::Header("Outgoing", outgoing.len()));

                        if outgoing.is_empty() {
                            groups.push(ListItem::Placeholder);
                        }

                        groups.extend(outgoing);
                    }
                    FriendPage::Blocked => {
                        groups.push(ListItem::Header("Blocked", relations.len()));

                        if relations.is_empty() {
                            groups.push(ListItem::Placeholder);
                        }

                        groups.extend(relations.iter().cloned().map(ListItem::User));
                    }
                };

                groups
            }
        });

        VirtualScrollView::new(move |vitem, _| {
            let item = groups.read()[vitem.index].clone();

            match item {
                ListItem::Header(title, count) => {
                    rect()
                        .key(&(title, count))
                        .padding((0., 32.))
                        .height(Size::px(54.))
                        .main_align(Alignment::Center)
                        .font_size(11.)
                        .color(theme.md.on_surface_variant.as_argb_u32())
                        .child(format!("{title} – {count}"))
                        .into_element()
                }

                ListItem::User(user) => rect()
                    .key(&user.peek().id)
                    .padding((0., 16., 2., 16.))
                    .child(FriendButton { user })
                    .into_element(),

                ListItem::Placeholder => rect()
                    .key(vitem.index)
                    .padding((0., 32.))
                    .height(Size::px(54.))
                    .main_align(Alignment::Center)
                    .font_size(16.)
                    .opacity(0.38)
                    .child("Nobody here right now!")
                    .into_element(),
            }
        })
        .item_size(54.)
        .length(groups.read().len())
    }
}
