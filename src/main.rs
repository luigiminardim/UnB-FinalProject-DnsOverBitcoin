use std::{path::Path, sync::Arc};

use hickory_server::{
    authority::{Catalog, ZoneType},
    proto::rr::{LowerName, Name},
    store::file::{FileAuthority, FileConfig},
    ServerFuture,
};
use tokio::net;

#[tokio::main]
async fn main() {
    let file_authority = FileAuthority::try_from_config(
        Name::from_ascii("ord").map_err(|e| println!("{}", e)).unwrap(),
        ZoneType::Primary,
        false,
        Some(Path::new("./data/zone-files")),
        &FileConfig {
            zone_file_path: "ord.zone".to_string(),
        },
    )
    .unwrap();

    let mut catalog = Catalog::new();
    catalog.upsert(
        LowerName::new(&Name::from_ascii("ord").unwrap()),
        Box::new(Arc::new(file_authority)),
    );

    let mut server: ServerFuture<Catalog> = ServerFuture::new(catalog);
    let udp_socket = net::UdpSocket::bind("127.0.0.1:1053")
        .await
        .expect("could not bind udp socket");
    server.register_socket(udp_socket);

    server
        .block_until_done()
        .await
        .expect("could not block_until_done");
}
