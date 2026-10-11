use freya::prelude::*;
use stoat_models::v0;

use crate::http;

pub fn proxy_url(url: &str) -> Url {
    format!(
        "{}/proxy?url={}",
        http().api_config.features.january.url,
        url
    )
    .parse::<Url>()
    .unwrap()
}

pub fn format_autumn_url(file: &v0::File) -> Url {
    let url = if matches!(
        file.metadata,
        v0::Metadata::Image {
            animated: Some(true),
            ..
        }
    ) && &file.tag == "avatars"
    {
        format!(
            "{}/{}/{}/original",
            http().api_config.features.autumn.url,
            &file.tag,
            &file.id
        )
    } else {
        format!(
            "{}/{}/{}",
            http().api_config.features.autumn.url,
            &file.tag,
            &file.id
        )
    };

    url.parse().unwrap()
}
