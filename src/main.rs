use std::{
    net::SocketAddr, path::{Path, PathBuf}, str::FromStr, sync::Arc
};

use hickory_server::{
    authority::{Authority, Catalog, LookupError, LookupOptions, MessageRequest, UpdateResult, ZoneType},
    proto::rr::{LowerName, Name, RecordType},
    server::RequestInfo,
    store::{
        file::{FileAuthority, FileConfig},
        recursor::{RecursiveAuthority, RecursiveConfig},
    },
    ServerFuture,
};
use tokio::net::UdpSocket;

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
        catalog.upsert("".parse().unwrap(), Box::new(Arc::new(recursive_authority)));


    let mut server = ServerFuture::new(catalog);
    let socket_address: SocketAddr = "0.0.0.0:1053".parse().unwrap();
    println!("Listening on: {}", socket_address);
    server.register_socket(UdpSocket::bind(socket_address).await.unwrap());
    server.block_until_done().await.expect("block_until_done");
}

struct OrdAuthority {
    origin: LowerName,
}

impl OrdAuthority {
    fn file_authority(&self) -> FileAuthority {
        FileAuthority::try_from_config(
            self.origin.clone().into(),
            ZoneType::Primary,
            false,
            Some(Path::new("./data/zone-files")),
            &FileConfig {
                zone_file_path: "ord.zone".to_string(),
            },
        )
        .unwrap()
    }
}

impl Default for OrdAuthority {
    fn default() -> Self {
        Self { origin: LowerName::from_str("ord").unwrap() }
    }
}

#[async_trait::async_trait]
impl Authority for OrdAuthority {
    type Lookup = <FileAuthority as Authority>::Lookup;

    fn zone_type(&self) -> ZoneType {
        ZoneType::Primary
    }

    fn is_axfr_allowed(&self) -> bool {
        false
    }

    async fn update(&self, _update: &MessageRequest) -> UpdateResult<bool> {
        use hickory_server::proto::op::ResponseCode;
        Err(ResponseCode::NotImp)
    }

    fn origin(&self) -> &LowerName {
        &self.origin
    }

    async fn lookup(
        &self,
        name: &LowerName,
        rtype: RecordType,
        lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        println!("lookup: {:?} {:?} {:?}", name, rtype, lookup_options);
        self.file_authority().lookup(name, rtype, lookup_options).await
    }

    async fn search(
        &self,
        request_info: RequestInfo<'_>,
        lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        println!("search: {:?} {:?}", request_info.query, lookup_options);
        self.file_authority().search(request_info, lookup_options).await
    }

    async fn ns(&self, lookup_options: LookupOptions) -> Result<Self::Lookup, LookupError> {
        println!("ns: {:?}", lookup_options);
        self.file_authority().ns(lookup_options).await
    }

    async fn get_nsec_records(
        &self,
        name: &LowerName,
        lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        println!("get_nsec_records: {:?} {:?}", name, lookup_options);
        self.file_authority().get_nsec_records(name, lookup_options).await
    }

    async fn soa(&self) -> Result<Self::Lookup, LookupError> {
        println!("soa");
        self.file_authority().soa().await
    }

    async fn soa_secure(&self, lookup_options: LookupOptions) -> Result<Self::Lookup, LookupError> {
        println!("soa_secure: {:?}", lookup_options);
        self.file_authority().soa_secure(lookup_options).await
    }
}
