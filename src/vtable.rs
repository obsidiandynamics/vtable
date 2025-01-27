use std::any::{Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{LazyLock, RwLock};

/// Specialises a vtable for [T].
pub trait Specialise<T> {
    fn specialise() -> Self;
}

#[derive(Default)]
struct Registry {
    internals: RwLock<RegistryInternals>,
}

#[derive(Default)]
struct RegistryInternals {
    types: HashMap<(TypeId, TypeId), Record>,
}

struct Record(Box<dyn Any + Sync + Send>);

impl Registry {
    fn singleton() -> &'static Registry {
        static LAZY: LazyLock<Registry> = LazyLock::new(Default::default);
        &LAZY
    }

    fn get_or_create<T: 'static, V: Specialise<T> + Sync + Send + 'static>(&self) -> &'static V {
        let mut internals = self.internals.write().unwrap();
        let key = (TypeId::of::<T>(), TypeId::of::<V>());
        let entry = internals.types.entry(key);
        match entry {
            Entry::Occupied(entry) => {
                let record = entry.get();
                let vtable = record.0.downcast_ref::<&'static V>().unwrap();
                vtable
            }
            Entry::Vacant(entry) => {
                let vtable = Box::new(V::specialise());
                let vtable: &'static V = Box::leak(vtable);
                entry.insert(Record(Box::new(vtable)));
                vtable
            }
        }
    }
}

/// A static reference to a vtable of type [V]. The [T] parameter acts as proof that
/// a [V]-type vtable has been specialised for the [T]-type value. By invoking
/// [`Token::default()`], a [T]-specialised entry for [V] is added to the
/// singleton [Registry].
pub struct Token<T, V: Sync + Send + 'static>(&'static V, PhantomData<T>);

impl<T, V: Sync + Send + 'static> Token<T, V> {
    pub fn vtable_ref(&self) -> &'static V {
        self.0
    }
}

impl<T, V: Sync + Send + 'static> Token<T, V> {
    fn create_unchecked(vtable: &'static V) -> Self {
        Self(vtable, PhantomData)
    }
}

impl<T: 'static, V: Specialise<T> + Sync + Send + 'static> Default for Token<T, V> {
    fn default() -> Self {
        let vtable = Registry::singleton().get_or_create::<T, V>();
        Self::create_unchecked(vtable)
    }
}

#[cfg(test)]
mod tests {
    use crate::vtable::{Registry, Specialise};
    use std::any::TypeId;

    impl Registry {
        fn try_get<T: 'static, V: 'static>(&self) -> Option<&'static V> {
            let internals = self.internals.read().unwrap();
            let key = (TypeId::of::<T>(), TypeId::of::<V>());
            internals
                .types
                .get(&key)
                .map(|record| *record.0.downcast_ref::<&'static V>().unwrap())
        }
    }

    #[test]
    fn singleton_registry() {
        // #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        struct Custom;

        struct VTable;

        impl<T> Specialise<T> for VTable {
            fn specialise() -> Self {
                Self
            }
        }

        {
            let virtuals = Registry::singleton();
            assert!(virtuals.try_get::<Custom, VTable>().is_none());
            let _ = virtuals.get_or_create::<Custom, VTable>();
            assert!(virtuals.try_get::<Custom, VTable>().is_some());
        }
        {
            let virtuals = Registry::singleton();
            assert!(virtuals.try_get::<Custom, VTable>().is_some());
        }
    }
}