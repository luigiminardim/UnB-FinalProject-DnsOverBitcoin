use lib::dns::net::udp_listener::UdpListener;

#[tokio::main]
async fn main() {
    let udp_listener = UdpListener {};
    udp_listener
        .listen()
        .await
        .map_err(|e| {
            eprintln!("Error: {:?}", e);
        })
        .unwrap();
}
