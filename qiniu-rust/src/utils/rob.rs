use std::{fmt, ops::Deref};

pub enum Rob<'t, T: ?Sized + 't> {
    Referenced(&'t T),
    Owned(Box<T>),
}

impl<'t, T: ?Sized + 't> AsRef<T> for Rob<'t, T> {
    fn as_ref(&self) -> &T {
        match self {
            Rob::Referenced(r) => r,
            Rob::Owned(r) => &r,
        }
    }
}

impl<'t, T: ?Sized + 't> Deref for Rob<'t, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'t, T: fmt::Debug + ?Sized + 't> fmt::Debug for Rob<'t, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'t, T: fmt::Display + ?Sized + 't> fmt::Display for Rob<'t, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'t, T: ?Sized + 't> From<&'t T> for Rob<'t, T> {
    #[inline]
    fn from(t: &'t T) -> Self {
        Rob::Referenced(t)
    }
}

impl<T: ?Sized> From<Box<T>> for Rob<'_, T> {
    #[inline]
    fn from(t: Box<T>) -> Self {
        Rob::Owned(t)
    }
}

impl<T> From<T> for Rob<'_, T> {
    #[inline]
    fn from(t: T) -> Self {
        Rob::Owned(Box::new(t))
    }
}
