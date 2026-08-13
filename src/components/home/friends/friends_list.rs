use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{FriendButton, FriendPage},
};

#[derive(PartialEq)]
pub struct FriendsList {
    pub page: State<FriendPage>,
}

impl Component for FriendsList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());

        let user = radio.slice(AppChannel::Users, move |state| {
            state.users.get(user_id.read().as_str()).unwrap()
        });

        let relations = use_memo({
            let page = self.page.clone();
            let radio = radio.clone();

            move || {
                let page = *page.read();

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

        // rect().padding((0., 16.)).child(
        VirtualScrollView::new(move |item, _| {
            let user = relations.read()[item.index].clone();

            rect()
                .key(user.peek().id.clone())
                .padding((0., 16., 2., 16.))
                // .margin((0., 16.))
                .child(FriendButton { user })
                .into_element()
        })
        .item_size(54.)
        .length(relations.read().len())
        // )
    }
}
