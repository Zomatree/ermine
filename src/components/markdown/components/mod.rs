use freya::{prelude::*, radio::Readable};
use stoat_models::v0;

mod channel_mention;
mod emoji;
mod hyperlink;
mod link;
mod role_mention;
mod spoiler;
mod user_mention;

pub use channel_mention::*;
pub use emoji::*;
pub use hyperlink::*;
pub use link::*;
pub use role_mention::*;
pub use spoiler::*;
pub use user_mention::*;

pub fn consume_server() -> Option<Readable<v0::Server>> {
    consume_context()
}
