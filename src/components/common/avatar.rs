use freya::prelude::*;
use stoat_models::v0;

use crate::{components::file_image, http, consume_material_theme};

#[derive(PartialEq)]
pub struct Avatar {
    user: Readable<v0::User>,
    member: Option<Readable<v0::Member>>,
    size: f32,
    presence: bool,
}

impl Avatar {
    pub fn new(user: Readable<v0::User>, member: Option<Readable<v0::Member>>, size: f32) -> Self {
        Self {
            user,
            member,
            size,
            presence: false,
        }
    }

    pub fn presence(mut self, presence: bool) -> Self {
        self.presence = presence;
        self
    }
}

impl Component for Avatar {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let image = self
            .member
            .as_ref()
            .and_then(|member| member.read().avatar.as_ref().map(file_image))
            .or_else(|| self.user.read().avatar.as_ref().map(file_image));

        rect()
            .width(Size::px(self.size))
            .height(Size::px(self.size))
            .child(if let Some(image) = image {
                image
                    .aspect_ratio(AspectRatio::Max)
                    .image_cover(ImageCover::Center)
                    .expanded()
                    .corner_radius(self.size)
                    .overflow(Overflow::Clip)
            } else {
                ImageViewer::new(
                    http()
                        .format_default_avatar_url(&self.user.peek().id)
                        .parse::<Url>()
                        .unwrap(),
                )
                .sampling_mode(SamplingMode::Trilinear)
                .expanded()
                .corner_radius(self.size)
                .overflow(Overflow::Clip)
            })
            .maybe_child(self.presence.then(|| {
                let diameter = (12. / 32.) * self.size;
                let pos = (20. / 32.) * self.size;

                let user = self.user.read();

                let presence = if !user.online {
                    v0::Presence::Invisible
                } else {
                    user.status
                        .as_ref()
                        .and_then(|status| status.presence.clone())
                        .unwrap_or(v0::Presence::Online)
                };

                let color = match presence {
                    v0::Presence::Online => theme.stoat.presence_online,
                    v0::Presence::Idle => theme.stoat.presence_idle,
                    v0::Presence::Focus => theme.stoat.presence_focus,
                    v0::Presence::Busy => theme.stoat.presence_busy,
                    v0::Presence::Invisible => theme.stoat.presence_invisible,
                };

                rect()
                    .position(Position::new_absolute().left(pos).top(pos))
                    .layer(Layer::Relative(1))
                    .width(Size::px(diameter))
                    .height(Size::px(diameter))
                    .corner_radius(diameter)
                    .background(color)
                    .border(Border::new().width(2.).fill(Color::BLACK))
            }))
    }

    fn render_key(&self) -> DiffKey {
        (&self.user.peek().id).into()
    }
}
