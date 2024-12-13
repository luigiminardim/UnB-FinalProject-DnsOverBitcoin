use lib::dns::{handler::EmptyHandler, net::UdpListener};

#[tokio::main]
async fn main() {
    let udp_listener = UdpListener::new(EmptyHandler::new());
    udp_listener
        .listen()
        .await
        .map_err(|e| {
            eprintln!("Error: {:?}", e);
        })
        .unwrap();
}
