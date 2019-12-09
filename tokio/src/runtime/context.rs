//! Global runtime context
#![allow(dead_code)]
use crate::park::ParkThread;
cfg_io_driver! {
    use crate::io::driver::Handle as ReactorHandle;
}
cfg_rt_core! {
    use crate::runtime::basic_scheduler::SchedulerPriv;
}
cfg_blocking_impl! {
    use crate::runtime::blocking::Spawner;
}
cfg_rt_core! {
    use crate::runtime::global::State as ExecutorState;
}

#[cfg(all(feature = "time", not(loom)))]
use crate::time::driver::Handle as TimerHandle;
#[cfg(all(feature = "time", not(loom)))]
use crate::time::Clock;

use std::cell::RefCell;

thread_local! {
    static CONTEXT: RefCell<ThreadContext> = RefCell::new(ThreadContext::new())
}

#[derive(Clone)]
pub(crate) struct ThreadContext {
    entered: bool,
    parker: ParkThread,
    #[cfg(feature = "io-driver")]
    reactor: Option<ReactorHandle>,
    #[cfg(feature = "rt-core")]
    executor: ExecutorState,
    #[cfg(feature = "rt-core")]
    scheduler: Option<*const SchedulerPriv>,
    #[cfg(all(feature = "time", not(loom)))]
    timer: Option<TimerHandle>,
    #[cfg(all(feature = "time", not(loom)))]
    clock: Option<*const Clock>,
    #[cfg(any( // cfg_blocking_impl features
        feature = "blocking",
        feature = "fs",
        feature = "dns",
        feature = "io-std",
        feature = "rt-threaded",
    ))]
    blocking_pool: Option<*const Spawner>,
}

impl ThreadContext {
    fn new() -> Self {
        ThreadContext {
            entered: false,
            parker: ParkThread::new(),
            #[cfg(feature = "io-driver")]
            reactor: None,
            #[cfg(feature = "rt-core")]
            executor: ExecutorState::Empty,
            #[cfg(feature = "rt-core")]
            scheduler: None,
            #[cfg(all(feature = "time", not(loom)))]
            timer: None,
            #[cfg(all(feature = "time", not(loom)))]
            clock: None,
            #[cfg(any( // cfg_blocking_impl features
                feature = "blocking",
                feature = "fs",
                feature = "dns",
                feature = "io-std",
                feature = "rt-threaded",
            ))]
            blocking_pool: None,
        }
    }

    pub(super) fn set_entered(entered: bool) {
        CONTEXT.with(|ctx| {
            ctx.borrow_mut().entered = entered;
        })
    }

    pub(super) fn entered() -> bool {
        CONTEXT.with(|ctx| ctx.borrow().entered)
    }

    #[cfg(all(feature = "time", not(loom)))]
    pub(crate) fn set_default_clock(clock: *const Clock) -> ClockGuard {
        CONTEXT.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            let was = ctx.clock.replace(clock);
            ClockGuard(was)
        })
    }

    #[cfg(all(feature = "time", not(loom)))]
    pub(crate) fn with_clock<F, U>(f: F) -> U
    where
        F: FnOnce(&Option<*const Clock>) -> U,
    {
        CONTEXT.with(|ctx| f(&ctx.borrow().clock))
    }

    #[cfg(feature = "rt-core")]
    pub(super) fn set_default_executor(state: ExecutorState) -> ExecutorGuard {
        CONTEXT.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            let was = ::std::mem::replace(&mut ctx.executor, state);
            ExecutorGuard(was)
        })
    }

    #[cfg(feature = "rt-core")]
    pub(super) fn with_executor<F, U>(f: F) -> U
    where
        F: FnOnce(&ExecutorState) -> U,
    {
        CONTEXT.with(|ctx| f(&ctx.borrow().executor))
    }

    #[cfg(feature = "io-driver")]
    pub(crate) fn set_default_reactor(handle: ReactorHandle) -> ReactorGuard {
        CONTEXT.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            let was = ctx.reactor.replace(handle);
            ReactorGuard(was)
        })
    }

    #[cfg(feature = "io-driver")]
    pub(crate) fn with_reactor<F, U>(f: F) -> U
    where
        F: FnOnce(&Option<ReactorHandle>) -> U,
    {
        CONTEXT.with(|ctx| f(&ctx.borrow().reactor))
    }

    pub(crate) fn with_parker<F, U>(f: F) -> U
    where
        F: FnOnce(&ParkThread) -> U,
    {
        CONTEXT.with(|ctx| f(&ctx.borrow().parker))
    }

    #[cfg(feature = "rt-core")]
    pub(super) fn set_default_scheduler(scheduler: *const SchedulerPriv) -> SchedulerGuard {
        CONTEXT.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            let was = ctx.scheduler.replace(scheduler);
            SchedulerGuard(was)
        })
    }

    #[cfg(feature = "rt-core")]
    pub(super) fn with_scheduler<F, U>(f: F) -> U
    where
        F: FnOnce(&Option<*const SchedulerPriv>) -> U,
    {
        CONTEXT.with(|ctx| f(&ctx.borrow().scheduler))
    }

    #[cfg(all(feature = "time", not(loom)))]
    pub(crate) fn set_default_timer(handle: TimerHandle) -> TimerGuard {
        CONTEXT.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            let was = ctx.timer.replace(handle);
            TimerGuard(was)
        })
    }

    #[cfg(all(feature = "time", not(loom)))]
    pub(crate) fn with_timer<F, U>(f: F) -> U
    where
        F: FnOnce(&Option<TimerHandle>) -> U,
    {
        CONTEXT.with(|ctx| f(&ctx.borrow().timer))
    }

    #[cfg(any(
        feature = "blocking",
        feature = "fs",
        feature = "dns",
        feature = "io-std",
        feature = "rt-threaded",
    ))]
    pub(super) fn set_default_blocking_pool(blocking_pool: *const Spawner) -> BlockingPoolGuard {
        CONTEXT.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            let was = ctx.blocking_pool.replace(blocking_pool);
            BlockingPoolGuard(was)
        })
    }

    #[cfg(any(
        feature = "blocking",
        feature = "fs",
        feature = "dns",
        feature = "io-std",
        feature = "rt-threaded",
    ))]
    pub(super) fn with_blocking_pool<F, U>(f: F) -> U
    where
        F: FnOnce(&Option<*const Spawner>) -> U,
    {
        CONTEXT.with(|ctx| f(&ctx.borrow().blocking_pool))
    }
}

cfg_rt_core! {
    #[derive(Debug)]
    /// Guard that resets the current executor on drop.
    pub(super) struct ExecutorGuard(ExecutorState);
    impl Drop for ExecutorGuard {
        fn drop(&mut self) {
            let was = ::std::mem::replace(&mut self.0, ExecutorState::Empty);
            CONTEXT.with(|ctx| {
                ctx.borrow_mut().executor = was;
            })
        }
    }
}

cfg_io_driver! {
    #[derive(Debug)]
    /// Guard that resets the current reactor on drop.
    pub(crate) struct ReactorGuard(Option<ReactorHandle>);
    impl Drop for ReactorGuard {
        fn drop(&mut self) {
            CONTEXT.with(|ctx| {
                if let Some(reactor) = self.0.take() {
                    ctx.borrow_mut().reactor.replace(reactor);
                } else {
                    ctx.borrow_mut().reactor.take();
                }
            })
        }
    }
}

cfg_rt_core! {
    #[derive(Debug)]
    /// Guard that resets the current scheduler on drop.
    pub(crate) struct SchedulerGuard(Option<*const SchedulerPriv>);
    impl Drop for SchedulerGuard {
        fn drop(&mut self) {
            CONTEXT.with(|ctx| {
                if let Some(scheduler) = self.0.take() {
                    ctx.borrow_mut().scheduler.replace(scheduler);
                } else {
                    ctx.borrow_mut().scheduler.take();
                }
            })
        }
    }
}

#[cfg(all(feature = "time", not(loom)))]
#[derive(Debug)]
/// Guard that resets the current clock on drop.
pub(crate) struct ClockGuard(Option<*const Clock>);
#[cfg(all(feature = "time", not(loom)))]
impl Drop for ClockGuard {
    fn drop(&mut self) {
        CONTEXT.with(|ctx| {
            if let Some(clock) = self.0.take() {
                ctx.borrow_mut().clock.replace(clock);
            } else {
                ctx.borrow_mut().clock.take();
            }
        })
    }
}

#[cfg(all(feature = "time", not(loom)))]
#[derive(Debug)]
/// Guard that resets the current timer handle on drop.
pub(crate) struct TimerGuard(Option<TimerHandle>);
#[cfg(all(feature = "time", not(loom)))]
impl Drop for TimerGuard {
    fn drop(&mut self) {
        CONTEXT.with(|ctx| {
            if let Some(timer) = self.0.take() {
                ctx.borrow_mut().timer.replace(timer);
            } else {
                ctx.borrow_mut().timer.take();
            }
        })
    }
}

cfg_blocking_impl! {
    #[derive(Debug)]
    /// Guard that resets the current blocking pool spawner on drop.
    pub(crate) struct BlockingPoolGuard(Option<*const Spawner>);
    impl Drop for BlockingPoolGuard {
        fn drop(&mut self) {
            CONTEXT.with(|ctx| {
                if let Some(spawner) = self.0.take() {
                    ctx.borrow_mut().blocking_pool.replace(spawner);
                } else {
                    ctx.borrow_mut().blocking_pool.take();
                }
            })
        }
    }
}
