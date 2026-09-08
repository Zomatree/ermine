use freya::{prelude::*, radio::Readable};
use stoat_models::v0;

mod channel_mention;
mod emoji;
mod hyperlink;
mod link;
mod message_mention;
mod role_mention;
mod spoiler;
mod user_mention;
mod codeblock;

pub use channel_mention::*;
pub use emoji::*;
pub use hyperlink::*;
pub use link::*;
pub use message_mention::*;
pub use role_mention::*;
pub use spoiler::*;
pub use user_mention::*;
pub use codeblock::*;

pub fn consume_server() -> Option<Readable<v0::Server>> {
    consume_context()
}
