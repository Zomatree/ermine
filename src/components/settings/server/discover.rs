use freya::prelude::*;
use stoat_models::v0;

use crate::{
    components::{
        CategoryButton, CategoryButtonGroup,
        material::outlined::{cancel, public},
        stoat_label,
    },
    http,
};

#[derive(PartialEq)]
pub struct DiscoverServerSettings {
    pub server: Readable<v0::Server>,
}

impl Component for DiscoverServerSettings {
    fn render(&self) -> impl IntoElement {
        let mut status = use_state(|| None);

        let server_id = self.server.read().id.clone();

        let mut fetch_status = use_future({
            let server_id = server_id.clone();

            move || {
                let server_id = server_id.clone();

                async move {
                    if let Ok(response) = http().fetch_discover_status(&server_id).await {
                        status.set(Some(response));
                    } else {
                        status.set(None);
                    }
                }
            }
        });

        let current_status = status.read();

        if self.server.read().discoverable {
            rect()
                .child(
                    stoat_label()
                        .font_weight(600)
                        .text("This server is discoverable"),
                )
                .child(stoat_label().text("Contact support to remove this server from discover"))
        } else {
            rect()
                .maybe_child(current_status.as_ref().map(|status| {
                    match &status.status {
                        v0::DiscoverRequestStatus::Pending
                        | v0::DiscoverRequestStatus::UnderReview => {
                            rect()
                                .padding(8.)
                                .spacing(12.)
                                .child(stoat_label().font_weight(600).text("Your server is under review"))
                                .child(stoat_label().text(
                                    "Check back often to see if your server has been approved!",
                                ))
                                .child(
                                    CategoryButtonGroup::new().child(
                                        CategoryButton::new(cancel())
                                            .chevron()
                                            .on_press({
                                                let server_id = server_id.clone();

                                                move |_| {
                                                    let server_id = server_id.clone();

                                                    spawn(async move {
                                                        if http()
                                                            .remove_from_discover(&server_id)
                                                            .await
                                                            .is_ok()
                                                        {
                                                            fetch_status.start();
                                                        };
                                                    });
                                                }
                                            })
                                            .title(stoat_label().text("Cancel discover request"))
                                            .description(stoat_label().text(
                                                "Cancel your discover request and remove it from the review queue",
                                            )),
                                    ),
                                )
                        }
                        v0::DiscoverRequestStatus::Removed(reason) => rect()
                            .padding(8.)
                            .spacing(12.)
                            .child(stoat_label().font_weight(600).text("Your discover request has been removed"))
                            .maybe_child(reason.as_ref().map(|reason| stoat_label().text(format!("Reason: {reason}"))))
                            .child(stoat_label().text("Contact support for more information.")),
                        v0::DiscoverRequestStatus::Denied(reason) => rect()
                            .padding(8.)
                            .spacing(12.)
                            .child(stoat_label().font_weight(600).text("Your discover request has been denied"))
                            .maybe_child(reason.as_ref().map(|reason| stoat_label().text(format!("Reason: {reason}"))))
                            .child(stoat_label().text("You can request again below."))
                            .child(CategoryButtonGroup::new().child(
                                CategoryButton::new(public())
                                    .chevron()
                                    .on_press({
                                        let server_id = server_id.clone();

                                        move |_| {
                                            let server_id = server_id.clone();

                                            spawn(async move {
                                                if http().add_to_discover(&server_id).await.is_ok() {
                                                    fetch_status.start();
                                                };
                                            });
                                        }
                                    })
                                    .title(stoat_label().text("Submit to Discover"))
                                    .description(
                                        stoat_label()
                                            .text("Allow others to join your server from Discover"),
                                    )
                        )),
                        v0::DiscoverRequestStatus::Approved(_) => rect(),
                    }
                }))
                .child(
                    CategoryButtonGroup::new().maybe_child(current_status.is_none().then(|| {
                        CategoryButton::new(public())
                            .chevron()
                            .on_press({
                                let server_id = server_id.clone();

                                move |_| {
                                    let server_id = server_id.clone();

                                    spawn(async move {
                                        if http().add_to_discover(&server_id).await.is_ok() {
                                            fetch_status.start();
                                        };
                                    });
                                }
                            })
                            .title(stoat_label().text("Submit to Discover"))
                            .description(
                                stoat_label()
                                    .text("Allow others to join your server from Discover"),
                            )
                    })),
                )
        }
    }
}
