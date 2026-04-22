use wamr_wasi_socket::{
    socket::{AddressFamily, Socket, SocketType},
    ToSocketAddrs,
};

fn main() {
    let s = Socket::new(AddressFamily::Inet4, SocketType::Stream).unwrap();
    // let device = s.device().unwrap();
    // assert!(device.is_none());
    // s.bind_device(Some(b"lo")).unwrap();
    // let device = s.device().unwrap();
    // assert!(device.is_some());
    // assert_eq!(device.unwrap(), b"lo");
    let addr = "8.8.8.8:53".to_socket_addrs().unwrap().next().unwrap();

    let read_timeout = s.read_timeout().unwrap();
    println!("read_timeout {:?}", read_timeout);
    let write_timeout = s.write_timeout().unwrap();
    println!("write_timeout {:?}", write_timeout);

    let snd_timeout = std::time::Duration::from_secs(1);
    let rcv_timeout = std::time::Duration::from_secs(1);

    s.set_read_timeout(Some(snd_timeout)).unwrap();
    s.set_write_timeout(Some(rcv_timeout)).unwrap();

    let read_timeout = s.read_timeout().unwrap();
    println!("read_timeout {:?}", read_timeout);
    assert_eq!(read_timeout, Some(rcv_timeout));
    let write_timeout = s.write_timeout().unwrap();
    println!("write_timeout {:?}", write_timeout);
    assert_eq!(write_timeout, Some(snd_timeout));

    println!("start connect {addr}");
    assert!(s.connect(&addr).is_err());
}
