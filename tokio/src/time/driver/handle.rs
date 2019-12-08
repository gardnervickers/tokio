use crate::runtime::context::{ThreadContext, TimerGuard};
use crate::time::driver::Inner;
use std::fmt;
use std::sync::{Arc, Weak};

/// Handle to time driver instance.
#[derive(Clone)]
pub(crate) struct Handle {
    inner: Weak<Inner>,
}

/// Sets handle to default timer, returning guard that unsets it on drop.
///
/// # Panics
///
/// This function panics if there already is a default timer set.
pub(crate) fn set_default(handle: &Handle) -> TimerGuard {
    ThreadContext::set_default_timer(handle.clone())
}

impl Handle {
    /// Create a new timer `Handle` from a shared `Inner` timer state.
    pub(crate) fn new(inner: Weak<Inner>) -> Self {
        Handle { inner }
    }

    /// Try to get a handle to the current timer.
    ///
    /// # Panics
    ///
    /// This function panics if there is no current timer set.
    pub(crate) fn current() -> Self {
        ThreadContext::with_timer(|timer| match timer {
            Some(ref handle) => handle.clone(),
            None => panic!("no current timer"),
        })
    }

    /// Try to return a strong ref to the inner
    pub(crate) fn inner(&self) -> Option<Arc<Inner>> {
        self.inner.upgrade()
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Handle")
    }
}
