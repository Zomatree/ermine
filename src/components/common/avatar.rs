use freya::prelude::*;
use freya_engine::prelude::{ClipOp, Paint, Path, PathDirection, Point, Rect};
use stoat_models::v0;

use crate::{SizeExt, components::file_image, consume_material_theme, http};

#[derive(PartialEq)]
pub struct Avatar {
    user: Readable<v0::User>,
    member: Option<Readable<v0::Member>>,
    size: f32,
    presence: bool,
    selectable: bool,
}

impl Avatar {
    pub fn new(
        user: impl Into<Readable<v0::User>>,
        member: Option<Readable<v0::Member>>,
        size: f32,
    ) -> Self {
        Self {
            user: user.into(),
            member,
            size,
            presence: false,
            selectable: false,
        }
    }

    pub fn presence(mut self, presence: bool) -> Self {
        self.presence = presence;
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }
}

impl Component for Avatar {
    fn render(&self) -> impl IntoElement {
        let image = self
            .member
            .as_ref()
            .and_then(|member| member.read().avatar.as_ref().map(file_image))
            .or_else(|| self.user.read().avatar.as_ref().map(file_image));

        let mask = RenderCallback::new({
            let presence = self.presence;

            move |context| {
                let RenderTarget::Background { canvas, .. } = &context.target else {
                    return;
                };

                if presence {
                    let path = Path::circle(
                        Point::new(
                            context.origin.x + (context.size.width * (27. / 32.)),
                            context.origin.y + (context.size.width * (27. / 32.)),
                        ),
                        context.size.width * (7. / 32.),
                        PathDirection::CW,
                    );
                    canvas.clip_path(&path, ClipOp::Difference, true);
                };

                let mut white = Paint::default();
                white.set_anti_alias(true);
                white.set_color(Color::WHITE);
                canvas.draw_circle(
                    Point::new(
                        context.origin.x + (context.size.width / 2.),
                        context.origin.y + (context.size.height / 2.),
                    ),
                    context.size.width / 2.,
                    &white,
                );
            }
        });

        rect()
            .size(Size::px(self.size))
            .child(if let Some(image) = image {
                image
                    .aspect_ratio(AspectRatio::Max)
                    .image_cover(ImageCover::Center)
                    .size(Size::px(self.size))
                    .selectable(self.selectable)
                    .mask(mask)
                    .into_element()
            } else {
                ImageViewer::new(
                    http()
                        .format_default_avatar_url(&self.user.peek().id)
                        .parse::<Url>()
                        .unwrap(),
                )
                .sampling_mode(SamplingMode::Trilinear)
                .size(Size::px(self.size))
                .mask(mask)
                .into_element()
            })
            .maybe_child(self.presence.then(|| {
                let diameter = (10. / 32.) * self.size;
                let pos = (22. / 32.) * self.size;

                let user = self.user.read();

                let presence = if !user.online {
                    v0::Presence::Invisible
                } else {
                    user.status
                        .as_ref()
                        .and_then(|status| status.presence.clone())
                        .unwrap_or(v0::Presence::Online)
                };

                UserPresence::new(presence)
                    .position(Position::new_absolute().left(pos).top(pos))
                    .layer(Layer::Relative(2))
                    .size(Size::px(diameter))
            }))
    }

    fn render_key(&self) -> DiffKey {
        (&self.user.peek().id).into()
    }
}

#[derive(PartialEq)]
pub struct UserPresence {
    presence: v0::Presence,

    layout: LayoutData,
    layer: Layer,
}

impl UserPresence {
    pub fn new(presence: v0::Presence) -> Self {
        Self {
            presence,
            layout: LayoutData::default(),
            layer: Layer::default(),
        }
    }
}

impl LayoutExt for UserPresence {
    fn layout_mut(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl ContainerPositionExt for UserPresence {}

impl LayerExt for UserPresence {
    fn layer_mut(&mut self) -> &mut Layer {
        &mut self.layer
    }
}

impl ContainerSizeExt for UserPresence {}

impl Component for UserPresence {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let color = match &self.presence {
            v0::Presence::Online => theme.stoat.presence_online,
            v0::Presence::Idle => theme.stoat.presence_idle,
            v0::Presence::Focus => theme.stoat.presence_focus,
            v0::Presence::Busy => theme.stoat.presence_busy,
            v0::Presence::Invisible => theme.stoat.presence_invisible,
        };

        rect()
            .layout(self.layout.clone())
            .layer(self.layer)
            .mask(RenderCallback::new({
                let presence = self.presence.clone();

                move |context| {
                    let RenderTarget::Background { canvas, .. } = &context.target else {
                        return;
                    };

                    let mut white = Paint::default();
                    white.set_anti_alias(true);
                    white.set_color(Color::WHITE);

                    let clip_path = match &presence {
                        v0::Presence::Online => None,
                        v0::Presence::Idle => Some(Path::circle(
                            Point::new(
                                context.origin.x + (context.size.width * (3. / 10.)),
                                context.origin.y + (context.size.width * (3. / 10.)),
                            ),
                            context.size.width * (4. / 10.),
                            PathDirection::CW,
                        )),
                        v0::Presence::Focus => {
                            canvas.draw_circle(
                                Point::new(
                                    context.origin.x + (context.size.width / 2.),
                                    context.origin.y + (context.size.height / 2.),
                                ),
                                context.size.width * (2. / 10.),
                                &white,
                            );

                            Some(Path::circle(
                                Point::new(
                                    context.origin.x + (context.size.width / 2.),
                                    context.origin.y + (context.size.height / 2.),
                                ),
                                context.size.width * (4. / 10.),
                                PathDirection::CW,
                            ))
                        }
                        v0::Presence::Busy => Some(Path::rect(
                            Rect::new(
                                context.origin.x + (context.size.width * (2. / 10.)),
                                context.origin.y + (context.size.height * (4. / 10.)),
                                context.origin.x + (context.size.width * (8. / 10.)),
                                context.origin.y + (context.size.height * (6. / 10.)),
                            ),
                            PathDirection::CW,
                        )),
                        v0::Presence::Invisible => Some(Path::circle(
                            Point::new(
                                context.origin.x + (context.size.width / 2.),
                                context.origin.y + (context.size.height / 2.),
                            ),
                            context.size.width * (3. / 10.),
                            PathDirection::CW,
                        )),
                    };

                    if let Some(path) = clip_path {
                        canvas.clip_path(&path, ClipOp::Difference, true);
                    }

                    canvas.draw_circle(
                        Point::new(
                            context.origin.x + (context.size.width / 2.),
                            context.origin.y + (context.size.height / 2.),
                        ),
                        context.size.width / 2.,
                        &white,
                    );
                }
            }))
            .background(color)
    }
}
