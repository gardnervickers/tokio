use crate::runtime::basic_scheduler;
use crate::runtime::context::ThreadContext;
use crate::task::JoinHandle;
use std::future::Future;

#[derive(Debug, Clone, Copy)]
pub(super) enum State {
    // default executor not defined
    Empty,

    // Basic scheduler (runs on the current-thread)
    Basic(*const basic_scheduler::SchedulerPriv),

    // default executor is a thread pool instance.
    #[cfg(feature = "rt-threaded")]
    ThreadPool(*const crate::runtime::thread_pool::Spawner),
}

// ===== global spawn fns =====

/// Spawns a future on the default executor.
pub(crate) fn spawn<T>(future: T) -> JoinHandle<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    ThreadContext::with_executor(|state| {
        match *state {
            #[cfg(feature = "rt-threaded")]
            State::ThreadPool(thread_pool_ptr) => {
                let thread_pool = unsafe { &*thread_pool_ptr };
                thread_pool.spawn(future)
            }
            State::Basic(basic_scheduler_ptr) => {
                let basic_scheduler = unsafe { &*basic_scheduler_ptr };

                // Safety: The `BasicScheduler` value set the thread-local (same
                // thread).
                unsafe { basic_scheduler.spawn(future) }
            }
            State::Empty => {
                // Explicit drop of `future` silences the warning that `future` is
                // not used when neither rt-* feature flags are enabled.
                drop(future);
                panic!("must be called from the context of Tokio runtime configured with either `basic_scheduler` or `threaded_scheduler`");
            }
        }
    })
}
