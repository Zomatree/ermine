use std::{
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use freya::{elements::image::ImageData, prelude::*};
use freya_components::gif_viewer::{GifSource, GifViewer};
use mimetype_detector::IMAGE_GIF;

use crate::components::{ModalValue, use_modals};

#[derive(PartialEq)]
pub struct AnimatedImage {
    url: Url,
    selectable: bool,

    layout: LayoutData,
    image_data: ImageData,
    asset_age: AssetAge,
    accessibility: AccessibilityData,
    corner_radius: Option<CornerRadius>,
    effect: EffectData,
    decode_mode: DecodeMode,

    key: DiffKey,
}

impl AnimatedImage {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            selectable: false,
            layout: LayoutData::default(),
            image_data: ImageData::default(),
            asset_age: AssetAge::default(),
            accessibility: AccessibilityData::default(),
            corner_radius: None,
            effect: EffectData::default(),
            decode_mode: DecodeMode::default(),
            key: DiffKey::None,
        }
    }
}

impl KeyExt for AnimatedImage {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl LayoutExt for AnimatedImage {
    fn get_layout(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl ContainerSizeExt for AnimatedImage {}
impl ContainerWithContentExt for AnimatedImage {}
impl ContainerPositionExt for AnimatedImage {}
impl ContainerConstraintsExt for AnimatedImage {}

impl ImageExt for AnimatedImage {
    fn get_image_data(&mut self) -> &mut ImageData {
        &mut self.image_data
    }
}

impl AccessibilityExt for AnimatedImage {
    fn get_accessibility_data(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl EffectExt for AnimatedImage {
    fn get_effect(&mut self) -> &mut EffectData {
        &mut self.effect
    }
}

impl AnimatedImage {
    pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = Some(corner_radius.into());
        self
    }

    pub fn decode_mode(mut self, decode_mode: DecodeMode) -> Self {
        self.decode_mode = decode_mode;
        self
    }

    pub fn asset_age(mut self, asset_age: impl Into<AssetAge>) -> Self {
        self.asset_age = asset_age.into();
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }
}

impl Component for AnimatedImage {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        let asset_config = AssetConfiguration::new(&self.url, self.asset_age);
        use_asset(&asset_config);
        let mut asset_cacher = use_hook(AssetCacher::get);

        use_side_effect_with_deps(
            &(self.url.clone(), asset_config.clone()),
            move |(url, asset_config)| {
                if matches!(
                    asset_cacher.read_asset(asset_config),
                    Some(Asset::Pending) | Some(Asset::Error(_))
                ) {
                    asset_cacher.update_asset(asset_config.clone(), Asset::Loading);

                    let url = url.clone();
                    let asset_config = asset_config.clone();
                    spawn_forever(async move {
                        match tokio::task::spawn_blocking(move || {
                            let mut response = reqwest::blocking::get(url.clone()).unwrap();

                            if response.status().is_redirection() {
                                let location = response
                                    .headers()
                                    .get("location")
                                    .unwrap()
                                    .to_str()
                                    .unwrap();
                                let mut url = url.clone();
                                url.set_path(location);
                                response = reqwest::blocking::get(url.clone()).unwrap();
                            };

                            let bytes = response.bytes().unwrap();

                            let mime = mimetype_detector::detect(&bytes);

                            Some((bytes, mime.is(IMAGE_GIF)))
                        })
                        .await
                        {
                            Ok(Some((bytes, is_gif))) => {
                                asset_cacher.update_asset(
                                    asset_config,
                                    Asset::Cached(Rc::new((bytes, is_gif))),
                                );
                            }
                            Err(_) | Ok(None) => {
                                asset_cacher
                                    .update_asset(asset_config, Asset::Error("todo".to_string()));
                            }
                        }
                    });
                }
            },
        );

        let asset = asset_cacher.read_asset(&asset_config).unwrap();

        match asset {
            Asset::Cached(asset) => {
                let (bytes, is_gif) = asset.downcast_ref::<(Bytes, bool)>().unwrap().clone();
                let mut hasher = DefaultHasher::default();
                self.url.hash(&mut hasher);
                let id = hasher.finish();

                rect()
                    .layout(self.layout.clone())
                    .effect(self.effect.clone())
                    .accessibility(self.accessibility.clone())
                    .a11y_role(AccessibilityRole::Image)
                    .map(self.corner_radius, |rect, corner_radius| {
                        rect.corner_radius(corner_radius).overflow(Overflow::Clip)
                    })
                    .maybe(self.selectable, |this| {
                        this.cursor(CursorIcon::Pointer).on_press({
                            let url = self.url.clone();
                            move |e: Event<PressEventData>| {
                                e.stop_propagation();

                                modals
                                    .write()
                                    .push_modal(ModalValue::ImageViewer(url.clone()))
                            }
                        })
                    })
                    .child(if is_gif {
                        GifViewer::new(GifSource::Bytes(id, bytes))
                            .layout(self.layout.clone())
                            .image_data(self.image_data.clone())
                            .into_element()
                    } else {
                        ImageViewer::new(ImageSource::Bytes(id, bytes))
                            .decode_mode(self.decode_mode)
                            .layout(self.layout.clone())
                            .image_data(self.image_data.clone())
                            .into_element()
                    })
                    .into_element()
            }
            Asset::Loading | Asset::Pending => rect()
                .layout(self.layout.clone())
                .center()
                .child(CircularLoader::new())
                .into_element(),
            Asset::Error(_) => rect().child("error").into_element(),
        }
    }
}
