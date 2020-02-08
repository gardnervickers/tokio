//! The syscalls package defines interaction points for plugging in
//! alternative implementations of syscall related functionality, such as opening
//! sockets. "syscalls" is used pretty liberally here to mean "interacting with the runtime".
//!
//! One goal for syscalls is to support simulation for example.
use core::task::Poll;
use std::task::Context;
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
pub trait Syscalls {
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

    /// Write bytes to the logical TcpStream associated with the provided [TcpStreamIdentifier]
    ///
    /// [TcpStreamIdentifier]: struct.TcpStreamIdentifier.html
    fn poll_stream_write(
        &self,
        cx: &mut Context<'_>,
        ident: TcpStreamIdentifier,
        buf: &[u8],
    ) -> Poll<io::Result<usize>>;
}
