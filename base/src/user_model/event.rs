use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, RwLock, Weak,
    },
};

/// RAII handle: dropping it (or calling `.unsubscribe()`) removes the listener.
pub struct Subscription<T> {
    id: u32,
    inner: Weak<Inner<T>>,
}

impl<T> Subscription<T> {
    /// explicitly unsubscribe early
    pub fn unsubscribe(self) {
        if let Some(inner) = self.inner.upgrade() {
            inner.listeners.write().unwrap().remove(&self.id);
        }
    }
}

impl<T> Drop for Subscription<T> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            inner.listeners.write().unwrap().remove(&self.id);
        }
    }
}

struct Inner<T> {
    #[cfg(target_arch = "wasm32")]
    listeners: RwLock<HashMap<u32, Box<dyn Fn(&T) + 'static>>>,
    #[cfg(not(target_arch = "wasm32"))]
    listeners: RwLock<HashMap<u32, Box<dyn Fn(&T) + Send + Sync + 'static>>>,
    next_id: AtomicU32,
}

/// A tiny, generic event emitter.
#[derive(Clone)]
pub struct EventEmitter<T> {
    inner: Arc<Inner<T>>,
}

impl<T> EventEmitter<T> {
    /// Create a new emitter.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                listeners: RwLock::new(HashMap::new()),
                next_id: AtomicU32::new(0),
            }),
        }
    }

    /// Subscribe to events carrying `&T`.
    /// Returns a `Subscription` which unsubscribes on drop.
    #[cfg(target_arch = "wasm32")]
    pub fn subscribe<F>(&self, listener: F) -> Subscription<T>
    where
        F: Fn(&T) + 'static,
    {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        self.inner
            .listeners
            .write()
            .unwrap()
            .insert(id, Box::new(listener));

        Subscription {
            id,
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Subscribe to events carrying `&T`.
    /// Returns a `Subscription` which unsubscribes on drop.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn subscribe<F>(&self, listener: F) -> Subscription<T>
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        self.inner
            .listeners
            .write()
            .unwrap()
            .insert(id, Box::new(listener));

        Subscription {
            id,
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Manually unsubscribe by ID. Returns `true` if removed.
    pub fn unsubscribe(&self, id: u32) -> bool {
        self.inner.listeners.write().unwrap().remove(&id).is_some()
    }

    /// Emit a payload to all current listeners.
    pub fn emit(&self, payload: &T) {
        // Call listeners directly under read lock
        // This is safe because we're not modifying the map during iteration
        let read = self.inner.listeners.read().unwrap();
        for callback in read.values() {
            callback(payload);
        }
    }
}

impl<T> Default for EventEmitter<T> {
    fn default() -> Self {
        Self::new()
    }
}
