use freya::prelude::*;
use stoat_models::v0;

use crate::{components::AnimatedImage, format_autumn_url};

pub fn file_image(file: &v0::File) -> AnimatedImage {
    AnimatedImage::new(format_autumn_url(file)).sampling_mode(SamplingMode::Trilinear)
}
