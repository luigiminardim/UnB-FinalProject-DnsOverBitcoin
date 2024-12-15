use lib::dns::{net::UdpListener, resolver::EmptyResolver};

#[tokio::main]
async fn main() {
    let udp_listener = UdpListener::new(EmptyResolver::new());
    udp_listener
        .listen()
        .await
        .map_err(|e| {
            eprintln!("Error: {:?}", e);
        })
        .unwrap();
}
