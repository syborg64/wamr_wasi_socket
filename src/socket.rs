use std::io;
use std::mem::MaybeUninit;
use std::net;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, RawFd};

#[cfg(feature = "opt")]
use crate::syscall::mysyscall;
use crate::syscall::syscall;

/// __wasi_address_family_t
#[derive(Copy, Clone, Debug)]
#[repr(i32)]
pub enum AddressFamily {
    Inet4 = 0,
    Inet6,
    Unspec,
}

#[allow(unreachable_patterns)]
impl From<&std::net::SocketAddr> for AddressFamily {
    fn from(addr: &std::net::SocketAddr) -> Self {
        match addr {
            std::net::SocketAddr::V4(_) => AddressFamily::Inet4,
            std::net::SocketAddr::V6(_) => AddressFamily::Inet6,
            _ => AddressFamily::Unspec,
        }
    }
}

impl AddressFamily {
    pub fn is_unspec(&self) -> bool {
        matches!(*self, AddressFamily::Unspec)
    }

    pub fn is_v4(&self) -> bool {
        matches!(*self, AddressFamily::Inet4)
    }

    pub fn is_v6(&self) -> bool {
        matches!(*self, AddressFamily::Inet6)
    }
}

/// __wasi_sock_type_t
#[derive(Copy, Clone, Debug)]
#[repr(i32, align(1))]
pub enum SocketType {
    Any = -1,
    Datagram = 0,
    Stream = 1,
}

/// __wasi_addr_type_t
#[derive(Copy, Clone, Debug)]
#[repr(i32)]
pub enum AddrType {
    IpV4 = 0,
    IpV6,
}

/// __wasi_addr_ip4_t
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Ipv4Addr {
    n0: u8,
    n1: u8,
    n2: u8,
    n3: u8,
}

/// __wasi_addr_ip4_port_t
#[derive(Clone, Copy)]
#[repr(C, align(4))]
pub struct SocketAddrV4 {
    addr: Ipv4Addr,
    port: u16,
}

/// __wasi_addr_ip6_t
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Ipv6Addr {
    n0: u16,
    n1: u16,
    n2: u16,
    n3: u16,
    h0: u16,
    h1: u16,
    h2: u16,
    h3: u16,
}

/// __wasi_addr_ip6_port_t
#[derive(Clone, Copy)]
#[repr(C, align(4))]
pub struct SocketAddrV6 {
    addr: Ipv6Addr,
    port: u16,
}

/// __wasi_addr_t {
///     i32: kind;
///     addr: union { ip4: __wasi_addr_ip4_port_t, ip6: __wasi_addr_ip6_port_t}
/// }
/// taged union:
#[derive(Clone, Copy)]
#[repr(C, i32)]
pub enum SocketAddr {
    V4(SocketAddrV4),
    V6(SocketAddrV6),
}

impl Default for SocketAddr {
    fn default() -> Self {
        Self::V4((&net::SocketAddrV4::new(net::Ipv4Addr::UNSPECIFIED, 0)).into())
    }
}


/// __wasi_addr_ip_t {
///     i32: kind;
///     addr: union { ip4: __wasi_addr_ip4_t, ip6: __wasi_addr_ip6_t}
/// }
/// taged union:
#[derive(Clone, Copy)]
#[repr(C, i32)]
pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

#[cfg(test)]
mod test {
    use crate::socket::{IpAddr, Ipv6Addr};

    use super::{SocketAddr, SocketAddrV6};
    use std::net;
    use std::ptr::addr_of;
    use std::str::FromStr;

    #[test]
    fn test_sockaddr_union_layout() {
        let sock4 = SocketAddr::from(&net::SocketAddr::from_str("0.0.0.0:80").unwrap());
        let SocketAddr::V4(ref v4) = sock4 else {
            panic!()
        };
        let sock6 = SocketAddr::from(&net::SocketAddr::from_str("[::1]:80").unwrap());
        let SocketAddr::V6(ref v6) = sock6 else {
            panic!()
        };
        assert_eq!(
            addr_of!(*v4) as usize,
            addr_of!(sock4) as usize + size_of::<i32>(),
            "v4 member starts after the i32 discriminant"
        );
        assert_eq!(
            addr_of!(*v6) as usize,
            addr_of!(sock6) as usize + size_of::<i32>(),
            "v6 member6 starts after the i32 discriminant"
        );
        assert_eq!(
            size_of::<i32>() + size_of::<SocketAddrV6>(),
            size_of::<SocketAddr>(),
            "total size if the discriminant + the largest member of the union"
        );
    }

        #[test]
    fn test_addr_union_layout() {
        let addr4 = IpAddr::from(&net::IpAddr::from_str("0.0.0.0").unwrap());
        let IpAddr::V4(ref v4) = addr4 else {
            panic!()
        };
        let addr6 = IpAddr::from(&net::IpAddr::from_str("::1").unwrap());
        let IpAddr::V6(ref v6) = addr6 else {
            panic!()
        };
        assert_eq!(
            addr_of!(*v4) as usize,
            addr_of!(addr4) as usize + size_of::<i32>(),
            "v4 member starts after the i32 discriminant"
        );
        assert_eq!(
            addr_of!(*v6) as usize,
            addr_of!(addr6) as usize + size_of::<i32>(),
            "v6 member6 starts after the i32 discriminant"
        );
        assert_eq!(
            size_of::<i32>() + size_of::<Ipv6Addr>(),
            size_of::<IpAddr>(),
            "total size if the discriminant + the largest member of the union"
        );
    }
}

impl SocketAddr {
    pub fn port(&self) -> u16 {
        match self {
            SocketAddr::V4(v4) => v4.port,
            SocketAddr::V6(v6) => v6.port,
        }
    }
}

impl From<&std::net::Ipv4Addr> for Ipv4Addr {
    fn from(value: &std::net::Ipv4Addr) -> Self {
        Self {
            n0: value.octets()[0],
            n1: value.octets()[1],
            n2: value.octets()[2],
            n3: value.octets()[3],
        }
    }
}

impl From<&std::net::Ipv6Addr> for Ipv6Addr {
    fn from(value: &std::net::Ipv6Addr) -> Self {
        Self {
            n0: value.segments()[0],
            n1: value.segments()[1],
            n2: value.segments()[2],
            n3: value.segments()[3],
            h0: value.segments()[4],
            h1: value.segments()[5],
            h2: value.segments()[6],
            h3: value.segments()[7],
        }
    }
}

impl From<&std::net::SocketAddrV4> for SocketAddrV4 {
    fn from(value: &std::net::SocketAddrV4) -> Self {
        Self {
            addr: value.ip().into(),
            port: value.port(),
        }
    }
}

impl From<&std::net::SocketAddrV6> for SocketAddrV6 {
    fn from(value: &std::net::SocketAddrV6) -> Self {
        Self {
            addr: value.ip().into(),
            port: value.port(),
        }
    }
}

impl From<&std::net::SocketAddr> for SocketAddr {
    fn from(value: &std::net::SocketAddr) -> Self {
        match value {
            std::net::SocketAddr::V4(v4) => Self::V4(v4.into()),
            std::net::SocketAddr::V6(v6) => Self::V6(v6.into()),
        }
    }
}

impl From<&std::net::IpAddr> for IpAddr {
    fn from(value: &std::net::IpAddr) -> Self {
        match value {
            std::net::IpAddr::V4(v4) => Self::V4(v4.into()),
            std::net::IpAddr::V6(v6) => Self::V6(v6.into()),
        }
    }
}

impl From<&Ipv4Addr> for std::net::Ipv4Addr {
    fn from(value: &Ipv4Addr) -> Self {
        Self::new(value.n0, value.n1, value.n2, value.n3)
    }
}

impl From<&SocketAddrV4> for std::net::SocketAddrV4 {
    fn from(value: &SocketAddrV4) -> Self {
        Self::new((&value.addr).into(), value.port)
    }
}

impl From<&Ipv6Addr> for std::net::Ipv6Addr {
    fn from(value: &Ipv6Addr) -> Self {
        Self::new(
            value.n0, value.n1, value.n2, value.n3, value.h0, value.h1, value.h2, value.h3,
        )
    }
}

impl From<&SocketAddrV6> for std::net::SocketAddrV6 {
    fn from(value: &SocketAddrV6) -> Self {
        match net::SocketAddr::new(net::Ipv6Addr::from(&value.addr).into(), value.port) {
            net::SocketAddr::V6(v6) => v6,
            net::SocketAddr::V4(..) => unreachable!(),
        }
    }
}

impl From<&SocketAddr> for net::SocketAddr {
    fn from(value: &SocketAddr) -> Self {
        match value {
            SocketAddr::V4(v4) => net::SocketAddr::V4(v4.into()),
            SocketAddr::V6(v6) => net::SocketAddr::V6(v6.into()),
        }
    }
}

#[cfg(feature = "addrinfo")]
pub mod addrinfo {
    use std::{ffi::c_char, vec};

    use super::*;

    /// __wasi_addr_info_t
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct AddrInfo {
        addr: SocketAddr,
        r#type: SocketType,
    }

    /// __wasi_addr_info_hints_t
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct AddrInfoHints {
        r#type: SocketType,
        family: AddressFamily,
        hints_enabled: u8,
    }

    pub struct LookupHost(vec::IntoIter<SocketAddr>);

    impl Iterator for LookupHost {
        type Item = net::SocketAddr;
        fn next(&mut self) -> Option<net::SocketAddr> {
            self.0.next().map(|s| (&s).into())
        }
    }

    pub fn lookup_host(host: &str, _port: u16) -> io::Result<LookupHost> {
        const MAX_INFOS: usize = 65536;

        let host = host.as_bytes().as_ptr() as *const c_char;

        let service = std::ptr::null();

        let hints = AddrInfoHints {
            r#type: SocketType::Any,
            family: AddressFamily::Unspec,
            hints_enabled: 1,
        };

        let mut infos_count;
        let mut infos_len;

        infos_count = 4; // seed the self-expansion

        // loop is expected to run at most twice, but a race condition could lead to re-allocating a third time etc.
        loop {
            infos_len = infos_count;
            let mut infos = Vec::with_capacity(infos_len as usize);

            let info_buf = infos.spare_capacity_mut();

            // let info_uninit = MaybeUninit::from(value)
            let res = unsafe {
                sock_addr_resolve(
                    host,
                    service,
                    &hints,
                    info_buf.as_mut_ptr() as *mut _,
                    infos_len,
                    &mut infos_count,
                )
            };

            if res != 0 {
                return Err(io::Error::from_raw_os_error(res));
            }

            if infos_len >= infos_count {
                unsafe { infos.set_len(infos_count as usize) };
                return Ok(LookupHost(infos.into_iter()));
            }

            if infos_count as usize >= MAX_INFOS {
                // sanity check against mistaken infinite growth
                unsafe { infos.set_len(infos_count.min(infos_len) as usize) };
                return Ok(LookupHost(infos.into_iter()));
            }
        }
    }
}

#[cfg(feature = "addrinfo")]
pub use addrinfo::*;

#[cfg(feature = "opt")]
pub mod opt {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    #[repr(u8, align(1))]
    pub enum SocketOptLevel {
        SolSocket = 0,
    }

    impl TryFrom<i32> for SocketOptLevel {
        type Error = io::Error;

        fn try_from(value: i32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(Self::SolSocket),
                _ => Err(io::Error::from_raw_os_error(libc::EINVAL)),
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    #[repr(u8, align(1))]
    pub enum SocketOptName {
        SoReuseaddr = 0,
        SoType = 1,
        SoError = 2,
        SoDontroute = 3,
        SoBroadcast = 4,
        SoSndbuf = 5,
        SoRcvbuf = 6,
        SoKeepalive = 7,
        SoOobinline = 8,
        SoLinger = 9,
        SoRcvlowat = 10,
        SoRcvtimeo = 11,
        SoSndtimeo = 12,
        SoAcceptconn = 13,
        SoBindToDevice = 14,
    }

    impl TryFrom<i32> for SocketOptName {
        type Error = io::Error;

        fn try_from(value: i32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(Self::SoReuseaddr),
                1 => Ok(Self::SoType),
                2 => Ok(Self::SoError),
                3 => Ok(Self::SoDontroute),
                4 => Ok(Self::SoBroadcast),
                5 => Ok(Self::SoSndbuf),
                6 => Ok(Self::SoRcvbuf),
                7 => Ok(Self::SoKeepalive),
                8 => Ok(Self::SoOobinline),
                9 => Ok(Self::SoLinger),
                10 => Ok(Self::SoRcvlowat),
                11 => Ok(Self::SoRcvtimeo),
                12 => Ok(Self::SoSndtimeo),
                13 => Ok(Self::SoAcceptconn),
                14 => Ok(Self::SoBindToDevice),

                _ => Err(io::Error::from_raw_os_error(libc::EINVAL)),
            }
        }
    }
}
#[cfg(feature = "iov")]
use libc::iovec;
#[cfg(feature = "opt")]
pub use opt::*;

fn fcntl_get(fd: RawFd, cmd: i32) -> io::Result<i32> {
    syscall!(fcntl(fd, cmd))
}

fn fcntl_add(fd: RawFd, get_cmd: i32, set_cmd: i32, flag: i32) -> io::Result<()> {
    let previous = syscall!(fcntl(fd, get_cmd))?;
    let new = previous | flag;
    if new != previous {
        syscall!(fcntl(fd, set_cmd, new)).map(|_| ())
    } else {
        // Flag was already set.
        Ok(())
    }
}

/// Remove `flag` to the current set flags of `F_GETFD`.
fn fcntl_remove(fd: RawFd, get_cmd: i32, set_cmd: i32, flag: i32) -> io::Result<()> {
    let previous = syscall!(fcntl(fd, get_cmd))?;
    let new = previous & !flag;
    if new != previous {
        syscall!(fcntl(fd, set_cmd, new)).map(|_| ())
    } else {
        // Flag was already set.
        Ok(())
    }
}

pub const FDFLAG_APPEND: u16 = 0x0001; // __FDFLAG_APPEND
pub const FDFLAG_DSYNC: u16 = 0x0002; // __FDFLAG_DSYNC
pub const FDFLAG_NONBLOCK: u16 = 0x0004; // __FDFLAG_NONBLOCK
pub const FDFLAG_RSYNC: u16 = 0x0008; // __FDFLAG_RSYNC
pub const FDFLAG_SYNC: u16 = 0x0010; // __FDFLAG_SYNC

mod wasi_sock {
    #[cfg(feature = "addrinfo")]
    use std::ffi::c_char;

    #[cfg(feature = "opt")]
    use crate::socket::IpAddr;
    #[cfg(feature = "addrinfo")]
    use crate::socket::{AddrInfo, AddrInfoHints};

    use super::SocketAddr;
    #[cfg(feature = "iov")]
    use libc::iovec;

    #[link(wasm_import_module = "wasi_snapshot_preview1")]
    extern "C" {
        pub fn sock_open(poolfd: i32, af: i32, socktype: i32, sockfd: *mut i32) -> i32;

        pub fn sock_bind(sockfd: i32, addr: *const SocketAddr) -> i32;

        pub fn sock_listen(sockfd: i32, backlog: u32) -> i32;

        pub fn sock_accept(socket: i32, flags: u16, fd_new: *mut i32) -> i32;

        pub fn sock_connect(socket: i32, addr: *const SocketAddr) -> i32;

        #[cfg(feature = "iov")]
        pub fn sock_recv(
            fd: i32,
            buf: *const iovec,
            buf_len: u32,
            flags: u16,
            recv_len: *mut u32,
            oflags: *mut i32,
        ) -> i32;

        #[cfg(feature = "iov")]
        pub fn sock_recv_from(
            fd: i32,
            buf: *const iovec,
            buf_len: u32,
            flags: u16,
            addr: *mut SocketAddr,
            recv_len: *mut u32,
        ) -> i32;

        #[cfg(feature = "iov")]
        pub fn sock_send(
            fd: i32,
            buf: *const iovec,
            buf_len: u32,
            flags: u16,
            send_len: *mut u32,
        ) -> i32;

        #[cfg(feature = "iov")]
        pub fn sock_send_to(
            fd: i32,
            buf: *const iovec,
            buf_len: u32,
            flags: u16,
            addr: *const SocketAddr,
            send_len: *mut u32,
        ) -> i32;

        pub fn sock_addr_remote(fd: i32, addr: *mut SocketAddr) -> i32;

        pub fn sock_addr_local(fd: i32, addr: *mut SocketAddr) -> i32;

        #[cfg(feature = "addrinfo")]
        /// this is a terrible API
        ///
        /// * `info_buf` is an array for info structs
        ///
        /// * `info_len` is the length of that array in units of info struct
        ///
        /// * `info_count` is the number of elements available. If more than info_len, need to call again with more memory
        ///
        /// @param info_len
        pub fn sock_addr_resolve(
            host: *const c_char,
            service: *const c_char,
            hints: *const AddrInfoHints,
            info_buf: *mut AddrInfo,
            infos_len: u32,
            infos_count: *mut u32,
        ) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_broadcast(fd: i32, opt: *mut bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_keep_alive(fd: i32, opt: *mut bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_linger(fd: i32, is_linger: *mut bool, linger_s: *mut i32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_recv_buf_size(fd: i32, opt: *mut u32) -> i32;

        /// rare u64 !
        /// this actually differs from the libc timeval struct
        /// uses microseconds directly
        #[cfg(feature = "opt")]
        pub fn sock_get_recv_timeout(fd: i32, timeouts_us: *mut u64) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_reuse_addr(fd: i32, opt: *mut bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_reuse_port(fd: i32, opt: *mut bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_send_buf_size(fd: i32, opt: *mut u32) -> i32;

        /// rare u64 !
        /// this actually differs from the libc timeval struct
        /// uses microseconds directly
        #[cfg(feature = "opt")]
        pub fn sock_get_send_timeout(fd: i32, timeout_us: *mut u64) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_tcp_fastopen_connect(fd: i32, opt: *mut bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_tcp_keep_idle(fd: i32, opt: *mut u32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_tcp_keep_intvl(fd: i32, opt: *mut u32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_tcp_no_delay(fd: i32, opt: *mut bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_tcp_quick_ack(fd: i32, opt: *mut bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_ip_multicast_loop(fd: i32, is_ipv6: bool, opt: *mut bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_ip_multicast_ttl(fd: i32, ttl: *mut u32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_ip_ttl(fd: i32, ttl: *mut u32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_get_ipv6_only(fd: i32, opt: *mut bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_broadcast(fd: i32, opt: bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_keep_alive(fd: i32, opt: bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_linger(fd: i32, is_linger: bool, linger_s: i32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_recv_buf_size(fd: i32, opt: u32) -> i32;

        /// rare u64 !
        /// this actually differs from the libc timeval struct
        /// uses microseconds directly
        #[cfg(feature = "opt")]
        pub fn sock_set_recv_timeout(fd: i32, timeout_us: u64) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_reuse_addr(fd: i32, opt: bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_reuse_port(fd: i32, opt: bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_send_buf_size(fd: i32, opt: u32) -> i32;

        /// rare u64 !
        /// this actually differs from the libc timeval struct
        /// uses microseconds directly
        #[cfg(feature = "opt")]
        pub fn sock_set_send_timeout(fd: i32, timeout_us: u64) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_tcp_fastopen_connect(fd: i32, opt: bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_tcp_keep_idle(fd: i32, opt: u32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_tcp_keep_intvl(fd: i32, opt: u32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_tcp_no_delay(fd: i32, opt: bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_tcp_quick_ack(fd: i32, opt: bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_ip_multicast_loop(fd: i32, ipv6: bool, opt: bool) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_ip_multicast_ttl(fd: i32, opt: u32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_ip_add_membership(fd: i32, addr: *const IpAddr, interface: u32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_ip_drop_membership(fd: i32, addr: *const IpAddr, interface: u32)
            -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_ip_ttl(fd: i32, opt: u32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_ipv6_only(fd: i32, opt: bool) -> i32;
    }
}
use wasi_sock::*;

#[derive(Debug)]
pub struct Socket {
    fd: RawFd,
}

impl Socket {
    pub fn new(addr_family: AddressFamily, sock_kind: SocketType) -> io::Result<Self> {
        unsafe {
            let mut fd = 0;
            let res = sock_open(-1, addr_family as _, sock_kind as _, &mut fd);
            if res == 0 {
                Ok(Socket { fd })
            } else {
                Err(io::Error::from_raw_os_error(res))
            }
        }
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe { libc::write(self.as_raw_fd(), buf as *const _ as *const _, buf.len()) };
        if ret == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(ret as _)
        }
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe { libc::read(self.as_raw_fd(), buf as *mut _ as *mut _, buf.len()) };
        if ret == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(ret as _)
        }
    }
}

#[cfg(feature = "opt")]
use std::time::Duration;

#[cfg(feature = "opt")]
impl Socket {
    // pub fn device(&self) -> io::Result<Option<Vec<u8>>> {
    //     let mut buf: [MaybeUninit<u8>; 0x10] = unsafe { MaybeUninit::uninit().assume_init() };
    //     let mut len = buf.len() as u32;
    //     let e = unsafe {
    //         sock_getsockopt(
    //             self.fd,
    //             SocketOptLevel::SolSocket as i32,
    //             SocketOptName::SoBindToDevice as i32,
    //             &mut buf as *mut _ as *mut i32,
    //             &mut len,
    //         )
    //     };

    //     if e == 0 {
    //         if len == 0 {
    //             Ok(None)
    //         } else {
    //             let buf = &buf[..len as usize - 1];
    //             // TODO: use `MaybeUninit::slice_assume_init_ref` once stable.
    //             Ok(Some(unsafe { &*(buf as *const [_] as *const [u8]) }.into()))
    //         }
    //     } else {
    //         Err(io::Error::from_raw_os_error(e as i32))
    //     }
    // }

    // pub fn bind_device(&self, interface: Option<&[u8]>) -> io::Result<()> {
    //     let (value, len) = if let Some(interface) = interface {
    //         (interface.as_ptr(), interface.len())
    //     } else {
    //         (std::ptr::null(), 0)
    //     };

    //     unsafe {
    //         let e = sock_setsockopt(
    //             self.fd,
    //             SocketOptLevel::SolSocket as u8 as i32,
    //             SocketOptName::SoBindToDevice as u8 as i32,
    //             value as *const i32,
    //             len as u32,
    //         );
    //         if e == 0 {
    //             Ok(())
    //         } else {
    //             Err(io::Error::from_raw_os_error(e as i32))
    //         }
    //     }
    // }

    pub fn broadcast(&self) -> io::Result<bool> {
        let mut broadcast = MaybeUninit::<bool>::zeroed();
        mysyscall!(sock_get_broadcast(self.as_raw_fd(), broadcast.as_mut_ptr()))?;
        Ok(unsafe { broadcast.assume_init() })
    }

    pub fn keep_alive(&self) -> io::Result<bool> {
        let mut keep_alive = MaybeUninit::<bool>::zeroed();
        mysyscall!(sock_get_keep_alive(
            self.as_raw_fd(),
            keep_alive.as_mut_ptr()
        ))?;
        Ok(unsafe { keep_alive.assume_init() })
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        let mut is_linger = MaybeUninit::<bool>::zeroed();
        let mut duration = MaybeUninit::<i32>::zeroed();
        mysyscall!(sock_get_linger(
            self.as_raw_fd(),
            is_linger.as_mut_ptr(),
            duration.as_mut_ptr()
        ))?;
        if unsafe { is_linger.assume_init() } {
            Ok(Some(Duration::from_secs(
                unsafe { duration.assume_init() } as _
            )))
        } else {
            Ok(None)
        }
    }

    pub fn recv_buf_size(&self) -> io::Result<u32> {
        let mut size = MaybeUninit::<u32>::zeroed();
        mysyscall!(sock_get_recv_buf_size(self.as_raw_fd(), size.as_mut_ptr()))?;
        Ok(unsafe { size.assume_init() })
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        let mut timeout = MaybeUninit::<u64>::zeroed(); // rare u64 !
        mysyscall!(sock_get_recv_timeout(
            self.as_raw_fd(),
            timeout.as_mut_ptr()
        ))?;
        let timeout = unsafe { timeout.assume_init() };
        if timeout != 0 {
            Ok(Some(Duration::from_micros(timeout)))
        } else {
            Ok(None)
        }
    }

    pub fn reuse_addr(&self) -> io::Result<bool> {
        let mut reuse_addr = MaybeUninit::<bool>::zeroed();
        mysyscall!(sock_get_reuse_addr(
            self.as_raw_fd(),
            reuse_addr.as_mut_ptr()
        ))?;
        Ok(unsafe { reuse_addr.assume_init() })
    }

    pub fn reuse_port(&self) -> io::Result<bool> {
        let mut reuse_port = MaybeUninit::<bool>::zeroed();
        mysyscall!(sock_get_reuse_port(
            self.as_raw_fd(),
            reuse_port.as_mut_ptr()
        ))?;
        Ok(unsafe { reuse_port.assume_init() })
    }

    pub fn send_buf_size(&self) -> io::Result<u32> {
        let mut size = MaybeUninit::<u32>::zeroed();
        mysyscall!(sock_get_send_buf_size(self.as_raw_fd(), size.as_mut_ptr()))?;
        Ok(unsafe { size.assume_init() })
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        let mut timeout = MaybeUninit::<u64>::zeroed(); // rare u64 !
        mysyscall!(sock_get_send_timeout(
            self.as_raw_fd(),
            timeout.as_mut_ptr()
        ))?;
        let timeout = unsafe { timeout.assume_init() };
        if timeout != 0 {
            Ok(Some(Duration::from_micros(timeout)))
        } else {
            Ok(None)
        }
    }

    pub fn tcp_fastopen_connect(&self) -> io::Result<bool> {
        let mut opt = MaybeUninit::<bool>::zeroed();
        mysyscall!(sock_get_tcp_fastopen_connect(
            self.as_raw_fd(),
            opt.as_mut_ptr()
        ))?;
        Ok(unsafe { opt.assume_init() })
    }

    pub fn tcp_keep_idle(&self) -> io::Result<Duration> {
        let mut keep_idle = MaybeUninit::<u32>::zeroed();
        mysyscall!(sock_get_tcp_keep_idle(
            self.as_raw_fd(),
            keep_idle.as_mut_ptr()
        ))?;
        Ok(unsafe { Duration::from_secs(keep_idle.assume_init() as _) })
    }

    pub fn tcp_keep_intvl(&self) -> io::Result<Duration> {
        let mut keep_intvl = MaybeUninit::<u32>::zeroed();
        mysyscall!(sock_get_tcp_keep_intvl(
            self.as_raw_fd(),
            keep_intvl.as_mut_ptr()
        ))?;
        Ok(unsafe { Duration::from_secs(keep_intvl.assume_init() as _) })
    }

    pub fn tcp_no_delay(&self) -> io::Result<bool> {
        let mut no_delay = MaybeUninit::<bool>::zeroed();
        mysyscall!(sock_get_tcp_no_delay(
            self.as_raw_fd(),
            no_delay.as_mut_ptr()
        ))?;
        Ok(unsafe { no_delay.assume_init() })
    }

    pub fn tcp_quick_ack(&self) -> io::Result<bool> {
        let mut quick_ack = MaybeUninit::<bool>::zeroed();
        mysyscall!(sock_get_tcp_quick_ack(
            self.as_raw_fd(),
            quick_ack.as_mut_ptr()
        ))?;
        Ok(unsafe { quick_ack.assume_init() })
    }

    pub fn ip_multicast_loop(&self, ipv6: bool) -> io::Result<bool> {
        let mut opt = MaybeUninit::<bool>::zeroed();
        mysyscall!(sock_get_ip_multicast_loop(
            self.as_raw_fd(),
            ipv6,
            opt.as_mut_ptr()
        ))?;
        Ok(unsafe { opt.assume_init() })
    }

    pub fn ip_multicast_ttl(&self) -> io::Result<u32> {
        let mut ttl = MaybeUninit::<u32>::zeroed();
        mysyscall!(sock_get_ip_multicast_ttl(
            self.as_raw_fd(),
            ttl.as_mut_ptr()
        ))?;
        Ok(unsafe { ttl.assume_init() })
    }

    pub fn ip_ttl(&self) -> io::Result<u32> {
        let mut ttl = MaybeUninit::<u32>::zeroed();
        mysyscall!(sock_get_ip_ttl(self.as_raw_fd(), ttl.as_mut_ptr()))?;
        Ok(unsafe { ttl.assume_init() })
    }

    pub fn ipv6_only(&self) -> io::Result<bool> {
        let mut ipv6 = MaybeUninit::<bool>::zeroed();
        mysyscall!(sock_get_ipv6_only(self.as_raw_fd(), ipv6.as_mut_ptr()))?;
        Ok(unsafe { ipv6.assume_init() })
    }

    pub fn set_broadcast(&self, opt: bool) -> io::Result<()> {
        mysyscall!(sock_set_broadcast(self.as_raw_fd(), opt))?;
        Ok(())
    }

    pub fn set_keep_alive(&self, opt: bool) -> io::Result<()> {
        mysyscall!(sock_set_keep_alive(self.as_raw_fd(), opt))?;
        Ok(())
    }

    /// be advised that [`Duration`] here is truncated into whole seconds
    pub fn set_linger(&self, linger: Option<Duration>) -> io::Result<()> {
        let linger_s = linger.unwrap_or_default().as_secs();
        mysyscall!(sock_set_linger(
            self.as_raw_fd(),
            linger.is_some(),
            linger_s as i32
        ))?;
        Ok(())
    }

    /// be advised that [`usize`] here is truncated into a wasi native size_t: u32
    pub fn set_recv_buf_size(&self, opt: usize) -> io::Result<()> {
        mysyscall!(sock_set_recv_buf_size(self.as_raw_fd(), opt as u32))?;
        Ok(())
    }

    pub fn set_read_timeout(&self, opt: Option<Duration>) -> io::Result<()> {
        let timeout = opt.unwrap_or_default().as_micros() as u64;
        mysyscall!(sock_set_recv_timeout(self.as_raw_fd(), timeout))?;
        Ok(())
    }

    pub fn set_reuse_addr(&self, opt: bool) -> io::Result<()> {
        mysyscall!(sock_set_reuse_addr(self.as_raw_fd(), opt))?;
        Ok(())
    }

    pub fn set_reuse_port(&self, opt: bool) -> io::Result<()> {
        mysyscall!(sock_set_reuse_port(self.as_raw_fd(), opt))?;
        Ok(())
    }

    /// be advised that [`usize`] here is truncated into a wasi native size_t: u32
    pub fn set_send_buf_size(&self, opt: usize) -> io::Result<()> {
        mysyscall!(sock_set_send_buf_size(self.as_raw_fd(), opt as u32))?;
        Ok(())
    }

    pub fn set_write_timeout(&self, opt: Option<Duration>) -> io::Result<()> {
        let timeout = opt.unwrap_or_default().as_micros() as u64;
        mysyscall!(sock_set_send_timeout(self.as_raw_fd(), timeout))?;
        Ok(())
    }

    pub fn set_tcp_fastopen_connect(&self, opt: bool) -> io::Result<()> {
        mysyscall!(sock_set_tcp_fastopen_connect(self.as_raw_fd(), opt))?;
        Ok(())
    }

    /// be advised that [`Duration`] here is truncated into whole seconds
    pub fn set_tcp_keep_idle(&self, opt: Duration) -> io::Result<()> {
        let timeout = opt.as_secs() as u32;
        mysyscall!(sock_set_tcp_keep_idle(self.as_raw_fd(), timeout))?;
        Ok(())
    }

    /// be advised that [`Duration`] here is truncated into whole seconds
    pub fn set_tcp_keep_intvl(&self, opt: Duration) -> io::Result<()> {
        let timeout = opt.as_secs() as u32;
        mysyscall!(sock_set_tcp_keep_intvl(self.as_raw_fd(), timeout))?;
        Ok(())
    }

    pub fn set_tcp_no_delay(&self, opt: bool) -> io::Result<()> {
        mysyscall!(sock_set_tcp_no_delay(self.as_raw_fd(), opt))?;
        Ok(())
    }

    pub fn set_tcp_quick_ack(&self, opt: bool) -> io::Result<()> {
        mysyscall!(sock_set_tcp_quick_ack(self.as_raw_fd(), opt))?;
        Ok(())
    }

    pub fn set_ip_multicast_loop(&self, ipv6: bool, opt: bool) -> io::Result<()> {
        mysyscall!(sock_set_ip_multicast_loop(self.as_raw_fd(), ipv6, opt))?;
        Ok(())
    }

    pub fn set_ip_multicast_ttl(&self, opt: u32) -> io::Result<()> {
        mysyscall!(sock_set_ip_multicast_ttl(self.as_raw_fd(), opt))?;
        Ok(())
    }

    pub fn set_ip_add_membership(&self, addr: &net::IpAddr, interface: u32) -> io::Result<()> {
        let addr: IpAddr = addr.into();
        mysyscall!(sock_set_ip_add_membership(
            self.as_raw_fd(),
            &addr as *const IpAddr,
            interface
        ))?;
        Ok(())
    }

    pub fn set_ip_drop_membership(&self, addr: &net::IpAddr, interface: u32) -> io::Result<()> {
        let addr: IpAddr = addr.into();
        mysyscall!(sock_set_ip_drop_membership(
            self.as_raw_fd(),
            &addr as *const IpAddr,
            interface
        ))?;
        Ok(())
    }

    pub fn set_ip_ttl(&self, opt: u32) -> io::Result<()> {
        mysyscall!(sock_set_ip_ttl(self.as_raw_fd(), opt))?;
        Ok(())
    }

    pub fn set_ipv6_only(&self, opt: bool) -> io::Result<()> {
        mysyscall!(sock_set_ipv6_only(self.as_raw_fd(), opt))?;
        Ok(())
    }
}

#[cfg(feature = "iov")]
impl Socket {
    pub fn send_with_flags(&self, buf: &[u8], flags: u16) -> io::Result<usize> {
        let vec = iovec {
            iov_base: buf.as_ptr() as *mut _,
            iov_len: buf.len(),
        };

        let mut send_len: u32 = 0;
        let res = unsafe { sock_send(self.fd, &vec, 1, flags, &mut send_len) };
        if res == 0 {
            Ok(send_len as usize)
        } else {
            Err(io::Error::from_raw_os_error(res))
        }
    }

    pub fn send_vectored(&self, bufs: &[io::IoSlice<'_>], flags: u16) -> io::Result<usize> {
        let iov = bufs.as_ptr() as *const _; // IoSlice is guaranteed to be equivalent to iovec

        let mut send_len: u32 = 0;
        let res = unsafe { sock_send(self.fd, iov, bufs.len() as u32, flags, &mut send_len) };
        if res == 0 {
            Ok(send_len as usize)
        } else {
            Err(io::Error::from_raw_os_error(res))
        }
    }

    pub fn send_to(&self, buf: &[u8], addr: &net::SocketAddr) -> io::Result<usize> {
        let addr = addr.into();

        let vec = iovec {
            iov_base: buf.as_ptr() as *mut _,
            iov_len: buf.len(),
        };

        let flags = 0;
        let mut send_len: u32 = 0;
        let res = unsafe { sock_send_to(self.fd, &vec, 1, flags, &addr, &mut send_len) };
        if res == 0 {
            Ok(send_len as usize)
        } else {
            Err(io::Error::from_raw_os_error(res))
        }
    }

    pub fn send_to_vectored(
        &self,
        bufs: &[io::IoSlice<'_>],
        addr: &net::SocketAddr,
        flags: u16,
    ) -> io::Result<usize> {
        let addr = addr.into();

        let iov = bufs.as_ptr() as *const _; // IoSlice is guaranteed to be equivalent to iovec

        let mut send_len: u32 = 0;
        let res =
            unsafe { sock_send_to(self.fd, iov, bufs.len() as u32, flags, &addr, &mut send_len) };
        if res == 0 {
            Ok(send_len as usize)
        } else {
            Err(io::Error::from_raw_os_error(res))
        }
    }

    pub fn recv_with_flags(
        &self,
        buf: &mut [MaybeUninit<u8>],
        flags: u16,
    ) -> io::Result<(u32, i32)> {
        let mut recv_len: u32 = 0;
        let mut oflags: i32 = 0;

        let iov = libc::iovec {
            iov_base: buf.as_mut_ptr() as *mut _,
            iov_len: buf.len(),
        };

        unsafe {
            let res = sock_recv(self.as_raw_fd(), &iov, 1, flags, &mut recv_len, &mut oflags);
            if res == 0 {
                Ok((recv_len, oflags))
            } else {
                Err(io::Error::from_raw_os_error(res))
            }
        }
    }

    pub fn recv_vectored(
        &self,
        bufs: &mut [io::IoSliceMut<'_>],
        flags: u16,
    ) -> io::Result<(usize, i32)> {
        let mut recv_len: u32 = 0;
        let mut oflags: i32 = 0;

        let iov = bufs.as_ptr() as *const _; // IoSlice is guaranteed to be equivalent to iovec

        let res = unsafe {
            sock_recv(
                self.as_raw_fd(),
                iov,
                bufs.len() as u32,
                flags,
                &mut recv_len,
                &mut oflags,
            )
        };
        if res == 0 {
            Ok((recv_len as usize, oflags))
        } else {
            Err(io::Error::from_raw_os_error(res))
        }
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, net::SocketAddr)> {
        let mut recv_len: u32 = 0;

        let iov = libc::iovec {
            iov_base: buf.as_mut_ptr() as *mut _,
            iov_len: buf.len(),
        };

        let mut addr = SocketAddr::default();

        let flags = 0;

        let res = unsafe {
            sock_recv_from(
                self.as_raw_fd(),
                &iov as *const _,
                1,
                flags,
                &mut addr,
                &mut recv_len,
            )
        };
        if res == 0 {
            Ok((recv_len as usize, (&addr).into()))
        } else {
            Err(io::Error::from_raw_os_error(res))
        }
    }

    // pub fn recv_from_with_flags(
    //     &self,
    //     buf: &mut [MaybeUninit<u8>],
    //     flags: u16,
    // ) -> io::Result<(usize, SocketAddr, usize)> {
    // }

    pub fn recv_from_vectored(
        &self,
        bufs: &[io::IoSliceMut<'_>],
        flags: u16,
    ) -> io::Result<(usize, net::SocketAddr)> {
        let mut recv_len: u32 = 0;

        let iov = bufs.as_ptr() as *const _; // IoSlice is guaranteed to be equivalent to iovec

        let mut addr = SocketAddr::default();

        let res = unsafe {
            sock_recv_from(
                self.as_raw_fd(),
                iov,
                bufs.len() as u32,
                flags,
                &mut addr,
                &mut recv_len,
            )
        };
        if res == 0 {
            Ok((recv_len as usize, (&addr).into()))
        } else {
            Err(io::Error::from_raw_os_error(res))
        }
    }
}

impl Socket {
    pub fn nonblocking(&self) -> io::Result<bool> {
        let fd = self.as_raw_fd();
        let file_status_flags = fcntl_get(fd, libc::F_GETFL)?;
        Ok((file_status_flags & libc::O_NONBLOCK) != 0)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        let fd = self.as_raw_fd();
        if nonblocking {
            fcntl_add(fd, libc::F_GETFL, libc::F_SETFL, libc::O_NONBLOCK)
        } else {
            fcntl_remove(fd, libc::F_GETFL, libc::F_SETFL, libc::O_NONBLOCK)
        }
    }

    pub fn connect(&self, addrs: &net::SocketAddr) -> io::Result<()> {
        let fd = self.as_raw_fd();

        let addr = addrs.into();

        let res = unsafe { sock_connect(fd, &addr) };
        if res != 0 {
            Err(io::Error::from_raw_os_error(res))
        } else {
            Ok(())
        }
    }

    pub fn bind(&self, addrs: &net::SocketAddr) -> io::Result<()> {
        let fd = self.as_raw_fd();
        let addr = addrs.into();
        let res = unsafe { sock_bind(fd, &addr) };
        if res != 0 {
            Err(io::Error::from_raw_os_error(res))
        } else {
            Ok(())
        }
    }

    pub fn listen(&self, backlog: u32) -> io::Result<()> {
        let fd = self.as_raw_fd();
        let res = unsafe { sock_listen(fd, backlog) };
        if res != 0 {
            Err(io::Error::from_raw_os_error(res))
        } else {
            Ok(())
        }
    }

    pub fn accept(&self, nonblocking: bool) -> io::Result<Self> {
        unsafe {
            let mut fd: i32 = 0;
            let mut flags = 0;
            if nonblocking {
                flags |= FDFLAG_NONBLOCK;
            }
            let res = sock_accept(self.as_raw_fd(), flags, &mut fd);
            if res != 0 {
                Err(io::Error::from_raw_os_error(res))
            } else {
                let s = Socket { fd };
                // s.set_nonblocking(nonblocking)?;
                Ok(s)
            }
        }
    }

    pub fn get_local(&self) -> io::Result<net::SocketAddr> {
        let mut addr = MaybeUninit::<SocketAddr>::uninit();

        let res = unsafe { sock_addr_local(self.fd, addr.as_mut_ptr()) };
        if res != 0 {
            Err(io::Error::from_raw_os_error(res))
        } else {
            let addr = unsafe { addr.assume_init() };
            Ok((&addr).into())
        }
    }

    pub fn get_peer(&self) -> io::Result<net::SocketAddr> {
        let mut addr = MaybeUninit::<SocketAddr>::uninit();

        let res = unsafe { sock_addr_remote(self.fd, addr.as_mut_ptr()) };
        if res != 0 {
            Err(io::Error::from_raw_os_error(res))
        } else {
            let addr = unsafe { addr.assume_init() };
            Ok((&addr).into())
        }
    }

    pub fn shutdown(&self, how: net::Shutdown) -> io::Result<()> {
        unsafe {
            let flags = match how {
                net::Shutdown::Read => 0b01,
                net::Shutdown::Write => 0b10,
                net::Shutdown::Both => 0b11,
            };
            let res = libc::shutdown(self.as_raw_fd(), flags);
            if res == 0 {
                Ok(())
            } else {
                Err(io::Error::from_raw_os_error(res))
            }
        }
    }
}

#[cfg(feature = "opt")]
impl Socket {
    // pub fn take_error(&self) -> io::Result<()> {
    //     unsafe {
    //         let fd = self.fd;
    //         let mut error = 0;
    //         let mut len = std::mem::size_of::<i32>() as u32;
    //         let res = sock_getsockopt(
    //             fd,
    //             SocketOptLevel::SolSocket as i32,
    //             SocketOptName::SoError as i32,
    //             &mut error,
    //             &mut len,
    //         );
    //         if res == 0 && error == 0 {
    //             Ok(())
    //         } else if res == 0 && error != 0 {
    //             Err(io::Error::from_raw_os_error(error))
    //         } else {
    //             Err(io::Error::from_raw_os_error(res))
    //         }
    //     }
    // }

    // pub fn is_listener(&self) -> io::Result<bool> {
    //     unsafe {
    //         let fd = self.fd;
    //         let mut val = 0;
    //         let mut len = std::mem::size_of::<i32>() as u32;
    //         let res = sock_getsockopt(
    //             fd as i32,
    //             SocketOptLevel::SolSocket as i32,
    //             SocketOptName::SoAcceptconn as i32,
    //             &mut val,
    //             &mut len,
    //         );
    //         if res != 0 {
    //             Err(io::Error::from_raw_os_error(res))
    //         } else {
    //             Ok(val != 0)
    //         }
    //     }
    // }

    // pub fn r#type(&self) -> io::Result<SocketType> {
    //     unsafe {
    //         let fd = self.fd;
    //         let mut val = 0;
    //         let mut len = std::mem::size_of::<i32>() as u32;
    //         let res = sock_getsockopt(
    //             fd as u32,
    //             SocketOptLevel::SolSocket as i32,
    //             SocketOptName::SoType as i32,
    //             &mut val,
    //             &mut len,
    //         );
    //         if res != 0 {
    //             Err(io::Error::from_raw_os_error(res))
    //         } else {
    //             match val {
    //                 1 => Ok(SocketType::Datagram),
    //                 2 => Ok(SocketType::Stream),
    //                 _ => Err(io::Error::from_raw_os_error(libc::EINVAL)),
    //             }
    //         }
    //     }
    // }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = self.shutdown(net::Shutdown::Both);
        unsafe { libc::close(self.fd) };
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl AsFd for Socket {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

impl IntoRawFd for Socket {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.fd;
        std::mem::forget(self);
        fd
    }
}

impl FromRawFd for Socket {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Socket { fd }
    }
}

impl std::io::Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.send(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl std::io::Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv(buf)
    }
}

impl std::io::Write for &Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.send(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl std::io::Read for &Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv(buf)
    }
}
