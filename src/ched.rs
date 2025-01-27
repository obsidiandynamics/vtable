//! A dynamic object supporting [Clone], [Hash], [Eq], and [Debug] traits.

use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use crate::every::Every;
use crate::{clone, debug, hash, partial_eq, vtable, CloneFn, DebugFn, HashFn, PartialEqFn};
use crate::vtable::Specialise;

pub type Token<T> = vtable::Token<T, VTable>;

pub struct CHED {
    inner: Box<dyn Every>,
    vtable: &'static VTable,
}

impl CHED {
    #[inline]
    pub fn new<T: 'static>(value: T, tok: &Token<T>) -> Self {
        Self {
            inner: Box::new(value),
            vtable: tok.vtable_ref(),
        }
    }

    #[inline]
    #[allow(clippy::borrowed_box)]
    pub fn inner(&self) -> &Box<dyn Every> {
        &self.inner
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut Box<dyn Every> {
        &mut self.inner
    }

    #[inline]
    pub fn into_inner(self) -> Box<dyn Every> {
        self.inner
    }
}

pub struct VTable {
    clone: CloneFn,
    debug: DebugFn,
    partial_eq: PartialEqFn,
    hash: HashFn,
}

impl<T: Clone + Debug + Eq + Hash + 'static> Specialise<T> for VTable {
    fn specialise() -> Self {
        Self {
            clone: clone::<T>,
            debug: debug::<T>,
            partial_eq: partial_eq::<T>,
            hash: hash::<T>,
        }
    }
}

impl Debug for CHED {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (self.vtable.debug)(&*self.inner, f)
    }
}

impl Clone for CHED {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: (self.vtable.clone)(&*self.inner),
            vtable: self.vtable,
        }
    }
}

impl PartialEq for CHED {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.vtable.partial_eq)(&*self.inner, &*other.inner)
    }
}

impl Eq for CHED {}

impl Hash for CHED {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.vtable.hash)(&*self.inner, state);
    }
}

#[cfg(test)]
mod tests {
    use crate::every::{panic, BoxDowncast};
    use crate::vtable::Token;
    use crate::ched::CHED;
    use std::collections::HashMap;

    #[test]
    fn self_is_equal() {
        let obj = CHED::new(42, &Token::default());
        assert_eq!(obj, obj);
    }

    #[test]
    fn same_value_equal() {
        let vtable_tok_i32 = Token::default();
        let obj_1 = CHED::new(42, &vtable_tok_i32);
        let obj_2 = CHED::new(42, &vtable_tok_i32);
        assert_eq!(obj_1, obj_2);
    }

    #[test]
    fn cloned_value_equal() {
        let obj_1 = CHED::new(42, &Token::default());
        let obj_2 = obj_1.clone();
        assert_eq!(obj_1, obj_2);
    }

    #[test]
    fn different_values_not_equal() {
        let vtable_tok_i32 = Token::default();

        let obj_1 = CHED::new(42, &vtable_tok_i32);
        let obj_2 = CHED::new(43, &vtable_tok_i32);
        assert_ne!(obj_1, obj_2);
    }

    #[test]
    fn different_types_not_equal() {
        let obj_1 = CHED::new(42, &Token::default());
        let obj_2 = CHED::new("foo", &Token::default());
        assert_ne!(obj_1, obj_2);
    }

    #[test]
    fn test_debug() {
        let obj = CHED::new(42, &Token::default());
        let debug = format!("{obj:?}");
        assert_eq!("42", debug);
    }

    #[test]
    fn test_hash() {
        let vtable_tok_i32 = Token::default();
        let vtable_tok_str_slice = Token::default();

        let mut map = HashMap::new();
        assert!(map.insert(CHED::new(42, &vtable_tok_i32), ()).is_none());
        assert!(map.insert(CHED::new(43, &vtable_tok_i32), ()).is_none());
        assert!(map
            .insert(CHED::new("foo", &vtable_tok_str_slice), ())
            .is_none());

        assert!(map.insert(CHED::new(42, &vtable_tok_i32), ()).is_some());
        assert!(map.insert(CHED::new(43, &vtable_tok_i32), ()).is_some());
        assert!(map
            .insert(CHED::new("foo", &vtable_tok_str_slice), ())
            .is_some());
    }

    #[test]
    fn downcast_ref() {
        let obj = CHED::new(42i32, &Token::default());
        assert_eq!(
            &42i32,
            obj.inner().downcast_ref::<i32>().unwrap_or_else(panic)
        );
    }

    #[test]
    fn downcast_mut() {
        let mut obj_1 = CHED::new(42i32, &Token::default());
        let obj_2 = obj_1.clone();
        *obj_1.inner_mut().downcast_mut().unwrap() = 13;
        assert_eq!(
            &13i32,
            obj_1.inner().downcast_ref::<i32>().unwrap_or_else(panic)
        );
        assert_eq!(
            &42i32,
            obj_2.inner().downcast_ref::<i32>().unwrap_or_else(panic)
        );
    }

    #[test]
    fn downcast() {
        let obj = CHED::new(42i32, &Token::default());
        assert_eq!(42i32, obj.into_inner().downcast::<i32>().unwrap());
    }

    #[test]
    #[should_panic(expected = "cannot downcast i32 into u32")]
    fn downcast_ref_with_wrong_type() {
        let obj = CHED::new(42i32, &Token::default());
        let _: &u32 = obj.inner().downcast_ref().unwrap_or_else(panic);
    }

    #[test]
    #[should_panic(expected = "cannot downcast i32 into u32")]
    fn downcast_with_wrong_type() {
        let obj = CHED::new(42i32, &Token::default());
        let _: u32 = obj.into_inner().downcast().unwrap_or_else(panic);
    }
}
