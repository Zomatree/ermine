use std::sync::{Arc, LazyLock};

use indexmap::{IndexMap, IndexSet};

static UNICODE_EMOJIS: LazyLock<Arc<IndexMap<String, String>>> = LazyLock::new(|| {
    Arc::new(serde_json::from_str(include_str!("../assets/emojiMapping.json")).unwrap())
});

static UNICODE_EMOJIS_SET: LazyLock<Arc<IndexSet<String>>> =
    LazyLock::new(|| Arc::new(IndexSet::from_iter(UNICODE_EMOJIS.values().cloned())));

pub fn get_unicode_emojis() -> Arc<IndexMap<String, String>> {
    UNICODE_EMOJIS.clone()
}

pub fn get_unicode_emoji_set() -> Arc<IndexSet<String>> {
    UNICODE_EMOJIS_SET.clone()
}
