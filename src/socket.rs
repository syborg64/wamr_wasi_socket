use std::io;
use std::mem::MaybeUninit;
use std::net;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, RawFd};

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

#[cfg(test)]
mod test {
    use super::{SocketAddr, SocketAddrV6};
    use std::net;
    use std::ptr::addr_of;
    use std::str::FromStr;

    #[test]
    fn test_union_layout() {
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
    use super::*;

    #[repr(u16, align(2))]
    pub enum AiFlags {
        AiPassive,
        AiCanonname,
        AiNumericHost,
        AiNumericServ,
        AiV4Mapped,
        AiAll,
        AiAddrConfig,
    }

    #[derive(Copy, Clone, Debug)]
    #[repr(u8, align(1))]
    pub enum AiProtocol {
        IPProtoIP,
        IPProtoTCP,
        IPProtoUDP,
    }

    #[derive(Debug, Clone)]
    #[repr(C, packed(4))]
    pub struct WasiAddrinfo {
        pub ai_flags: AiFlags,
        pub ai_family: AddressFamily,
        pub ai_socktype: SocketType,
        pub ai_protocol: AiProtocol,
        pub ai_addrlen: u32,
        pub ai_addr: *mut SocketAddr,
        pub ai_canonname: *mut u8,
        pub ai_canonnamelen: u32,
        pub ai_next: *mut WasiAddrinfo,
    }

    impl WasiAddrinfo {
        pub fn default() -> WasiAddrinfo {
            WasiAddrinfo {
                ai_flags: AiFlags::AiPassive,
                ai_family: AddressFamily::Inet4,
                ai_socktype: SocketType::Stream,
                ai_protocol: AiProtocol::IPProtoTCP,
                ai_addr: std::ptr::null_mut(),
                ai_addrlen: 0,
                ai_canonname: std::ptr::null_mut(),
                ai_canonnamelen: 0,
                ai_next: std::ptr::null_mut(),
            }
        }

        /// Get Address Information
        ///
        /// As calling FFI, use buffer as parameter in order to avoid memory leak.
        pub fn get_addrinfo(
            node: &str,
            service: &str,
            hints: &WasiAddrinfo,
            max_reslen: usize,
            sockaddr: &mut Vec<SocketAddr>,
            sockbuff: &mut Vec<[u8; 26]>,
            ai_canonname: &mut Vec<String>,
        ) -> io::Result<Vec<WasiAddrinfo>> {
            #[link(wasm_import_module = "wasi_snapshot_preview1")]
            extern "C" {
                pub fn sock_getaddrinfo(
                    node: *const u8,
                    node_len: u32,
                    server: *const u8,
                    server_len: u32,
                    hint: *const WasiAddrinfo,
                    res: *mut u32,
                    max_len: u32,
                    res_len: *mut u32,
                ) -> u32;
            }
            let mut node = node.to_string();
            let mut service = service.to_string();

            if !node.ends_with('\0') {
                node.push('\0');
            }

            if !service.ends_with('\0') {
                service.push('\0');
            }

            let mut res_len: u32 = 0;
            sockbuff.resize(max_reslen, [0u8; 26]);
            ai_canonname.resize(max_reslen, String::with_capacity(30));
            sockaddr.resize(max_reslen, WasiSockaddr::default());
            let mut wasiaddrinfo_array: Vec<WasiAddrinfo> =
                vec![WasiAddrinfo::default(); max_reslen];

            for i in 0..max_reslen {
                sockaddr[i].sa_data = sockbuff[i].as_mut_ptr();
                wasiaddrinfo_array[i].ai_addr = &mut sockaddr[i];
                wasiaddrinfo_array[i].ai_canonname = ai_canonname[i].as_mut_ptr();
                if i > 0 {
                    wasiaddrinfo_array[i - 1].ai_next = &mut wasiaddrinfo_array[i];
                }
            }
            let mut res = wasiaddrinfo_array.as_mut_ptr() as u32;

            unsafe {
                let return_code = sock_getaddrinfo(
                    node.as_ptr(),
                    node.len() as u32,
                    service.as_ptr(),
                    service.len() as u32,
                    hints as *const WasiAddrinfo,
                    &mut res,
                    max_reslen as u32,
                    &mut res_len,
                );

                match return_code {
                    0 => Ok(wasiaddrinfo_array[..res_len as usize].to_vec()),
                    e => Err(std::io::Error::from_raw_os_error(e as i32)),
                }
            }
        }
    }
}

#[cfg(feature = "addrinfo")]
pub use addrinfo::*;

#[cfg(feature = "iov")]
pub mod iov {
    use super::*;

    #[repr(C)]
    pub struct IovecRead {
        pub buf: *mut u8,
        pub size: usize,
    }

    impl From<libc::iovec> for IovecRead {
        fn from(value: libc::iovec) -> Self {
            IovecRead {
                buf: value.iov_base.cast(),
                size: value.iov_len,
            }
        }
    }

    #[repr(C)]
    pub struct IovecWrite {
        pub buf: *const u8,
        pub size: usize,
    }

    impl From<libc::iovec> for IovecWrite {
        fn from(value: libc::iovec) -> Self {
            IovecWrite {
                buf: value.iov_base.cast(),
                size: value.iov_len,
            }
        }
    }
}
#[cfg(feature = "iov")]
pub use iov::*;

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
#[cfg(feature = "opt")]
pub use opt::*;

// pub const MSG_PEEK: u16 = 1; // __WASI_RIFLAGS_RECV_PEEK
// pub const MSG_WAITALL: u16 = 2; // __WASI_RIFLAGS_RECV_WAITALL

// pub const MSG_TRUNC: u16 = 1; // __WASI_ROFLAGS_RECV_DATA_TRUNCATED

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
    use super::SocketAddr;
    #[cfg(feature = "iov")]
    use super::{IovecRead, IovecWrite};

    #[link(wasm_import_module = "wasi_snapshot_preview1")]
    extern "C" {
        pub fn sock_open(poolfd: i32, af: i32, socktype: i32, sockfd: *mut i32) -> i32;

        pub fn sock_bind(sockfd: i32, addr: *const SocketAddr) -> i32;

        pub fn sock_listen(sockfd: i32, backlog: u32) -> i32;

        pub fn sock_accept(socket: i32, flags: u16, fd_new: *mut i32) -> i32;

        pub fn sock_connect(socket: i32, addr: *const SocketAddr) -> i32;

        #[cfg(feature = "iov")]
        pub fn sock_recv(
            fd: u32,
            buf: *mut IovecRead,
            buf_len: usize,
            flags: u16,
            recv_len: *mut usize,
            oflags: *mut usize,
        ) -> u32;

        #[cfg(feature = "iov")]
        pub fn sock_recv_from(
            fd: u32,
            buf: *mut IovecRead,
            buf_len: u32,
            addr: *mut u8,
            flags: u16,
            port: *mut u32,
            recv_len: *mut usize,
            oflags: *mut usize,
        ) -> u32;

        #[cfg(feature = "iov")]
        pub fn sock_send(
            fd: u32,
            buf: *const IovecWrite,
            buf_len: u32,
            flags: u16,
            send_len: *mut u32,
        ) -> u32;

        #[cfg(feature = "iov")]
        pub fn sock_send_to(
            fd: u32,
            buf: *const IovecWrite,
            buf_len: u32,
            addr: *const u8,
            port: u32,
            flags: u16,
            send_len: *mut u32,
        ) -> u32;

        pub fn sock_addr_remote(fd: i32, addr: *mut SocketAddr) -> i32;

        pub fn sock_addr_local(fd: i32, addr: *mut SocketAddr) -> i32;

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
        pub fn sock_set_ip_add_membership(fd: i32, addr: *const SocketAddr, interface: u32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_ip_drop_membership(fd: i32, addr: *const SocketAddr, interface: u32)
            -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_ip_ttl(fd: i32, opt: u32) -> i32;

        #[cfg(feature = "opt")]
        pub fn sock_set_ipv6_only(fd: i32, opt: bool) -> i32;
    }
}

#[derive(Debug)]
pub struct Socket {
    fd: RawFd,
}

use wasi_sock::*;

#[cfg(feature = "opt")]
use crate::syscall::mysyscall;
use crate::syscall::syscall;

impl Socket {
    pub fn new(addr_family: AddressFamily, sock_kind: SocketType) -> io::Result<Self> {
        unsafe {
            let mut fd = 0;
            let res = sock_open(-1, addr_family as _, sock_kind as _, &mut fd);
            if res == 0 {
                Ok(Socket { fd: fd as i32 })
            } else {
                Err(io::Error::from_raw_os_error(res))
            }
        }
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe { libc::write(self.as_raw_fd(), buf as *const _ as *const _, buf.len()) };
        if ret == -1 {
            return Err(std::io::Error::last_os_error());
        } else {
            return Ok(ret as _);
        }
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe { libc::read(self.as_raw_fd(), buf as *mut _ as *mut _, buf.len()) };
        if ret == -1 {
            return Err(std::io::Error::last_os_error());
        } else {
            return Ok(ret as _);
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

    pub fn set_ip_add_membership(&self, addr: &net::SocketAddr, interface: u32) -> io::Result<()> {
        let addr: SocketAddr = addr.into();
        mysyscall!(sock_set_ip_add_membership(
            self.as_raw_fd(),
            &addr as *const SocketAddr,
            interface
        ))?;
        Ok(())
    }

    pub fn set_ip_drop_membership(&self, addr: &net::SocketAddr, interface: u32) -> io::Result<()> {
        let addr: SocketAddr = addr.into();
        mysyscall!(sock_set_ip_drop_membership(
            self.as_raw_fd(),
            &addr as *const SocketAddr,
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
    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        let addr = (&addr).into();

        let vec = IovecWrite {
            buf: buf.as_ptr(),
            size: buf.len(),
        };

        let flags = 0;
        let mut send_len: u32 = 0;
        let res = unsafe {
            sock_send_to(
                self.fd as u32,
                &vec,
                1,
                &addr as *const WasiAddress as *const u8,
                port,
                flags,
                &mut send_len,
            )
        };
        if res == 0 {
            Ok(send_len as usize)
        } else {
            Err(io::Error::from_raw_os_error(res))
        }
    }

    pub fn send_to_vectored(
        &self,
        bufs: &[io::IoSlice<'_>],
        addr: SocketAddr,
        flags: u16,
    ) -> io::Result<usize> {
        let port = addr.port() as u32;
        let vaddr = match addr {
            SocketAddr::V4(ipv4) => ipv4.ip().octets().to_vec(),
            SocketAddr::V6(ipv6) => ipv6.ip().octets().to_vec(),
        };
        let addr = WasiAddress {
            buf: vaddr.as_ptr(),
            size: vaddr.len(),
        };

        let mut write_bufs = Vec::with_capacity(bufs.len());
        for b in bufs {
            write_bufs.push(IovecWrite {
                buf: b.as_ptr().cast(),
                size: b.len(),
            });
        }

        let mut send_len: u32 = 0;
        unsafe {
            let res = sock_send_to(
                self.fd as u32,
                write_bufs.as_ptr(),
                write_bufs.len() as u32,
                &addr as *const WasiAddress as *const u8,
                port,
                flags,
                &mut send_len,
            );
            if res == 0 {
                Ok(send_len as usize)
            } else {
                Err(io::Error::from_raw_os_error(res))
            }
        }
    }

    pub fn recv_with_flags(
        &self,
        buf: &mut [MaybeUninit<u8>],
        flags: u16,
    ) -> io::Result<(usize, usize)> {
        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;
        let mut vec = IovecRead {
            buf: buf.as_mut_ptr().cast(),
            size: buf.len(),
        };

        unsafe {
            let res = sock_recv(
                self.as_raw_fd() as u32,
                &mut vec,
                1,
                flags,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                Ok((recv_len, oflags))
            } else {
                Err(io::Error::from_raw_os_error(res))
            }
        }
    }

    pub fn recv_vectored(&self, bufs: &mut [IovecRead], flags: u16) -> io::Result<(usize, usize)> {
        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;

        unsafe {
            let res = sock_recv(
                self.as_raw_fd() as u32,
                bufs.as_mut_ptr(),
                bufs.len(),
                flags,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                Ok((recv_len, oflags))
            } else {
                Err(io::Error::from_raw_os_error(res))
            }
        }
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let flags = 0;
        let addr_buf = [0; 128];

        let mut addr = WasiAddress {
            buf: addr_buf.as_ptr(),
            size: 128,
        };

        let mut recv_buf = IovecRead {
            buf: buf.as_mut_ptr(),
            size: buf.len(),
        };

        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;
        let mut sin_port: u32 = 0;
        unsafe {
            let res = sock_recv_from(
                self.as_raw_fd() as u32,
                &mut recv_buf,
                1,
                &mut addr as *mut WasiAddress as *mut u8,
                flags,
                &mut sin_port,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                let sin_family = {
                    let mut d = [0, 0];
                    d.clone_from_slice(&addr_buf[0..2]);
                    u16::from_le_bytes(d) as u8
                };
                let sin_addr = if sin_family == AddressFamily::Inet4 as u8 {
                    let ip_addr = Ipv4Addr::new(addr_buf[2], addr_buf[3], addr_buf[4], addr_buf[5]);
                    SocketAddr::V4(SocketAddrV4::new(ip_addr, sin_port as u16))
                } else if sin_family == AddressFamily::Inet6 as u8 {
                    let mut ipv6_addr = [0u8; 16];
                    ipv6_addr.copy_from_slice(&addr_buf[2..18]);
                    let ip_addr = Ipv6Addr::from(ipv6_addr);
                    SocketAddr::V6(SocketAddrV6::new(ip_addr, sin_port as u16, 0, 0))
                } else {
                    unimplemented!("Address family not supported by protocol");
                };

                Ok((recv_len, sin_addr))
            } else {
                Err(io::Error::from_raw_os_error(res))
            }
        }
    }

    pub fn recv_from_with_flags(
        &self,
        buf: &mut [MaybeUninit<u8>],
        flags: u16,
    ) -> io::Result<(usize, SocketAddr, usize)> {
        let addr_buf = [0; 128];

        let mut addr = WasiAddress {
            buf: addr_buf.as_ptr(),
            size: 128,
        };

        let mut recv_buf = IovecRead {
            buf: buf.as_mut_ptr().cast(),
            size: buf.len(),
        };

        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;
        let mut sin_port: u32 = 0;
        unsafe {
            let res = sock_recv_from(
                self.as_raw_fd() as u32,
                &mut recv_buf,
                1,
                &mut addr as *mut WasiAddress as *mut u8,
                flags,
                &mut sin_port,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                let sin_family = {
                    let mut d = [0, 0];
                    d.clone_from_slice(&addr_buf[0..2]);
                    u16::from_le_bytes(d) as u8
                };
                let sin_addr = if sin_family == AddressFamily::Inet4 as u8 {
                    let ip_addr = Ipv4Addr::new(addr_buf[2], addr_buf[3], addr_buf[4], addr_buf[5]);
                    SocketAddr::V4(SocketAddrV4::new(ip_addr, sin_port as u16))
                } else if sin_family == AddressFamily::Inet6 as u8 {
                    let mut ipv6_addr = [0u8; 16];
                    ipv6_addr.copy_from_slice(&addr_buf[2..18]);
                    let ip_addr = Ipv6Addr::from(ipv6_addr);
                    SocketAddr::V6(SocketAddrV6::new(ip_addr, sin_port as u16, 0, 0))
                } else {
                    unimplemented!("Address family not supported by protocol");
                };

                Ok((recv_len, sin_addr, oflags))
            } else {
                Err(io::Error::from_raw_os_error(res))
            }
        }
    }

    pub fn recv_from_vectored(
        &self,
        bufs: &mut [IovecRead],
        flags: u16,
    ) -> io::Result<(usize, SocketAddr, usize)> {
        let addr_buf = [0; 128];

        let mut addr = WasiAddress {
            buf: addr_buf.as_ptr(),
            size: 128,
        };

        let mut recv_len: usize = 0;
        let mut oflags: usize = 0;
        let mut sin_port: u32 = 0;
        unsafe {
            let res = sock_recv_from(
                self.as_raw_fd() as u32,
                bufs.as_mut_ptr(),
                1,
                &mut addr as *mut WasiAddress as *mut u8,
                flags,
                &mut sin_port,
                &mut recv_len,
                &mut oflags,
            );
            if res == 0 {
                let sin_family = {
                    let mut d = [0, 0];
                    d.clone_from_slice(&addr_buf[0..2]);
                    u16::from_le_bytes(d) as u8
                };
                let sin_addr = if sin_family == AddressFamily::Inet4 as u8 {
                    let ip_addr = Ipv4Addr::new(addr_buf[2], addr_buf[3], addr_buf[4], addr_buf[5]);
                    SocketAddr::V4(SocketAddrV4::new(ip_addr, sin_port as u16))
                } else if sin_family == AddressFamily::Inet6 as u8 {
                    let mut ipv6_addr = [0u8; 16];
                    ipv6_addr.copy_from_slice(&addr_buf[2..18]);
                    let ip_addr = Ipv6Addr::from(ipv6_addr);
                    SocketAddr::V6(SocketAddrV6::new(ip_addr, sin_port as u16, 0, 0))
                } else {
                    unimplemented!("Address family not supported by protocol");
                };

                Ok((recv_len, sin_addr, oflags))
            } else {
                Err(io::Error::from_raw_os_error(res))
            }
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
                let s = Socket { fd: fd as i32 };
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
            // Err(io::Error::from(io::ErrorKind::Unsupported))
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
            // Err(io::Error::from(io::ErrorKind::Unsupported))
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
