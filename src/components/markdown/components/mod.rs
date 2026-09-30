use freya::{prelude::*, radio::Readable};
use stoat_models::v0;

mod channel_mention;
mod codeblock;
mod emoji;
mod hyperlink;
mod invite;
mod link;
mod message_mention;
mod role_mention;
mod spoiler;
mod user_mention;

pub use channel_mention::*;
pub use codeblock::*;
pub use emoji::*;
pub use hyperlink::*;
pub use invite::*;
pub use link::*;
pub use message_mention::*;
pub use role_mention::*;
pub use spoiler::*;
pub use user_mention::*;

pub fn consume_server() -> Option<Readable<v0::Server>> {
    consume_context()
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct MessageUrl(pub String);
