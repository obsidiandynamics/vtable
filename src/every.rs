//! An improvement upon [Any], introducing type name support. Failing to downcast now returns a useful error,
//! containing the type name of the source. (The [Any] implementation only captures the source [TypeId].)
//!
//! Note: some methods from `impl dyn Any` and `impl Box<dyn Any>` were copied verbatim (prefixed with `__`)
//! as upcasting coercion from `&dyn Every` to `&dyn Any` was not stable at the time.
//! See [feature(trait_upcasting)](https://github.com/rust-lang/rust/issues/65991).

use std::any;
use std::any::{Any, TypeId};
use std::error::Error;
use std::fmt::{Display, Formatter};

pub trait Every: Any {
    fn type_name(&self) -> &'static str;
}

impl<T: 'static + ?Sized> Every for T {
    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }
}

pub trait AsEvery: Every {
    fn as_every(&self) -> &dyn Every;
}

impl<T: 'static> AsEvery for T {
    #[inline]
    fn as_every(&self) -> &dyn Every {
        self
    }
}

pub trait AsEveryMut: AsEvery {
    fn as_every_mut(&mut self) -> &mut dyn Every;
}

impl<T: 'static> AsEveryMut for T {
    #[inline]
    fn as_every_mut(&mut self) -> &mut dyn Every {
        self
    }
}

pub trait IntoEvery: Every {
    fn into_every(self: Box<Self>) -> Box<dyn Every>;
}

impl<T: 'static> IntoEvery for T {
    #[inline]
    fn into_every(self: Box<Self>) -> Box<dyn Every> {
        self
    }
}

impl dyn Every {
    #[inline]
    pub fn is<T: Every>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    #[inline]
    pub fn downcast_ref<T: Every>(&self) -> Result<&T, DowncastError> {
        self.__downcast_ref::<T>()
            .ok_or_else(|| cannot_downcast::<T>(self))
    }

    #[inline]
    pub fn downcast_mut<T: Every>(&mut self) -> Result<&mut T, DowncastError> {
        let self_ptr: *const dyn Every = self;
        self.__downcast_mut::<T>().ok_or_else(|| {
            // SAFETY: when `Option::None` is returned, no mutable references to self are held
            let self_alias = unsafe { &*self_ptr };
            cannot_downcast::<T>(self_alias)
        })
    }

    #[inline]
    fn __downcast_ref<T: Every>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.__downcast_ref_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    fn __downcast_mut<T: Every>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.__downcast_mut_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    unsafe fn __downcast_ref_unchecked<T: Every>(&self) -> &T {
        debug_assert!(self.is::<T>());
        // SAFETY: caller guarantees that T is the correct type
        unsafe { &*(self as *const dyn Every as *const T) }
    }

    #[inline]
    unsafe fn __downcast_mut_unchecked<T: Every>(&mut self) -> &mut T {
        debug_assert!(self.is::<T>());
        // SAFETY: caller guarantees that T is the correct type
        unsafe { &mut *(self as *mut dyn Every as *mut T) }
    }
}

impl dyn Every + Send {
    #[inline]
    pub fn is<T: Every>(&self) -> bool {
        <dyn Every>::is::<T>(self)
    }

    #[inline]
    pub fn downcast_ref<T: Every>(&self) -> Result<&T, DowncastError> {
        <dyn Every>::downcast_ref::<T>(self)
    }

    #[inline]
    pub fn downcast_mut<T: Every>(&mut self) -> Result<&mut T, DowncastError> {
        <dyn Every>::downcast_mut::<T>(self)
    }
}

impl dyn Every + Send + Sync {
    #[inline]
    pub fn is<T: Every>(&self) -> bool {
        <dyn Every>::is::<T>(self)
    }

    #[inline]
    pub fn downcast_ref<T: Every>(&self) -> Result<&T, DowncastError> {
        <dyn Every>::downcast_ref::<T>(self)
    }

    #[inline]
    pub fn downcast_mut<T: Every>(&mut self) -> Result<&mut T, DowncastError> {
        <dyn Every>::downcast_mut::<T>(self)
    }
}

fn cannot_downcast<T: Every>(source: &dyn Every) -> DowncastError {
    DowncastError {
        source_type_id: source.type_id(),
        source_type_name: source.type_name(),
        target_type_id: TypeId::of::<T>(),
        target_type_name: any::type_name::<T>(),
    }
}

/// Extension methods for `Box<dyn Every>`.
pub trait BoxDowncast {
    fn downcast<T: 'static>(self) -> Result<T, DowncastError>;
}

impl BoxDowncast for Box<dyn Every> {
    #[inline]
    fn downcast<T: 'static>(self) -> Result<T, DowncastError> {
        __downcast::<T>(self)
            .map(|this| *this)
            .map_err(|this| cannot_downcast::<T>(&*this))
    }
}

impl BoxDowncast for Box<dyn Every + Send> {
    #[inline]
    fn downcast<T: 'static>(self) -> Result<T, DowncastError> {
        <Box<dyn Every>>::downcast(self)
    }
}

impl BoxDowncast for Box<dyn Every + Send + Sync> {
    #[inline]
    fn downcast<T: 'static>(self) -> Result<T, DowncastError> {
        <Box<dyn Every>>::downcast(self)
    }
}

#[inline]
fn __downcast<T: Every>(s: Box<dyn Every>) -> Result<Box<T>, Box<dyn Every>> {
    if s.is::<T>() {
        unsafe { Ok(__downcast_unchecked::<T>(s)) }
    } else {
        Err(s)
    }
}

#[inline]
unsafe fn __downcast_unchecked<T: Every>(s: Box<dyn Every>) -> Box<T> {
    debug_assert!(s.is::<T>());
    let raw: *mut dyn Every = Box::into_raw(s);
    unsafe {
        Box::from_raw(raw as *mut T)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DowncastError {
    pub source_type_id: TypeId,
    pub source_type_name: &'static str,
    pub target_type_id: TypeId,
    pub target_type_name: &'static str,
}

impl Display for DowncastError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cannot downcast {} into {}",
            self.source_type_name, self.target_type_name
        )
    }
}

impl Error for DowncastError {}

pub fn panic<R>(error: impl Error) -> R {
    panic!("{error}")
}

#[cfg(test)]
mod tests {
    use crate::every::{panic, BoxDowncast, DowncastError, Every};
    use std::any::TypeId;
    use std::panic::AssertUnwindSafe;
    use std::{any, panic};

    #[test]
    fn is() {
        let val = Box::new(42i32) as Box<dyn Every>;
        assert!(val.is::<i32>());
        assert!(!val.is::<&str>());
    }

    #[test]
    fn downcast_ref_ok() {
        let val = Box::new(42i32) as Box<dyn Every>;
        assert_eq!(Ok(&42i32), val.downcast_ref());
    }

    #[test]
    fn downcast_ref_error() {
        let val = Box::new(42i32) as Box<dyn Every>;
        assert_eq!(
            Err(DowncastError {
                source_type_id: TypeId::of::<i32>(),
                source_type_name: any::type_name::<i32>(),
                target_type_id: TypeId::of::<&str>(),
                target_type_name: any::type_name::<&str>(),
            }),
            val.downcast_ref::<&str>()
        );
    }

    #[test]
    fn downcast_mut_ok() {
        let mut val = Box::new(42i32) as Box<dyn Every>;
        *val.downcast_mut().unwrap() = 13;
        assert_eq!(Ok(&13i32), val.downcast_ref());
    }

    #[test]
    fn downcast_mut_error() {
        let mut val = Box::new(42i32) as Box<dyn Every>;
        assert_eq!(
            Err(DowncastError {
                source_type_id: TypeId::of::<i32>(),
                source_type_name: any::type_name::<i32>(),
                target_type_id: TypeId::of::<&str>(),
                target_type_name: any::type_name::<&str>(),
            }),
            val.downcast_mut::<&str>()
        );
    }

    #[test]
    fn downcast_ok() {
        let val = Box::new(42i32) as Box<dyn Every>;
        assert_eq!(Ok(42i32), val.downcast());
    }

    #[test]
    fn downcast_error() {
        let val = Box::new(42i32) as Box<dyn Every>;
        assert_eq!(
            Err(DowncastError {
                source_type_id: TypeId::of::<i32>(),
                source_type_name: any::type_name::<i32>(),
                target_type_id: TypeId::of::<&str>(),
                target_type_name: any::type_name::<&str>(),
            }),
            val.downcast::<&str>()
        );
    }

    #[test]
    fn downcast_with_panic() {
        let val = Box::new(42i32) as Box<dyn Every>;
        let p = panic::catch_unwind(AssertUnwindSafe(|| {
            let _ = val.downcast_ref::<&str>().unwrap_or_else(panic);
        }));
        let err = p.unwrap_err();
        assert_eq!(
            "cannot downcast i32 into &str",
            err.downcast_ref::<String>().unwrap()
        );
    }

    #[test]
    fn dyn_any_send_requirement() {
        let val = &mut 42i32 as &mut (dyn Every + Send);
        assert_eq!(Ok(&42i32), val.downcast_ref());
        *val.downcast_mut().unwrap() = 13;
        assert_eq!(Ok(&13i32), val.downcast_ref());
    }

    #[test]
    fn dyn_any_send_sync_requirement() {
        let val = &mut 42i32 as &mut (dyn Every + Send + Sync);
        assert_eq!(Ok(&42i32), val.downcast_ref());
        *val.downcast_mut().unwrap() = 13;
        assert_eq!(Ok(&13i32), val.downcast_ref());
    }

    #[test]
    fn box_dyn_any_send_requirement() {
        let val = Box::new(42i32) as Box<dyn Every + Send>;
        assert_eq!(Ok(42i32), val.downcast());
    }

    #[test]
    fn box_dyn_any_send_sync_requirement() {
        let val = Box::new(42i32) as Box<dyn Every + Send + Sync>;
        assert_eq!(Ok(42i32), val.downcast());
    }
}
