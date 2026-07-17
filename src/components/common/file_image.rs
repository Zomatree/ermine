use freya::prelude::*;
use stoat_models::v0;

use crate::http;

pub fn file_image(file: &v0::File) -> ImageViewer {
    ImageViewer::new(
        format!(
            "{}/{}/{}",
            http().api_config.features.autumn.url,
            &file.tag,
            &file.id
        )
        .parse::<Url>()
        .unwrap(),
    )
    .sampling_mode(SamplingMode::Trilinear)
}
