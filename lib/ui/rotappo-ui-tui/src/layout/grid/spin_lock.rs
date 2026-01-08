//! Simple spin lock used for lightweight shared state.

use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};

/// Mutex wrapper that spins until the lock is available.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::layout::SpinLock;
///
/// let lock = SpinLock::new(1);
/// {
///     let mut guard = lock.lock();
///     *guard = 2;
/// }
/// assert_eq!(*lock.lock(), 2);
/// ```
pub struct SpinLock<T> {
    inner: Mutex<T>,
}

impl<T> SpinLock<T> {
    /// Create a new spin lock with the provided value.
    pub fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
        }
    }

    /// Acquire the lock, spinning if another thread holds it.
    pub fn lock(&self) -> SpinGuard<'_, T> {
        loop {
            if let Ok(guard) = self.inner.try_lock() {
                return SpinGuard { guard };
            }
            std::hint::spin_loop();
        }
    }
}

/// Guard type for `SpinLock`.
pub struct SpinGuard<'a, T> {
    guard: MutexGuard<'a, T>,
}

impl<T> Deref for SpinGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T> DerefMut for SpinGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}
