//! The syscalls package defines interaction points for plugging in
//! alternative implementations of syscall related functionality, such as opening
//! sockets. "syscalls" is used pretty liberally here to mean "interacting with the runtime".
//!
//! One goal for syscalls is to support simulation for example.
use core::task::Poll;
use std::task::Context;
use std::time;
use std::{io, net, vec};

/// A value which can be resolved to a SocketAddr. Usually, these come
/// in two forms. Either a hostname/port string like "localhost:8080", or
/// a hostname port tuple like ("localhost", 8080).
#[derive(Debug)]
pub enum Resolveable<'a> {
    /// String variant of the form "host:port".
    Str(&'a str),
    /// Tuple variant of the form ("host", port).
    Tuple(&'a (&'a str, u16)),
}

impl<'a> From<&'a str> for Resolveable<'a> {
    fn from(s: &'a str) -> Self {
        Resolveable::Str(s)
    }
}

impl<'a> From<&'a (&'a str, u16)> for Resolveable<'a> {
    fn from(t: &'a (&'a str, u16)) -> Self {
        Resolveable::Tuple(t)
    }
}

/// TcpStreamIdentifier represents a TCP stream with respect to the Syscall trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcpStreamIdentifier(u64);

/// TcpListenerIdentifier represents a TCP listener with respect to the Syscall trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcpListenerIdentifier(u64);

/// Extension points for Tokio.
pub trait Syscalls: TcpSyscalls + Send + Sync + std::fmt::Debug {
    /// Resolve the provided [Resolveable] to one or more [net::SocketAddr] entries.
    ///
    /// [Resolveable]: struct.Resolveable.html
    /// [net::SocketAddr]: std::net::SocketAddr
    fn resolve(&self, addr: Resolveable<'_>) -> io::Result<vec::IntoIter<net::SocketAddr>>;
}

/// Tcp* syscalls
pub trait TcpSyscalls {
    /// Connect to the logical TcpStream associated with the provided [net::SocketAddr].
    ///
    /// [net::SocketAddr]: std::net::SocketAddr
    fn connect_addr(&self, addr: net::SocketAddr) -> io::Result<TcpStreamIdentifier>;

    /// Poll the provided [TcpStreamIdentifier] until it is connected.
    ///
    /// If the stream represented by the [TcpStreamIdentifier] is not ready, `Poll::Pending` is returned and
    /// the current task will be notified once a new event is received.
    ///
    /// [TcpStreamIdentifer]: struct.TcpStreamIdentifier.html
    fn poll_connected(
        &self,
        cx: &mut Context<'_>,
        ident: &TcpStreamIdentifier,
    ) -> Poll<io::Result<()>>;

    /// Write bytes to the logical TcpStream associated with the provided [TcpStreamIdentifier]
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn poll_stream_write(
        &self,
        cx: &mut Context<'_>,
        ident: &TcpStreamIdentifier,
        buf: &[u8],
    ) -> Poll<io::Result<usize>>;

    /// Lookup the [net::SocketAddr] corresponding ot the provided [TcpStreamIdentifier]
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    /// [net::SocketAddr]: std::net::SocketAddr
    fn stream_local_addr(&self, ident: &TcpStreamIdentifier) -> io::Result<net::SocketAddr>;

    /// Lookup the [net::SocketAddr] corresponding to the provided [TcpStreamIdentifier]
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    /// [net::SocketAddr]: std::net::SocketAddr
    fn stream_peer_addr(&self, ident: &TcpStreamIdentifier) -> io::Result<net::SocketAddr>;

    /// Attempts to receive data from the logical stream without removing data from the queue.
    fn poll_peek(
        &self,
        cx: &mut Context<'_>,
        ident: &TcpStreamIdentifier,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>>;

    /// Shuts down the read/write or both halves of the connections represented by the provided [TcpStreamIdentifer].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn shutdown(&self, ident: &TcpStreamIdentifier, how: net::Shutdown) -> io::Result<()>;

    /// Gets the value of the `TCP_NODELAY` option on this [TcpStreamIdentifier].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn nodelay(&self, ident: &TcpStreamIdentifier) -> io::Result<bool>;

    /// Sets the value of the `TCP_NODELAY` option on this [TcpStreamIdentifier].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn set_nodelay(&self, ident: &TcpStreamIdentifier, nodelay: bool) -> io::Result<()>;

    /// Gets the receive buffer size for this [TcpStreamIdentifier].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn recv_buffer_size(&self, ident: &TcpStreamIdentifier) -> io::Result<usize>;

    /// Sets the receive buffer size for this [TcpStreamIdentifier].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn set_recv_buffer_size(&self, ident: &TcpStreamIdentifier, size: usize) -> io::Result<()>;

    /// Gets the send buffer size for this [TcpStreamIdentifier].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn send_buffer_size(&self, ident: &TcpStreamIdentifier) -> io::Result<usize>;

    /// Sets the send buffer size for this [TcpStreamIdentifier].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn set_send_buffer_size(&self, ident: &TcpStreamIdentifier, size: usize) -> io::Result<()>;

    /// Get the keepalive value for this [TcpStreamIdentifier].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn keepalive(&self, ident: &TcpStreamIdentifier) -> io::Result<Option<time::Duration>>;

    /// Set the keepalive value for this [TcpStreamIdentifer].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn set_keepalive(
        &self,
        ident: &TcpStreamIdentifier,
        keepalive: Option<time::Duration>,
    ) -> io::Result<()>;

    /// Get the ttl value for this [TcpStreamIdentifier].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn stream_ttl(&self, ident: &TcpStreamIdentifier) -> io::Result<u32>;

    /// Set the ttl value for this [TcpStreamIdentifer].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn stream_set_ttl(&self, ident: &TcpStreamIdentifier, ttl: u32) -> io::Result<()>;

    /// Get the linger value for this [TcpStreamIdentifier].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn linger(&self, ident: &TcpStreamIdentifier) -> io::Result<Option<time::Duration>>;

    /// Set the linger value for this [TcpStreamIdentifer].
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn set_linger(
        &self,
        ident: &TcpStreamIdentifier,
        linger: Option<time::Duration>,
    ) -> io::Result<()>;

    /// TODO dox
    fn poll_read_priv(
        &self,
        cx: &mut Context<'_>,
        ident: &TcpStreamIdentifier,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>>;

    /// TODO dox
    fn poll_write_priv(
        &self,
        cx: &mut Context<'_>,
        ident: &TcpStreamIdentifier,
        buf: &[u8],
    ) -> Poll<io::Result<usize>>;

    /// Lookup the [net::SocketAddr] corresponding ot the provided [TcpListenerIdentifier]
    ///
    /// [TcpListenerIdentifier]: struct.TcpStreamIdentifier.html
    /// [net::SocketAddr]: std::net::SocketAddr
    fn listener_local_addr(&self, ident: &TcpListenerIdentifier) -> io::Result<net::SocketAddr>;

    /// Get the ttl value for this [TcpListenerIdentifier].
    ///
    /// [TcpListenerIdentifier]: struct.TcpListenerIdentifier.html
    fn listener_ttl(&self, ident: &TcpListenerIdentifier) -> io::Result<u32>;

    /// Set the ttl value for this [TcpListenerIdentifer].
    ///
    /// [TcpListenerIdentifier]: struct.TcpListenerIdentifier.html
    fn listener_set_ttl(&self, ident: &TcpListenerIdentifier, ttl: u32) -> io::Result<()>;

    /// Attempts to poll `SocketAddr` and `TcpStream` bound to this [TcpListenerIdentifier].
    ///
    /// [TcpListenerIdentifier]: struct.TcpListenerIdentifier.html
    fn poll_accept(
        &self,
        cx: &mut Context<'_>,
        ident: &TcpListenerIdentifier,
    ) -> Poll<io::Result<(TcpStreamIdentifier, net::SocketAddr)>>;
}
