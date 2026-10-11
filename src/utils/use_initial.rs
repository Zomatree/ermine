use std::ops::{Deref, DerefMut};

use freya::prelude::*;

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

    pub fn is_different(&self) -> bool
    where
        T: PartialEq,
    {
        let initial = &*self.initial.read();
        let current = &*self.current.read();

        initial != current
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

    pub fn as_current(&self) -> State<T> {
        self.current
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