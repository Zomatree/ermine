use std::sync::{Arc, LazyLock};

use indexmap::{IndexMap, IndexSet};
use regex::Regex;

static UNICODE_EMOJIS: LazyLock<Arc<IndexMap<String, String>>> = LazyLock::new(|| {
    Arc::new(serde_json::from_str(include_str!("../assets/emojiMapping.json")).unwrap())
});

static UNICODE_EMOJIS_SET: LazyLock<Arc<IndexSet<String>>> =
    LazyLock::new(|| Arc::new(IndexSet::from_iter(UNICODE_EMOJIS.values().cloned())));

pub static CUSTOM_EMOJI_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(":([0-9ABCDEFGHJKMNPQRSTVWXYZ]{26}):").unwrap());

pub fn get_unicode_emojis() -> Arc<IndexMap<String, String>> {
    UNICODE_EMOJIS.clone()
}

pub fn get_unicode_emoji_set() -> Arc<IndexSet<String>> {
    UNICODE_EMOJIS_SET.clone()
}
