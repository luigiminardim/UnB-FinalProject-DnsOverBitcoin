use lib::dns::{
    core::{AData, Class, Data, Record},
    net::{UdpListener, UdpStubResolver},
    resolver::InMemoryAuthority,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let cloud_flare_socket_addr = "1.1.1.1:53".parse::<SocketAddr>().unwrap();
    let _udp_resolver = UdpStubResolver::new(cloud_flare_socket_addr);
    let in_memory_authority = InMemoryAuthority::new(vec![Record::new(
        "ord.".parse().unwrap(),
        Class::IN,
        0,
        Data::A(AData::new("1.2.3.4".parse().unwrap())),
    )]);
    let udp_listener = UdpListener::new(in_memory_authority);
    udp_listener
        .listen()
        .await
        .inspect_err(|e| {
            eprintln!("Error: {:?}", e);
        })
        .unwrap();
}
