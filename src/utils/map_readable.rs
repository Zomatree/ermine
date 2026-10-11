use std::{cell::Ref, rc::Rc};

use freya::{prelude::*, radio::Readable};

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


pub fn with_readable< T: 'static, R>(readable: Option<&Readable<T>>, f: impl FnOnce(Option<&T>) -> R) -> R {
    if let Some(readable) = readable {
        let value = readable.read();
        f(Some(&value))
    } else {
        f(None)
    }
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
