use wamr_wasi_socket::lookup_host;

fn main() {
    let addrs = lookup_host("google.com", 80).unwrap();
    for addr in addrs {
        println!("{:?}", addr);
    }
}
