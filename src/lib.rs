use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use crate::every::{panic, Every};

pub mod ched;
pub mod every;
pub mod vtable;

type PartialEqFn = fn(&dyn Every, &dyn Every) -> bool;

pub fn partial_eq<T: PartialEq + 'static>(this: &dyn Every, other: &dyn Every) -> bool {
    let lhs = this.downcast_ref::<T>().unwrap_or_else(panic);
    let rhs = other.downcast_ref::<T>();
    rhs.is_ok_and(|rhs| lhs == rhs)
}

type DebugFn = fn(&dyn Every, &mut Formatter<'_>) -> Result<(), core::fmt::Error>;

pub fn debug<T: Debug + 'static>(
    this: &dyn Every,
    f: &mut Formatter<'_>,
) -> Result<(), core::fmt::Error> {
    let value = this.downcast_ref::<T>().unwrap_or_else(panic);
    value.fmt(f)
}

pub type CloneFn = fn(&dyn Every) -> Box<dyn Every>;

pub fn clone<T: Clone + 'static>(this: &dyn Every) -> Box<dyn Every> {
    let value = this.downcast_ref::<T>().unwrap_or_else(panic);
    let cloned = value.clone();
    Box::new(cloned)
}

pub type HashFn = fn(&dyn Every, &mut dyn Hasher);

pub fn hash<T: Hash + 'static>(this: &dyn Every, mut state: &mut dyn Hasher) {
    let this = this.downcast_ref::<T>().unwrap_or_else(panic);
    this.hash(&mut state);
}
