pub use std::net::Shutdown;
use std::{
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd},
    time::Duration,
};
pub mod poll;
pub mod socket;
#[cfg(feature = "wasi_poll")]
pub mod wasi_poll;
#[cfg(not(feature = "wasi_poll"))]
mod wasi_poll;
#[cfg(feature = "addrinfo")]
pub use crate::socket::lookup_host;

pub(crate) mod syscall {
    macro_rules! syscall {
        ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
            #[allow(unused_unsafe)]
            let res = unsafe { libc::$fn($($arg, )*) };
            if res == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(res)
            }
        }};
    }

    #[allow(unused)]
    macro_rules! mysyscall {
        ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
            let res = unsafe { $fn($($arg, )*) };
            if res == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(res)
            }
        }};
    }
    #[allow(unused)]
    pub(crate) use mysyscall;
    pub(crate) use syscall;
}

#[allow(unused)]
use syscall::syscall;

#[derive(Debug)]
pub struct TcpStream {
    s: socket::Socket,
}

impl AsRef<socket::Socket> for TcpStream {
    fn as_ref(&self) -> &socket::Socket {
        &self.s
    }
}

impl AsMut<socket::Socket> for TcpStream {
    fn as_mut(&mut self) -> &mut socket::Socket {
        &mut self.s
    }
}

impl AsFd for TcpStream {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[derive(Debug)]
pub struct TcpListener {
    s: socket::Socket,
    pub address: std::io::Result<SocketAddr>,
    pub port: Option<u16>,
}

impl AsRef<socket::Socket> for TcpListener {
    fn as_ref(&self) -> &socket::Socket {
        &self.s
    }
}

impl AsMut<socket::Socket> for TcpListener {
    fn as_mut(&mut self) -> &mut socket::Socket {
        &mut self.s
    }
}

impl AsFd for TcpListener {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(feature = "udp")]
pub mod udp {
    use super::*;

    #[derive(Debug)]
    pub struct UdpSocket {
        s: socket::Socket,
    }

    impl AsRef<socket::Socket> for UdpSocket {
        fn as_ref(&self) -> &socket::Socket {
            &self.s
        }
    }

    impl AsMut<socket::Socket> for UdpSocket {
        fn as_mut(&mut self) -> &mut socket::Socket {
            &mut self.s
        }
    }

    impl AsFd for UdpSocket {
        fn as_fd(&self) -> BorrowedFd<'_> {
            unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
        }
    }

    impl UdpSocket {
        /// Create UDP socket and bind to the given address.
        ///
        /// If multiple address is given, the first successful socket is
        /// returned.
        pub fn bind<A: ToSocketAddrs>(addrs: A) -> io::Result<UdpSocket> {
            let mut last_error = io::Error::from(io::ErrorKind::Other);
            let addrs = addrs.to_socket_addrs()?;

            let bind = |addrs| {
                let addr_family = socket::AddressFamily::from(&addrs);
                let s = socket::Socket::new(addr_family, socket::SocketType::Datagram)?;
                s.bind(&addrs)?;
                Ok(UdpSocket { s })
            };

            for addr in addrs {
                match bind(addr) {
                    Ok(udp) => return Ok(udp),
                    Err(e) => last_error = e,
                }
            }

            Err(last_error)
        }

        pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
            let mut last_error = io::Error::from(io::ErrorKind::Other);

            let addrs = addr.to_socket_addrs()?;
            for addr in addrs {
                match self.s.connect(&addr) {
                    Ok(_) => return Ok(()),
                    Err(e) => last_error = e,
                }
            }

            Err(last_error)
        }

        pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
            self.s.recv(buf)
        }

        pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
            self.s.recv_from(buf)
        }

        pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> io::Result<usize> {
            let addr = addr.to_socket_addrs()?.next().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "No address.")
            })?;

            self.s.send_to(buf, &addr)
        }

        pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
            self.s.send(buf)
        }

        pub fn peek(&self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "peaking is not supported on this platform",
            ))
        }

        pub fn peek_from(&self, _buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "peaking is not supported on this platform",
            ))
        }

        /// Get local address.
        pub fn local_addr(&self) -> io::Result<SocketAddr> {
            self.s.get_local()
        }

        /// Get peer address.
        pub fn peer_addr(&self) -> io::Result<SocketAddr> {
            self.s.get_peer()
        }

        pub fn broadcast(&self) -> io::Result<bool> {
            self.s.broadcast()
        }

        pub fn keep_alive(&self) -> io::Result<bool> {
            self.s.keep_alive()
        }
        pub fn linger(&self) -> io::Result<Option<Duration>> {
            self.s.linger()
        }
        pub fn recv_buf_size(&self) -> io::Result<u32> {
            self.s.recv_buf_size()
        }
        pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
            self.s.read_timeout()
        }
        pub fn reuse_addr(&self) -> io::Result<bool> {
            self.s.reuse_addr()
        }
        pub fn reuse_port(&self) -> io::Result<bool> {
            self.s.reuse_port()
        }
        pub fn send_buf_size(&self) -> io::Result<u32> {
            self.s.send_buf_size()
        }
        pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
            self.s.write_timeout()
        }
        pub fn tcp_quick_ack(&self) -> io::Result<bool> {
            self.s.tcp_quick_ack()
        }
        pub fn multicast_loop_v4(&self) -> io::Result<bool> {
            self.s.ip_multicast_loop(false)
        }
        pub fn multicast_loop_v6(&self) -> io::Result<bool> {
            self.s.ip_multicast_loop(false)
        }
        
        pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
            self.s.ip_multicast_ttl()
        }
        pub fn multicast_ttl_v6(&self) -> io::Result<u32> {
            self.s.ip_multicast_ttl()
        }
        pub fn ttl(&self) -> io::Result<u32> {
            self.s.ip_ttl()
        }
        pub fn ipv6_only(&self) -> io::Result<bool> {
            self.s.ipv6_only()
        }
        pub fn set_broadcast(&self, opt: bool) -> io::Result<()> {
            self.s.set_broadcast(opt)
        }
        pub fn set_keep_alive(&self, opt: bool) -> io::Result<()> {
            self.s.set_keep_alive(opt)
        }
        pub fn set_linger(&self, linger: Option<Duration>) -> io::Result<()> {
            self.s.set_linger(linger)
        }
        pub fn set_recv_buf_size(&self, opt: usize) -> io::Result<()> {
            self.s.set_recv_buf_size(opt)
        }
        pub fn set_read_timeout(&self, opt: Option<Duration>) -> io::Result<()> {
            self.s.set_read_timeout(opt)
        }
        pub fn set_reuse_addr(&self, opt: bool) -> io::Result<()> {
            self.s.set_reuse_addr(opt)
        }
        pub fn set_reuse_port(&self, opt: bool) -> io::Result<()> {
            self.s.set_reuse_port(opt)
        }
        pub fn set_send_buf_size(&self, opt: usize) -> io::Result<()> {
            self.s.set_send_buf_size(opt)
        }
        pub fn set_write_timeout(&self, opt: Option<Duration>) -> io::Result<()> {
            self.s.set_write_timeout(opt)
        }
        /// technically incorrect
        pub fn set_multicast_loop_v4(&self, opt: bool) -> io::Result<()> {
            self.s.set_ip_multicast_loop(false, opt)
        }
        /// technically incorrect
        pub fn set_multicast_loop_v6(&self, opt: bool) -> io::Result<()> {
            self.s.set_ip_multicast_loop(false, opt)
        }
        /// technically incorrect
        pub fn set_multicast_ttl_v4(&self, opt: u32) -> io::Result<()> {
            self.s.set_ip_multicast_ttl(opt)
        }
        /// technically incorrect
        pub fn set_multicast_ttl_v6(&self, opt: u32) -> io::Result<()> {
            self.s.set_ip_multicast_ttl(opt)
        }
        pub fn join_multicast_v4(&self, addr: &Ipv4Addr, _interface: &Ipv4Addr) -> io::Result<()> {
            self.s.set_ip_add_membership(&IpAddr::V4(*addr), 0)
        }
        pub fn join_multicast_v6(&self, addr: &Ipv6Addr, interface: u32) -> io::Result<()> {
            self.s.set_ip_add_membership(&IpAddr::V6(*addr), interface)
        }
        pub fn leave_multicast_v4(&self, addr: &Ipv4Addr, _interface: &Ipv4Addr) -> io::Result<()> {
            self.s.set_ip_drop_membership(&IpAddr::V4(*addr), 0)
        }
        pub fn leave_multicast_v6(&self, addr: &Ipv6Addr, interface: u32) -> io::Result<()> {
            self.s.set_ip_drop_membership(&IpAddr::V6(*addr), interface)
        }
        pub fn set_ttl(&self, opt: u32) -> io::Result<()> {
            self.s.set_ip_ttl(opt)
        }
        pub fn set_ipv6_only(&self, opt: bool) -> io::Result<()> {
            self.s.set_ipv6_only(opt)
        }

        #[cfg(feature = "fake")]
        pub fn take_error(&self) -> io::Result<Option<io::Error>> {
            Ok(None)
        }
    }

    impl AsRawFd for UdpSocket {
        fn as_raw_fd(&self) -> std::os::fd::RawFd {
            self.s.as_raw_fd()
        }
    }
}

#[cfg(feature = "udp")]
pub use udp::*;

impl TcpStream {
    /// Create TCP socket and connect to the given address.
    ///
    /// If multiple address is given, the first successful socket is
    /// returned.
    pub fn connect<A: ToSocketAddrs>(addrs: A) -> io::Result<TcpStream> {
        let mut last_error = io::Error::from(io::ErrorKind::ConnectionRefused);
        let addrs = addrs.to_socket_addrs()?;

        let connect = |addrs| {
            let addr_family = socket::AddressFamily::from(&addrs);
            let s = socket::Socket::new(addr_family, socket::SocketType::Stream)?;
            s.connect(&addrs)?;
            Ok(s)
        };

        for addr in addrs {
            match connect(addr) {
                Ok(s) => return Ok(TcpStream { s }),
                Err(e) => last_error = e,
            }
        }
        Err(last_error)
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.s.shutdown(how)
    }

    /// Get peer address.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.s.get_peer()
    }

    /// Get local address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.s.get_local()
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.s.set_nonblocking(nonblocking)
    }

    pub fn new(s: socket::Socket) -> Self {
        Self { s }
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        self.s.tcp_no_delay()
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.s.set_tcp_no_delay(nodelay)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.s.ip_ttl()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.s.set_ip_ttl(ttl)
    }

    pub fn keep_alive(&self) -> io::Result<bool> {
        self.s.keep_alive()
    }
    pub fn linger(&self) -> io::Result<Option<Duration>> {
        self.s.linger()
    }
    pub fn recv_buf_size(&self) -> io::Result<u32> {
        self.s.recv_buf_size()
    }
    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.s.read_timeout()
    }
    pub fn reuse_addr(&self) -> io::Result<bool> {
        self.s.reuse_addr()
    }
    pub fn reuse_port(&self) -> io::Result<bool> {
        self.s.reuse_port()
    }
    pub fn send_buf_size(&self) -> io::Result<u32> {
        self.s.send_buf_size()
    }
    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.s.write_timeout()
    }
    pub fn tcp_fastopen_connect(&self) -> io::Result<bool> {
        self.s.tcp_fastopen_connect()
    }
    pub fn tcp_keep_idle(&self) -> io::Result<Duration> {
        self.s.tcp_keep_idle()
    }
    pub fn tcp_keep_intvl(&self) -> io::Result<Duration> {
        self.s.tcp_keep_intvl()
    }
    pub fn tcp_no_delay(&self) -> io::Result<bool> {
        self.s.tcp_no_delay()
    }
    pub fn tcp_quick_ack(&self) -> io::Result<bool> {
        self.s.tcp_quick_ack()
    }
    pub fn ip_multicast_loop(&self, ipv6: bool) -> io::Result<bool> {
        self.s.ip_multicast_loop(ipv6)
    }
    pub fn ip_multicast_ttl(&self) -> io::Result<u32> {
        self.s.ip_multicast_ttl()
    }
    pub fn ip_ttl(&self) -> io::Result<u32> {
        self.s.ip_ttl()
    }
    pub fn ipv6_only(&self) -> io::Result<bool> {
        self.s.ipv6_only()
    }
    pub fn set_broadcast(&self, opt: bool) -> io::Result<()> {
        self.s.set_broadcast(opt)
    }
    pub fn set_keep_alive(&self, opt: bool) -> io::Result<()> {
        self.s.set_keep_alive(opt)
    }
    pub fn set_linger(&self, linger: Option<Duration>) -> io::Result<()> {
        self.s.set_linger(linger)
    }
    pub fn set_recv_buf_size(&self, opt: usize) -> io::Result<()> {
        self.s.set_recv_buf_size(opt)
    }
    pub fn set_read_timeout(&self, opt: Option<Duration>) -> io::Result<()> {
        self.s.set_read_timeout(opt)
    }
    pub fn set_reuse_addr(&self, opt: bool) -> io::Result<()> {
        self.s.set_reuse_addr(opt)
    }
    pub fn set_reuse_port(&self, opt: bool) -> io::Result<()> {
        self.s.set_reuse_port(opt)
    }
    pub fn set_send_buf_size(&self, opt: usize) -> io::Result<()> {
        self.s.set_send_buf_size(opt)
    }
    pub fn set_write_timeout(&self, opt: Option<Duration>) -> io::Result<()> {
        self.s.set_write_timeout(opt)
    }
    pub fn set_tcp_fastopen_connect(&self, opt: bool) -> io::Result<()> {
        self.s.set_tcp_fastopen_connect(opt)
    }
    pub fn set_tcp_keep_idle(&self, opt: Duration) -> io::Result<()> {
        self.s.set_tcp_keep_idle(opt)
    }
    pub fn set_tcp_keep_intvl(&self, opt: Duration) -> io::Result<()> {
        self.s.set_tcp_keep_intvl(opt)
    }
    pub fn set_tcp_no_delay(&self, opt: bool) -> io::Result<()> {
        self.s.set_tcp_no_delay(opt)
    }
    pub fn set_tcp_quick_ack(&self, opt: bool) -> io::Result<()> {
        self.s.set_tcp_quick_ack(opt)
    }
    pub fn set_ip_multicast_loop(&self, ipv6: bool, opt: bool) -> io::Result<()> {
        self.s.set_ip_multicast_loop(ipv6, opt)
    }
    pub fn set_ip_multicast_ttl(&self, opt: u32) -> io::Result<()> {
        self.s.set_ip_multicast_ttl(opt)
    }
    pub fn set_ip_add_membership(&self, addr: &IpAddr, interface: u32) -> io::Result<()> {
        self.s.set_ip_add_membership(addr, interface)
    }
    pub fn set_ip_drop_membership(&self, addr: &IpAddr, interface: u32) -> io::Result<()> {
        self.s.set_ip_drop_membership(addr, interface)
    }
    pub fn set_ip_ttl(&self, opt: u32) -> io::Result<()> {
        self.s.set_ip_ttl(opt)
    }
    pub fn set_ipv6_only(&self, opt: bool) -> io::Result<()> {
        self.s.set_ipv6_only(opt)
    }
}

impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.s.as_raw_fd()
    }
}

impl IntoRawFd for TcpStream {
    fn into_raw_fd(self) -> std::os::fd::RawFd {
        self.s.into_raw_fd()
    }
}

impl FromRawFd for TcpStream {
    unsafe fn from_raw_fd(fd: std::os::fd::RawFd) -> Self {
        Self {
            s: socket::Socket::from_raw_fd(fd),
        }
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.s.recv(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.s.send(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for &TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.s.recv(buf)
    }
}

impl Write for &TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.s.send(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl From<socket::Socket> for TcpStream {
    fn from(s: socket::Socket) -> Self {
        TcpStream { s }
    }
}

impl TcpListener {
    /// Create TCP socket and bind to the given address.
    ///
    /// If multiple address is given, the first successful socket is
    /// returned.
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> io::Result<TcpListener> {
        let mut last_error = io::Error::from(io::ErrorKind::Other);
        let addrs = addrs.to_socket_addrs()?;

        let bind = |addrs| {
            let addr_family = socket::AddressFamily::from(&addrs);
            let s = socket::Socket::new(addr_family, socket::SocketType::Stream)?;
            #[cfg(feature = "opt")]
            s.set_reuse_addr(true)?;
            s.bind(&addrs)?;
            s.listen(1024)?;

            let port = addrs.port();
            Ok(TcpListener {
                s,
                address: Ok(addrs),
                port: Some(port),
            })
        };

        for addr in addrs {
            match bind(addr) {
                Ok(tcp_listener) => return Ok(tcp_listener),
                Err(e) => last_error = e,
            }
        }

        Err(last_error)
    }

    /// Accept incoming connections with given file descriptor flags.
    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let s = self.s.accept()?;
        let stream = TcpStream { s };
        let peer = stream.peer_addr()?;
        Ok((stream, peer))
    }

    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }

    /// Get local address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.s.get_local()
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        self.s.tcp_no_delay()
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.s.set_tcp_no_delay(nodelay)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.s.ip_ttl()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.s.set_ip_ttl(ttl)
    }

    #[cfg(feature = "fake")]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        Ok(None)
    }
}

impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.s.as_raw_fd()
    }
}

impl IntoRawFd for TcpListener {
    fn into_raw_fd(self) -> std::os::fd::RawFd {
        self.s.into_raw_fd()
    }
}

impl FromRawFd for TcpListener {
    unsafe fn from_raw_fd(fd: std::os::fd::RawFd) -> Self {
        let s: socket::Socket = FromRawFd::from_raw_fd(fd);
        let address = s.get_local();

        let port = address.as_ref().ok().map(|a| a.port());

        TcpListener { s, address, port }
    }
}

impl<'a> Iterator for Incoming<'a> {
    type Item = io::Result<TcpStream>;

    fn next(&mut self) -> Option<io::Result<TcpStream>> {
        Some(self.listener.accept().map(|s| s.0))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

pub struct Incoming<'a> {
    listener: &'a TcpListener,
}

/*
Implement ToScoketAddrs using nslookup, so that DNS can be resolved in wasi.
*/
pub trait ToSocketAddrs {
    type Iter: Iterator<Item = SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter>;
}

impl ToSocketAddrs for SocketAddr {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        Ok(Some(*self).into_iter())
    }
}

impl ToSocketAddrs for SocketAddrV4 {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        SocketAddr::V4(*self).to_socket_addrs()
    }
}

impl ToSocketAddrs for SocketAddrV6 {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        SocketAddr::V6(*self).to_socket_addrs()
    }
}

impl ToSocketAddrs for (IpAddr, u16) {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        match ip {
            IpAddr::V4(ref a) => (*a, port).to_socket_addrs(),
            IpAddr::V6(ref a) => (*a, port).to_socket_addrs(),
        }
    }
}

impl ToSocketAddrs for (Ipv4Addr, u16) {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV4::new(ip, port).to_socket_addrs()
    }
}

impl ToSocketAddrs for (Ipv6Addr, u16) {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV6::new(ip, port, 0, 0).to_socket_addrs()
    }
}

impl ToSocketAddrs for (&str, u16) {
    type Iter = std::vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::vec::IntoIter<SocketAddr>> {
        let (host, port) = *self;

        // try to parse the host as a regular IP address first
        if let Ok(addr) = host.parse::<std::net::Ipv4Addr>() {
            let addr = std::net::SocketAddrV4::new(addr, port);
            return Ok(vec![SocketAddr::V4(addr)].into_iter());
        }
        if let Ok(addr) = host.parse::<std::net::Ipv6Addr>() {
            let addr = std::net::SocketAddrV6::new(addr, port, 0, 0);
            return Ok(vec![SocketAddr::V6(addr)].into_iter());
        }
        #[cfg(feature = "addrinfo")]
        return Ok(Vec::from_iter(lookup_host(host, port)?).into_iter());

        #[cfg(not(feature = "addrinfo"))]
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "failed to lookup address information: Not supported on this platform",
        ));
    }
}

impl ToSocketAddrs for (String, u16) {
    type Iter = std::vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::vec::IntoIter<SocketAddr>> {
        (&*self.0, self.1).to_socket_addrs()
    }
}

// accepts strings like 'localhost:12345'
impl ToSocketAddrs for str {
    type Iter = std::vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::vec::IntoIter<SocketAddr>> {
        // try to parse as a regular SocketAddr first
        if let Ok(addr) = self.parse() {
            return Ok(vec![addr].into_iter());
        }

        if let Some((host, port)) = self.split_once(":") {
            if let Ok(port) = port.parse() {
                return (host, port).to_socket_addrs();
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid socket address",
        ))
    }
}

impl ToSocketAddrs for String {
    type Iter = std::vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<std::vec::IntoIter<SocketAddr>> {
        (**self).to_socket_addrs()
    }
}

impl<'a> ToSocketAddrs for &'a [SocketAddr] {
    type Iter = std::iter::Cloned<std::slice::Iter<'a, SocketAddr>>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(self.iter().cloned())
    }
}

impl<T: ToSocketAddrs + ?Sized> ToSocketAddrs for &T {
    type Iter = T::Iter;
    fn to_socket_addrs(&self) -> io::Result<T::Iter> {
        (**self).to_socket_addrs()
    }
}
