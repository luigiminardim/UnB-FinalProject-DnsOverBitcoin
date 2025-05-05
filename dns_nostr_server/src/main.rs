use hickory_server::{
    authority::{Authority, Catalog},
    ServerFuture,
};
use lib::{
    dns_nostr_token_repository::DnsNostrTokenRepository, nostr_authority::NostrAuthority,
    nostr_events_repository::NostrEventsRepository,
};
use std::sync::Arc;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    let dns_nostr_token_repository = DnsNostrTokenRepository::new();

    let nostr_events_repository = NostrEventsRepository::new("ws://localhost:8080".to_string());

    let mut handler = Catalog::new();
    let nostr_authority: NostrAuthority = NostrAuthority::new(
        "nostr.dns.name.".parse().unwrap(),
        dns_nostr_token_repository,
        nostr_events_repository,
    );
    handler.upsert(
        nostr_authority.origin().clone(),
        Box::new(Arc::new(nostr_authority)),
    );
    let mut server = ServerFuture::new(handler);
    server.register_socket(UdpSocket::bind("0.0.0.0:1053").await.unwrap());
    server.block_until_done().await.unwrap();
}
