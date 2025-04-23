use hickory_server::{
    authority::{Authority, Catalog},
    ServerFuture,
};
use lib::nostr_authority::NostrAuthority;
use std::sync::Arc;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    let mut handler = Catalog::new();
    let nostr_authority: NostrAuthority = NostrAuthority::new();
    handler.upsert(
        nostr_authority.origin().clone(),
        Box::new(Arc::new(nostr_authority)),
    );
    let mut server = ServerFuture::new(handler);
    server.register_socket(UdpSocket::bind("0.0.0.0:1053").await.unwrap());
    server.block_until_done().await.unwrap();
}
