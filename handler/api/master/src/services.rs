//! The services of api-master, looked up by type.
//!
//! Every feature's command interface (`Arc<dyn Supg003Commands>`, `Arc<dyn AccessControl>` ...) is
//! registered once at start-up (`wiring/<feature>.rs`) and handed to a route handler that asks for
//! it with `State<Arc<dyn ...>>`. The registry is keyed by the type, so adding a feature means
//! adding its own wiring file, not a field, an `Ok(Self { .. })` line and a `FromRef` impl in one
//! shared file (the merge conflicts that came with that).

use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::sync::Arc;

type Registry = HashMap<TypeId, Box<dyn Any + Send + Sync>>;

/// The registry while it is being filled (start-up only)
#[derive(Default)]
pub struct ServicesBuilder {
    map: Registry,
}

impl ServicesBuilder {
    /// Registers `service` for requests of type `Arc<T>` (a trait object, usually). A second
    /// registration of the same type replaces the first.
    pub fn insert<T: ?Sized + Send + Sync + 'static>(&mut self, service: Arc<T>) {
        self.map.insert(TypeId::of::<Arc<T>>(), Box::new(service));
    }

    /// A service registered earlier (features that build on another one)
    pub fn get<T: ?Sized + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        lookup::<T>(&self.map)
    }

    /// Like [`get`](Self::get), for a service that must be there: wiring order is wrong otherwise
    pub fn require<T: ?Sized + Send + Sync + 'static>(&self) -> Arc<T> {
        self.get::<T>().unwrap_or_else(|| {
            panic!("service {} is not registered yet: wire it before the features that need it (wiring/mod.rs)", type_name::<T>())
        })
    }

    pub fn build(self) -> Services {
        Services(Arc::new(self.map))
    }
}

/// The finished registry (cheap to clone)
#[derive(Clone)]
pub struct Services(Arc<Registry>);

impl Services {
    pub fn get<T: ?Sized + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        lookup::<T>(&self.0)
    }

    /// A service the handler cannot work without; a missing one is a start-up wiring mistake
    pub fn require<T: ?Sized + Send + Sync + 'static>(&self) -> Arc<T> {
        self.get::<T>().unwrap_or_else(|| {
            panic!("service {} was never registered: add its wiring file and list it in wiring/mod.rs", type_name::<T>())
        })
    }

    /// Whether a service is registered (the start-up self-check)
    pub fn has<T: ?Sized + Send + Sync + 'static>(&self) -> bool {
        self.0.contains_key(&TypeId::of::<Arc<T>>())
    }
}

fn lookup<T: ?Sized + Send + Sync + 'static>(map: &Registry) -> Option<Arc<T>> {
    map.get(&TypeId::of::<Arc<T>>()).and_then(|b| b.downcast_ref::<Arc<T>>()).cloned()
}
