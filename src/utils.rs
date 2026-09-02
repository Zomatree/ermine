use std::{
    cell::Ref,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::{Arc, LazyLock},
    time::{Duration, SystemTime},
};

use freya::{
    prelude::*,
    radio::{Radio, Readable},
};
use indexmap::IndexMap;
use rfd::AsyncFileDialog;
use stoat_models::v0;

use crate::{
    AppChannel, AppState, ChannelUnread, LocalFile, NotificationBadge, NotificationsSettings, Tag,
    color::parse_fill, http,
};

pub fn map_readable<T, U: PartialEq>(
    readable: Readable<T>,
    f: impl Fn(&T) -> &U + 'static,
) -> Readable<U> {
    let f = Rc::new(f);

    Readable::new(
        {
            let readable = readable.clone();
            let f = f.clone();

            move || {
                let f = f.clone();

                let ReadableRef::Ref(r) = readable.read() else {
                    panic!("Unsupported")
                };

                ReadableRef::Ref(r.map(move |r| Ref::map(r, |v| f(v))))
            }
        },
        {
            let readable = readable.clone();
            let f = f.clone();

            move || {
                let f = f.clone();

                let ReadableRef::Ref(r) = readable.peek() else {
                    panic!("Unsupported")
                };

                ReadableRef::Ref(r.map(move |r| Ref::map(r, |v| f(v))))
            }
        },
        {
            let readable = readable.clone();
            let f = f.clone();

            move |other| {
                let f = f.clone();

                let ReadableRef::Ref(r) = readable.peek() else {
                    panic!("Unsupported")
                };

                f(&*r) == other
            }
        },
    )
}

pub struct OptionalReadable<T: 'static> {
    pub(crate) read_fn: Rc<dyn Fn() -> Option<ReadableRef<T>>>,
    pub(crate) peek_fn: Rc<dyn Fn() -> Option<ReadableRef<T>>>,
}

impl<T: 'static> Clone for OptionalReadable<T> {
    fn clone(&self) -> Self {
        Self {
            read_fn: self.read_fn.clone(),
            peek_fn: self.peek_fn.clone(),
        }
    }
}

impl<T: 'static> OptionalReadable<T> {
    pub fn new(
        read_fn: Box<dyn Fn() -> Option<ReadableRef<T>>>,
        peek_fn: Box<dyn Fn() -> Option<ReadableRef<T>>>,
    ) -> Self {
        Self {
            read_fn: Rc::from(read_fn),
            peek_fn: Rc::from(peek_fn),
        }
    }

    pub fn none() -> Self {
        Self::new(Box::new(|| None), Box::new(|| None))
    }

    pub fn read(&self) -> Option<ReadableRef<T>> {
        (self.read_fn)()
    }

    pub fn peek(&self) -> Option<ReadableRef<T>> {
        (self.peek_fn)()
    }
}

impl<T: 'static> PartialEq for OptionalReadable<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

pub fn map_optional_readable<T, U>(
    readable: Readable<T>,
    f: impl Fn(&T) -> Option<&U> + 'static,
) -> OptionalReadable<U> {
    let f = Rc::new(f);

    OptionalReadable::new(
        Box::new({
            let readable = readable.clone();
            let f = f.clone();

            move || {
                let f = f.clone();

                let ReadableRef::Ref(r) = readable.read() else {
                    panic!("Unsupported")
                };

                r.try_map(|r| Ref::filter_map(r, |v| f(v)).ok())
                    .map(ReadableRef::Ref)
            }
        }),
        Box::new({
            let readable = readable.clone();
            let f = f.clone();

            move || {
                let f = f.clone();

                let ReadableRef::Ref(r) = readable.peek() else {
                    panic!("Unsupported")
                };

                r.try_map(|r| Ref::filter_map(r, |v| f(v)).ok())
                    .map(ReadableRef::Ref)
            }
        }),
    )
}

pub fn member_display_color(member: &v0::Member, server: &v0::Server) -> Option<Fill> {
    let mut roles = member
        .roles
        .iter()
        .filter_map(|id| server.roles.get(id))
        .collect::<Vec<_>>();

    roles.sort_by(|a, b| a.rank.cmp(&b.rank));

    let color = roles
        .into_iter()
        .filter_map(|role| role.colour.as_ref())
        .next()?;

    parse_fill(color)
}

pub fn is_channel_muted(
    channel_id: &str,
    settings: Readable<Option<NotificationsSettings>>,
) -> bool {
    let mute = settings
        .read()
        .as_ref()
        .and_then(|n| n.channel_mutes.get(channel_id).cloned());

    mute.is_some_and(|mute| {
        mute.until.is_none_or(|ts| {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                < ts
        })
    })
}

pub fn is_server_muted(server_id: &str, settings: Readable<Option<NotificationsSettings>>) -> bool {
    let mute = settings
        .read()
        .as_ref()
        .and_then(|n| n.server_mutes.get(server_id).cloned());

    mute.is_some_and(|mute| {
        mute.until.is_none_or(|ts| {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                < ts
        })
    })
}

pub fn get_unread_badge(
    channel: &v0::Channel,
    unread: &ChannelUnread,
) -> Option<NotificationBadge> {
    if !unread.mentions.is_empty() {
        Some(NotificationBadge::Mentions(unread.mentions.len()))
    } else {
        let last_message_id = match &channel {
            v0::Channel::TextChannel {
                last_message_id, ..
            }
            | v0::Channel::Group {
                last_message_id, ..
            }
            | v0::Channel::DirectMessage {
                last_message_id, ..
            } => last_message_id.as_ref(),
            _ => None,
        };

        if (unread.last_id.is_none() && last_message_id.is_some())
            || unread
                .last_id
                .as_ref()
                .zip(last_message_id)
                .is_some_and(|(last_id, last_message_id)| last_id < last_message_id)
        {
            Some(NotificationBadge::Unread)
        } else {
            None
        }
    }
}

static UNICODE_EMOJIS: LazyLock<Arc<IndexMap<String, String>>> = LazyLock::new(|| {
    Arc::new(serde_json::from_str(include_str!("./assets/emojiMapping.json")).unwrap())
});

pub fn get_unicode_emojis() -> Arc<IndexMap<String, String>> {
    UNICODE_EMOJIS.clone()
}

pub struct Initial<T> {
    initial: State<T>,
    current: State<T>,
}

impl<T: 'static> PartialEq for Initial<T> {
    fn eq(&self, other: &Self) -> bool {
        self.initial == other.initial && self.current == other.current
    }
}

impl<T: 'static> Eq for Initial<T> {}

impl<T> Clone for Initial<T> {
    fn clone(&self) -> Self {
        Self {
            initial: self.initial,
            current: self.current,
        }
    }
}

impl<T> Copy for Initial<T> {}

impl<T: Clone + 'static> Initial<T> {
    pub fn reset(&mut self) {
        self.current.set(self.initial.read().clone());
    }

    pub fn get_if_different(&self) -> Option<T>
    where
        T: PartialEq,
    {
        let initial = &*self.initial.read();
        let current = &*self.current.read();

        if initial == current {
            None
        } else {
            Some(current.clone())
        }
    }

    pub fn set_new(&mut self, value: T) {
        self.initial.set(value.clone());
        self.current.set(value);
    }

    pub fn apply(&mut self) {
        self.initial.set(self.current.read().cloned());
    }
}

impl<T> Deref for Initial<T> {
    type Target = State<T>;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl<T> DerefMut for Initial<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.current
    }
}

impl<T: 'static> IntoWritable<T> for Initial<T> {
    fn into_writable(self) -> Writable<T> {
        self.current.into_writable()
    }
}

impl<T: 'static> IntoReadable<T> for Initial<T> {
    fn into_readable(self) -> Readable<T> {
        self.current.into_readable()
    }
}

impl<T: 'static> Into<Writable<T>> for Initial<T> {
    fn into(self) -> Writable<T> {
        self.into_writable()
    }
}

impl<T: 'static> Into<Readable<T>> for Initial<T> {
    fn into(self) -> Readable<T> {
        self.into_readable()
    }
}

pub fn use_initial<T: Clone + 'static>(f: impl FnOnce() -> T) -> Initial<T> {
    use_hook(|| {
        let value = f();
        let initial = State::create(value.clone());
        let current = State::create(value);
        Initial { initial, current }
    })
}

/// Methods for setting an element's width and height.
pub trait SizeExt
where
    Self: ContainerSizeExt,
{
    /// Set the element's height and width. See [`Size`].
    fn size(mut self, size: impl Into<Size>) -> Self {
        let size = size.into();
        let layout = self.get_layout();

        layout.layout.width = size.clone();
        layout.layout.height = size;

        self
    }
}

impl<T: ContainerSizeExt> SizeExt for T {}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectedRole {
    Default,
    Role(String),
}

pub fn get_channel_server(channel: &v0::Channel) -> Option<&str> {
    match channel {
        v0::Channel::TextChannel { server, .. } => Some(server),
        _ => None,
    }
}

pub fn use_clipboard() -> State<arboard::Clipboard> {
    use_hook(|| match try_consume_root_context() {
        Some(state) => state,
        None => {
            let state = State::create_global(
                arboard::Clipboard::new().expect("Failed to connect to clipboard"),
            );
            provide_root_context(state);
            state
        }
    })
}

pub async fn prompt_image_upload(tag: Tag) -> Option<String> {
    if let Some(file) = AsyncFileDialog::new().pick_file().await {
        let contents = file.read().await.into();
        let filename = file.file_name();

        if let Ok(response) = http()
            .upload_file(
                tag.as_str(),
                LocalFile {
                    name: filename,
                    body: contents,
                },
            )
            .await
        {
            return Some(response.id);
        };
    };

    None
}

pub fn get_channel_name(radio: &Radio<AppState, AppChannel>, channel: &v0::Channel) -> String {
    match channel {
        v0::Channel::SavedMessages { .. } => "Saved Messages".to_string(),
        v0::Channel::DirectMessage { recipients, .. } => {
            let state = radio.peek_state();
            let user_id = state.user_id.as_ref().unwrap();

            let other = recipients.iter().find(|id| id != &user_id).unwrap();

            let users = radio.slice(AppChannel::Users, |state| &state.users);

            users.read().get(other).unwrap().username.clone()
        }
        v0::Channel::Group { name, .. } => name.clone(),
        v0::Channel::TextChannel { name, .. } => name.clone(),
    }
}

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

pub fn format_duration(duration: jiff::Span) -> String {
    format!(
        "{:#}",
        duration.nanoseconds(0).microseconds(0).milliseconds(0)
    )
}
// pub fn map_optional_readable<T, U>(
//     readable: Readable<T>,
//     f: impl Fn(&T) -> Option<&U> + 'static,
// ) -> Readable<Option<U>> {
//     let f = Rc::new(f);

//     debug_assert!(Layout::new::<&Option<U>>() == Layout::new::<&U>());

//     Readable::new(
//         Box::new({
//             let readable = readable.clone();
//             let f = f.clone();

//             move || {
//                 let f = f.clone();

//                 let ReadableRef::Ref(r) = readable.read() else {
//                     panic!("Unsupported")
//                 };

//                 ReadableRef::Ref(r.map(|r| match Ref::filter_map(r, |v| f(v)) {
//                     Ok(r) => unsafe { transmute(r) },
//                     Err(r) => Ref::map(r, |_| &None),
//                 }))
//             }
//         }),
//         Box::new({
//             let readable = readable.clone();
//             let f = f.clone();

//             move || {
//                 let f = f.clone();

//                 let ReadableRef::Ref(r) = readable.peek() else {
//                     panic!("Unsupported")
//                 };

//                 ReadableRef::Ref(r.map(|r| match Ref::filter_map(r, |v| f(v)) {
//                     Ok(r) => unsafe { transmute::<Ref<'_, U>, Ref<'_, Option<U>>>(r) },
//                     Err(r) => Ref::map(r, |_| &None),
//                 }))
//             }
//         }),
//     )
// }

// pub struct MapSlice<Value, SliceValue, MapValue, Channel>
// where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static,
//     MapValue: 'static,
// {
//     slice: RadioSlice<Value, SliceValue, Channel>,
//     f: Rc<dyn Fn(&SliceValue) -> &MapValue + 'static>,
// }

// impl<Value, SliceValue, MapValue, Channel> MapSlice<Value, SliceValue, MapValue, Channel>
// where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static,
//     MapValue: 'static,
// {
//     pub fn new(
//         slice: RadioSlice<Value, SliceValue, Channel>,
//         f: Rc<dyn Fn(&SliceValue) -> &MapValue + 'static>,
//     ) -> Self {
//         Self { slice, f }
//     }
// }

// impl<Value, SliceValue, MapValue, Channel> Clone for MapSlice<Value, SliceValue, MapValue, Channel>
// where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static,
//     MapValue: 'static,
// {
//     fn clone(&self) -> Self {
//         Self {
//             slice: self.slice.clone(),
//             f: self.f.clone(),
//         }
//     }
// }

// impl<Value, SliceValue, MapValue, Channel> PartialEq
//     for MapSlice<Value, SliceValue, MapValue, Channel>
// where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static,
//     MapValue: 'static,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.slice == other.slice
//     }
// }

// impl<Value, SliceValue, MapValue, Channel> IntoReadable<MapValue>
//     for MapSlice<Value, SliceValue, MapValue, Channel>
// where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static,
//     MapValue: 'static,
// {
//     fn into_readable(self) -> Readable<MapValue> {
//         Readable::new(
//             Box::new({
//                 let readable = self.slice.clone();
//                 let f = self.f.clone();

//                 move || {
//                     let f = f.clone();
//                     let readable = readable.clone();

//                     ReadableRef::Ref(
//                         readable
//                             .read_unchecked()
//                             .map(move |r| Ref::map(r, |v| f(v))),
//                     )
//                 }
//             }),
//             Box::new({
//                 let readable = self.slice.clone();
//                 let f = self.f.clone();

//                 move || {
//                     let f = f.clone();
//                     let readable = readable.clone();

//                     ReadableRef::Ref(
//                         readable
//                             .peek_unchecked()
//                             .map(move |r| Ref::map(r, |v| f(v))),
//                     )
//                 }
//             }),
//         )
//     }
// }

// pub struct OptionalSlice<Value, SliceValue, Channel> where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static
// {
//     channel: Channel,
//     station: RadioStation<Value, Channel>,
//     selector: Rc<dyn Fn(&Value) -> Option<&SliceValue> + 'static>
// }

// impl<Value, SliceValue, Channel> OptionalSlice<Value, SliceValue, Channel> where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static
// {
//     pub fn new(
//         channel: Channel,
//         station: RadioStation<Value, Channel>,
//         selector: impl Fn(&Value) -> Option<&SliceValue> + 'static,
//     ) -> Self {
//         Self {
//             channel,
//             station,
//             selector: Rc::new(selector),
//         }
//     }

//     pub fn peek_unchecked(&self) -> Option<ReadRef<'static, SliceValue>> {
//         let inner = self.station.peek_unchecked();

//         inner.try_map(|v| {
//             let o = Ref::filter_map(v, |v| {
//                 (self.selector)(v)
//             });

//             o.ok()
//         })
//     }
// }
