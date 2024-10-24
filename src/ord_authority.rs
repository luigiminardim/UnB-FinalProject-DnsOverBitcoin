use std::{
    path::Path,
    str::FromStr,
};

use hickory_server::{
    authority::{
        Authority, LookupError, LookupOptions, MessageRequest, UpdateResult, ZoneType,
    },
    proto::rr::{LowerName, RecordType},
    server::RequestInfo,
    store::
        file::{FileAuthority, FileConfig}
    ,
};

pub struct OrdAuthority {
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
        Self {
            origin: LowerName::from_str("ord").unwrap(),
        }
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
        self.file_authority()
            .lookup(name, rtype, lookup_options)
            .await
    }

    async fn search(
        &self,
        request_info: RequestInfo<'_>,
        lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        println!("search: {:?} {:?}", request_info.query, lookup_options);
        self.file_authority()
            .search(request_info, lookup_options)
            .await
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
        self.file_authority()
            .get_nsec_records(name, lookup_options)
            .await
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
