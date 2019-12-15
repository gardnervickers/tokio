//! The Tokio runtime.
//!
//! Unlike other Rust programs, asynchronous applications require
//! runtime support. In particular, the following runtime services are
//! necessary:
//!
//! * An **I/O event loop**, called the [driver], which drives I/O resources and
//!   dispatches I/O events to tasks that depend on them.
//! * A **scheduler** to execute [tasks] that use these I/O resources.
//! * A **timer** for scheduling work to run after a set period of time.
//!
//! Tokio's [`Runtime`] bundles all of these services as a single type, allowing
//! them to be started, shut down, and configured together. However, most
//! applications won't need to use [`Runtime`] directly. Instead, they can
//! use the [`tokio::main`] attribute macro, which creates a [`Runtime`] under
//! the hood.
//!
//! # Usage
//!
//! Most applications will use the [`tokio::main`] attribute macro.
//!
//! ```no_run
//! use tokio::net::TcpListener;
//! use tokio::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut listener = TcpListener::bind("127.0.0.1:8080").await?;
//!
//!     loop {
//!         let (mut socket, _) = listener.accept().await?;
//!
//!         tokio::spawn(async move {
//!             let mut buf = [0; 1024];
//!
//!             // In a loop, read data from the socket and write the data back.
//!             loop {
//!                 let n = match socket.read(&mut buf).await {
//!                     // socket closed
//!                     Ok(n) if n == 0 => return,
//!                     Ok(n) => n,
//!                     Err(e) => {
//!                         println!("failed to read from socket; err = {:?}", e);
//!                         return;
//!                     }
//!                 };
//!
//!                 // Write the data back
//!                 if let Err(e) = socket.write_all(&buf[0..n]).await {
//!                     println!("failed to write to socket; err = {:?}", e);
//!                     return;
//!                 }
//!             }
//!         });
//!     }
//! }
//! ```
//!
//! From within the context of the runtime, additional tasks are spawned using
//! the [`tokio::spawn`] function. Futures spawned using this function will be
//! executed on the same thread pool used by the [`Runtime`].
//!
//! A [`Runtime`] instance can also be used directly.
//!
//! ```no_run
//! use tokio::net::TcpListener;
//! use tokio::prelude::*;
//! use tokio::runtime::Runtime;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create the runtime
//!     let mut rt = Runtime::new()?;
//!
//!     // Spawn the root task
//!     rt.block_on(async {
//!         let mut listener = TcpListener::bind("127.0.0.1:8080").await?;
//!
//!         loop {
//!             let (mut socket, _) = listener.accept().await?;
//!
//!             tokio::spawn(async move {
//!                 let mut buf = [0; 1024];
//!
//!                 // In a loop, read data from the socket and write the data back.
//!                 loop {
//!                     let n = match socket.read(&mut buf).await {
//!                         // socket closed
//!                         Ok(n) if n == 0 => return,
//!                         Ok(n) => n,
//!                         Err(e) => {
//!                             println!("failed to read from socket; err = {:?}", e);
//!                             return;
//!                         }
//!                     };
//!
//!                     // Write the data back
//!                     if let Err(e) = socket.write_all(&buf[0..n]).await {
//!                         println!("failed to write to socket; err = {:?}", e);
//!                         return;
//!                     }
//!                 }
//!             });
//!         }
//!     })
//! }
//! ```
//!
//! ## Runtime Configurations
//!
//! Tokio provides multiple task scheding strategies, suitable for different
//! applications. The [runtime builder] or `#[tokio::main]` attribute may be
//! used to select which scheduler to use.
//!
//! #### Basic Scheduler
//!
//! The basic scheduler provides a _single-threaded_ future executor. All tasks
//! will be created and executed on the current thread. The basic scheduler
//! requires the `rt-core` feature flag, and can be selected using the
//! [`Builder::basic_scheduler`] method:
//! ```
//! use tokio::runtime;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let basic_rt = runtime::Builder::new()
//!     .basic_scheduler()
//!     .build()?;
//! # Ok(()) }
//! ```
//!
//! If the `rt-core` feature is enabled and `rt-threaded` is not,
//! [`Runtime::new`] will return a basic scheduler runtime by default.
//!
//! #### Threaded Scheduler
//!
//! The threaded scheduler executes futures on a _thread pool_, using a
//! work-stealing strategy. By default, it will start a worker thread for each
//! CPU core available on the system. This tends to be the ideal configurations
//! for most applications. The threaded scheduler requires the `rt-threaded` feature
//! flag, and can be selected using the  [`Builder::threaded_scheduler`] method:
//! ```
//! use tokio::runtime;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let threaded_rt = runtime::Builder::new()
//!     .threaded_scheduler()
//!     .build()?;
//! # Ok(()) }
//! ```
//!
//! If the `rt-threaded` feature flag is enabled, [`Runtime::new`] will return a
//! basic scheduler runtime by default.
//!
//! Most applications should use the threaded scheduler, except in some niche
//! use-cases, such as when running only a single thread is required.
//!
//! #### Resource drivers
//!
//! When configuring a runtime by hand, no resource drivers are enabled by
//! default. In this case, attempting to use networking types or time types will
//! fail. In order to enable these types, the resource drivers must be enabled.
//! This is done with [`Builder::enable_io`] and [`Builder::enable_time`]. As a
//! shorthand, [`Builder::enable_all`] enables both resource drivers.
//!
//! ## Lifetime of spawned threads
//!
//! The runtime may spawn threads depending on its configuration and usage. The
//! threaded scheduler spawns threads to schedule tasks and calls to
//! `spawn_blocking` spawn threads to run blocking operations.
//!
//! While the `Runtime` is active, threads may shutdown after periods of being
//! idle. Once `Runtime` is dropped, all runtime threads are forcibly shutdown.
//! Any tasks that have not yet completed will be dropped.
//!
//! [tasks]: crate::task
//! [driver]: crate::io::driver
//! [executor]: https://tokio.rs/docs/internals/runtime-model/#executors
//! [`Runtime`]: struct.Runtime.html
//! [`Reactor`]: ../reactor/struct.Reactor.html
//! [`run`]: fn.run.html
//! [`tokio::spawn`]: ../executor/fn.spawn.html
//! [`tokio::main`]: ../../tokio_macros/attr.main.html
//! [runtime builder]: crate::runtime::Builder
//! [`Runtime::new`]: crate::runtime::Runtime::new
//! [`Builder::basic_scheduler`]: crate::runtime::Builder::basic_scheduler
//! [`Builder:
//! :threaded_scheduler`]: crate::runtime::Builder::threaded_scheduler
//! [`Builder::enable_io`]: crate::runtime::Builder::enable_io
//! [`Builder::enable_time`]: crate::runtime::Builder::enable_time
//! [`Builder::enable_all`]: crate::runtime::Builder::enable_all

// At the top due to macros
#[cfg(test)]
#[macro_use]
mod tests;
pub(crate) mod context;
cfg_rt_core! {
    mod basic_scheduler;
    use basic_scheduler::BasicScheduler;
}

mod blocking;
use blocking::BlockingPool;

cfg_blocking_impl! {
    pub(crate) use blocking::spawn_blocking;
}

mod builder;
pub use self::builder::Builder;

pub(crate) mod enter;
use self::enter::enter;

cfg_rt_core! {
    mod global;
    pub(crate) use global::spawn;
}

mod handle;
pub use self::handle::Handle;

mod io;

cfg_rt_threaded! {
    mod park;
    use park::{Parker, Unparker};
}

mod shell;
use self::shell::Shell;

mod spawner;
use self::spawner::Spawner;

mod time;

cfg_rt_threaded! {
    pub(crate) mod thread_pool;
    use self::thread_pool::ThreadPool;
}

cfg_rt_core! {
    use crate::task::JoinHandle;
}

use std::future::Future;

/// The Tokio runtime.
///
/// The runtime provides an I/O [driver], task scheduler, [timer], and blocking
/// pool, necessary for running asynchronous tasks.
///
/// Instances of `Runtime` can be created using [`new`] or [`Builder`]. However,
/// most users will use the `#[tokio::main]` annotation on their entry point instead.
///
/// See [module level][mod] documentation for more details.
///
/// # Shutdown
///
/// Shutting down the runtime is done by dropping the value. The current thread
/// will block until the shut down operation has completed.
///
/// * Drain any scheduled work queues.
/// * Drop any futures that have not yet completed.
/// * Drop the reactor.
///
/// Once the reactor has dropped, any outstanding I/O resources bound to
/// that reactor will no longer function. Calling any method on them will
/// result in an error.
///
/// [driver]: crate::io::driver
/// [timer]: crate::time
/// [mod]: index.html
/// [`new`]: #method.new
/// [`Builder`]: struct.Builder.html
/// [`tokio::run`]: fn.run.html
#[derive(Debug)]
pub struct Runtime {
    /// Task executor
    kind: Kind,

    /// Handle to runtime, also contains driver handles
    handle: Handle,

    /// Blocking pool handle, used to signal shutdown
    blocking_pool: BlockingPool,
}

/// The runtime executor is either a thread-pool or a current-thread executor.
#[derive(Debug)]
enum Kind {
    /// Not able to execute concurrent tasks. This variant is mostly used to get
    /// access to the driver handles.
    Shell(Shell<time::Driver>),

    /// Execute all tasks on the current-thread.
    #[cfg(feature = "rt-core")]
    Basic(BasicScheduler<time::Driver>),

    /// Execute tasks across multiple threads.
    #[cfg(feature = "rt-threaded")]
    ThreadPool(ThreadPool),
}

/// After thread starts / before thread stops
type Callback = ::std::sync::Arc<dyn Fn() + Send + Sync>;

impl Runtime {
    /// Create a new runtime instance with default configuration values.
    ///
    /// This results in a scheduler, I/O driver, and time driver being
    /// initialized. The type of scheduler used depends on what feature flags
    /// are enabled: if the `rt-threaded` feature is enabled, the [threaded
    /// scheduler] is used, while if only the `rt-core` feature is enabled, the
    /// [basic scheduler] is used instead.
    ///
    /// If the threaded cheduler is selected, it will not spawn
    /// any worker threads until it needs to, i.e. tasks are scheduled to run.
    ///
    /// Most applications will not need to call this function directly. Instead,
    /// they will use the  [`#[tokio::main]` attribute][main]. When more complex
    /// configuration is necessary, the [runtime builder] may be used.
    ///
    /// See [module level][mod] documentation for more details.
    ///
    /// # Examples
    ///
    /// Creating a new `Runtime` with default configuration values.
    ///
    /// ```
    /// use tokio::runtime::Runtime;
    ///
    /// let rt = Runtime::new()
    ///     .unwrap();
    ///
    /// // Use the runtime...
    /// ```
    ///
    /// [mod]: index.html
    /// [main]: ../../tokio_macros/attr.main.html
    /// [threaded scheduler]: index.html#threaded-scheduler
    /// [basic scheduler]: index.html#basic-scheduler
    /// [runtime builder]: crate::runtime::Builder
    pub fn new() -> io::Result<Self> {
        #[cfg(feature = "rt-threaded")]
        let ret = Builder::new().threaded_scheduler().enable_all().build();

        #[cfg(all(not(feature = "rt-threaded"), feature = "rt-core"))]
        let ret = Builder::new().basic_scheduler().enable_all().build();

        #[cfg(not(feature = "rt-core"))]
        let ret = Builder::new().enable_all().build();

        ret
    }

    /// Spawn a future onto the Tokio runtime.
    ///
    /// This spawns the given future onto the runtime's executor, usually a
    /// thread pool. The thread pool is then responsible for polling the future
    /// until it completes.
    ///
    /// See [module level][mod] documentation for more details.
    ///
    /// [mod]: index.html
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Runtime;
    ///
    /// # fn dox() {
    /// // Create the runtime
    /// let rt = Runtime::new().unwrap();
    ///
    /// // Spawn a future onto the runtime
    /// rt.spawn(async {
    ///     println!("now running on a worker thread");
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if the spawn fails. Failure occurs if the executor
    /// is currently at capacity and is unable to spawn a new future.
    #[cfg(feature = "rt-core")]
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        match &self.kind {
            Kind::Shell(_) => panic!("task execution disabled"),
            #[cfg(feature = "rt-threaded")]
            Kind::ThreadPool(exec) => exec.spawn(future),
            Kind::Basic(exec) => exec.spawn(future),
        }
    }

    /// Run a future to completion on the Tokio runtime. This is the runtime's
    /// entry point.
    ///
    /// This runs the given future on the runtime, blocking until it is
    /// complete, and yielding its resolved result. Any tasks or timers which
    /// the future spawns internally will be executed on the runtime.
    ///
    /// This method should not be called from an asynchronous context.
    ///
    /// # Panics
    ///
    /// This function panics if the executor is at capacity, if the provided
    /// future panics, or if called within an asynchronous execution context.
    pub fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        let kind = &mut self.kind;

        self.handle.enter(|| match kind {
            Kind::Shell(exec) => exec.block_on(future),
            #[cfg(feature = "rt-core")]
            Kind::Basic(exec) => exec.block_on(future),
            #[cfg(feature = "rt-threaded")]
            Kind::ThreadPool(exec) => exec.block_on(future),
        })
    }

    /// Enter the runtime context
    pub fn enter<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.handle.enter(f)
    }

    /// Return a handle to the runtime's spawner.
    ///
    /// The returned handle can be used to spawn tasks that run on this runtime.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Runtime;
    ///
    /// let rt = Runtime::new()
    ///     .unwrap();
    ///
    /// let handle = rt.handle();
    ///
    /// handle.spawn(async { println!("hello"); });
    /// ```
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}

// ============== new impls ============== //

pub(crate) enum TimeDriver<T> {
    Enabled(crate::time::driver::Driver<T>),
    Disabled(T),
}

impl<T> TimeDriver<T>
where
    T: crate::park::Park,
{
    #[cfg(all(feature = "time", not(loom)))]
    fn create_clock() -> crate::time::Clock {
        crate::time::Clock::new()
    }

    #[cfg(all(feature = "time", not(loom)))]
    fn new(enabled: bool, io_driver: T) -> Self {
        if enabled {
            let driver = crate::time::driver::Driver::new(io_driver, Self::create_clock());
            Self::Enabled(driver)
        } else {
            Self::Disabled(io_driver)
        }
    }

    fn handle(&self) -> Option<crate::time::driver::Handle> {
        match self {
            TimeDriver::Enabled(inner) => Some(inner.handle()),
            TimeDriver::Disabled(_) => None,
        }
    }
}

impl<P> crate::park::Park for TimeDriver<P>
where
    P: crate::park::Park,
{
    type Unpark = P::Unpark;
    type Error = P::Error;
    fn unpark(&self) -> Self::Unpark {
        match self {
            TimeDriver::Enabled(p) => p.unpark(),
            TimeDriver::Disabled(p) => p.unpark(),
        }
    }
    fn park(&mut self) -> Result<(), Self::Error> {
        match self {
            TimeDriver::Enabled(p) => p.park(),
            TimeDriver::Disabled(p) => p.park(),
        }
    }
    fn park_timeout(&mut self, duration: std::time::Duration) -> Result<(), Self::Error> {
        match self {
            TimeDriver::Enabled(p) => p.park_timeout(duration),
            TimeDriver::Disabled(p) => p.park_timeout(duration),
        }
    }
}

/// The driver value the runtime passes to the `timer` layer.
///
/// When the `io-driver` feature is enabled, this is the "real" I/O driver
/// backed by Mio. Without the `io-driver` feature, this is a thread parker
/// backed by a condition variable.
#[derive(Debug)]
pub(crate) enum IoDriver {
    #[cfg(all(feature = "io-driver", not(loom)))]
    Enabled(crate::io::driver::Driver),
    Disabled(crate::park::ParkThread),
}

impl IoDriver {
    #[cfg(all(feature = "io-driver", not(loom)))]
    fn new(enabled: bool) -> io::Result<Self> {
        if enabled {
            let driver = crate::io::driver::Driver::new()?;
            Ok(Self::Enabled(driver))
        } else {
            let driver = crate::park::ParkThread::new();
            Ok(Self::Disabled(driver))
        }
    }
    #[cfg(any(not(feature = "io-driver"), loom))]
    fn new(_enabled: bool) -> io::Result<Self> {
        Ok(Self::Disabled(crate::park::ParkThread::new()))
    }

    fn handle(&self) -> Option<crate::io::driver::Handle> {
        match self {
            IoDriver::Enabled(inner) => Some(inner.handle()),
            IoDriver::Disabled(_) => None,
        }
    }
}

pub(crate) enum IoDriverUnpark {
    Enabled(crate::io::driver::Handle),
    Disabled(crate::park::UnparkThread),
}

impl crate::park::Unpark for IoDriverUnpark {
    fn unpark(&self) {
        match self {
            IoDriverUnpark::Enabled(p) => p.unpark(),
            IoDriverUnpark::Disabled(p) => p.unpark(),
        }
    }
}

impl crate::park::Park for IoDriver {
    type Unpark = IoDriverUnpark;
    type Error = std::io::Error;
    fn unpark(&self) -> Self::Unpark {
        match self {
            IoDriver::Enabled(p) => IoDriverUnpark::Enabled(p.unpark()),
            IoDriver::Disabled(p) => IoDriverUnpark::Disabled(p.unpark()),
        }
    }
    fn park(&mut self) -> Result<(), Self::Error> {
        match self {
            IoDriver::Enabled(p) => p.park(),
            IoDriver::Disabled(p) => p.park().map_err(Into::into),
        }
    }
    fn park_timeout(&mut self, duration: std::time::Duration) -> Result<(), Self::Error> {
        match self {
            IoDriver::Enabled(p) => p.park_timeout(duration),
            IoDriver::Disabled(p) => p.park_timeout(duration).map_err(Into::into),
        }
    }
}

pub(crate) enum Kind2 {
    Shell,
    Basic,
    ThreadPool,
}

/// The runtime executor is either a thread-pool or a current-thread executor.
#[derive(Debug)]
enum Kind3<P>
where
    P: crate::park::Park,
{
    /// Not able to execute concurrent tasks. This variant is mostly used to get
    /// access to the driver handles.
    Shell(Shell<P>),

    /// Execute all tasks on the current-thread.
    #[cfg(feature = "rt-core")]
    Basic(BasicScheduler<P>),

    /// Execute tasks across multiple threads.
    #[cfg(feature = "rt-threaded")]
    ThreadPool(ThreadPool),
}

struct Runtime2<P>
where
    P: crate::park::Park,
{
    runtime: Kind3<P>,
}

impl Runtime2<TimeDriver<IoDriver>> {
    pub(crate) fn new(
        kind: Kind2,
        enable_io_driver: bool,
        enable_timer_driver: bool,
    ) -> io::Result<Self> {
        let io_driver = IoDriver::new(enable_io_driver)?;
        let io_driver_handle = io_driver.handle();
        let time_driver = TimeDriver::new(enable_timer_driver, io_driver);
        let time_driver_handle = time_driver.handle();
        let runtime = match kind {
            Kind2::Shell => Kind3::Shell(Shell::new(time_driver)),
            Kind2::Basic => Kind3::Basic(BasicScheduler::new(time_driver)),
            Kind2::ThreadPool => unimplemented!(),
        };

        Ok(Self { runtime })
    }

    fn block_on<F>(&mut self, future: F) -> F::Output
    where
        F: Future,
    {
        match self.runtime {
            Kind3::Shell(ref mut rt) => rt.block_on(future),
            Kind3::Basic(ref mut rt) => rt.block_on(future),
            Kind3::ThreadPool(ref mut rt) => rt.block_on(future),
        }
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn smoke_test() {
        let mut runtime = super::Runtime2::new(Kind2::Shell, false, false).unwrap();
        runtime.block_on(async {
            println!("HI");
        });
    }
}
