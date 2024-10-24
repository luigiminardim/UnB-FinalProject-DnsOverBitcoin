use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use hickory_server::{
    authority::{Catalog, ZoneType},
    proto::rr::Name,
    store::recursor::{RecursiveAuthority, RecursiveConfig},
    ServerFuture,
};
use tokio::net::UdpSocket;

mod ord_authority;

use ord_authority::OrdAuthority;

#[tokio::main]
async fn main() {
    let ordinals_authority = OrdAuthority::default();

    // let forward_authority = ForwardAuthority::new(TokioConnectionProvider::default()).unwrap();

    let recursive_authority = RecursiveAuthority::try_from_config(
        Name::root(),
        ZoneType::Primary,
        &RecursiveConfig {
            roots: PathBuf::from("root.zone"),
        },
        Some(Path::new("./data")),
    )
    .await
    .unwrap();

    let mut catalog = Catalog::new();
    catalog.upsert(
        "ord".parse().unwrap(),
        Box::new(Arc::new(ordinals_authority)),
    );
    // catalog.upsert(Name::root().into(), Box::new(Arc::new(forward_authority)));
    catalog.upsert(Name::root().into(), Box::new(Arc::new(recursive_authority)));

    let mut server = ServerFuture::new(catalog);
    let socket_address: SocketAddr = "0.0.0.0:1053".parse().unwrap();
    println!("Listening on: {}", socket_address);
    server.register_socket(UdpSocket::bind(socket_address).await.unwrap());
    server.block_until_done().await.expect("block_until_done");
}
