use std::ops::Deref;

use freya::radio::{IntoReadable, Readable, use_radio};
use stoat_models::v0;

use crate::AppChannel;

pub struct KeyedReadable<T: 'static> {
    id: String,
    inner: Readable<T>,
}

impl<T: 'static> KeyedReadable<T> {
    pub fn new(id: String, readable: Readable<T>) -> Self {
        Self {
            id,
            inner: readable,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn as_readable(&self) -> &Readable<T> {
        &self.inner
    }
}

impl<T: 'static> Clone for KeyedReadable<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<T: 'static> PartialEq for KeyedReadable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.inner == other.inner
    }
}

impl<T: 'static> Deref for KeyedReadable<T> {
    type Target = Readable<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: 'static> IntoReadable<T> for KeyedReadable<T> {
    fn into_readable(self) -> Readable<T> {
        self.inner
    }
}

pub fn use_channel(id: impl Into<String>) -> KeyedReadable<v0::Channel> {
    let radio = use_radio(AppChannel::Channels);

    let id = id.into();

    KeyedReadable::new(
        id.clone(),
        radio
            .slice_current(move |state| state.channels.get(&id).unwrap())
            .into_readable(),
    )
}

pub fn use_server(id: impl Into<String>) -> KeyedReadable<v0::Server> {
    let radio = use_radio(AppChannel::Servers);

    let id = id.into();

    KeyedReadable::new(
        id.clone(),
        radio
            .slice_current(move |state| state.servers.get(&id).unwrap())
            .into_readable(),
    )
}
