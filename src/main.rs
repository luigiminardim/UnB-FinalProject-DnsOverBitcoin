use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use lib::dns::{
    net::{UdpListener, UdpStubResolver},
    // resolver::EmptyResolver,
};

#[tokio::main]
async fn main() {
    let udp_listener = UdpListener::new(UdpStubResolver::new(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
        53,
    )));
    udp_listener
        .listen()
        .await
        .map_err(|e| {
            eprintln!("Error: {:?}", e);
        })
        .unwrap();
}
