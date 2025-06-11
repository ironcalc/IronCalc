use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, RwLock, Weak,
    },
};

// Type alias to reduce complexity
#[cfg(any(target_arch = "wasm32", feature = "single_threaded"))]
type ListenerMap<T> = HashMap<u32, Box<dyn Fn(&T) + 'static>>;
#[cfg(not(any(target_arch = "wasm32", feature = "single_threaded")))]
type ListenerMap<T> = HashMap<u32, Box<dyn Fn(&T) + Send + Sync + 'static>>;

/// RAII handle: dropping it (or calling `.unsubscribe()`) removes the listener.
pub struct Subscription<T> {
    id: u32,
    inner: Weak<Inner<T>>,
}

impl<T> Subscription<T> {
    /// explicitly unsubscribe early
    pub fn unsubscribe(self) {
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut listeners) = inner.listeners.write() {
                listeners.remove(&self.id);
            }
        }
    }
}

impl<T> Drop for Subscription<T> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut listeners) = inner.listeners.write() {
                listeners.remove(&self.id);
            }
        }
    }
}

struct Inner<T> {
    listeners: RwLock<ListenerMap<T>>,
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
    #[cfg(any(target_arch = "wasm32", feature = "single_threaded"))]
    pub fn subscribe<F>(&self, listener: F) -> Subscription<T>
    where
        F: Fn(&T) + 'static,
    {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut listeners) = self.inner.listeners.write() {
            listeners.insert(id, Box::new(listener));
        }

        Subscription {
            id,
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Subscribe to events carrying `&T`.
    /// Returns a `Subscription` which unsubscribes on drop.
    #[cfg(not(any(target_arch = "wasm32", feature = "single_threaded")))]
    pub fn subscribe<F>(&self, listener: F) -> Subscription<T>
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut listeners) = self.inner.listeners.write() {
            listeners.insert(id, Box::new(listener));
        }

        Subscription {
            id,
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Manually unsubscribe by ID. Returns `true` if removed.
    pub fn unsubscribe(&self, id: u32) -> bool {
        if let Ok(mut listeners) = self.inner.listeners.write() {
            listeners.remove(&id).is_some()
        } else {
            false
        }
    }

    /// Emit a payload to all current listeners.
    pub fn emit(&self, payload: &T) {
        // Call listeners directly under read lock
        // This is safe because we're not modifying the map during iteration
        if let Ok(read) = self.inner.listeners.read() {
            for callback in read.values() {
                callback(payload);
            }
        }
    }
}

impl<T> Default for EventEmitter<T> {
    fn default() -> Self {
        Self::new()
    }
}
