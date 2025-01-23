use hickory_server::{
    authority::{
        Authority, Catalog, LookupError, LookupOptions, MessageRequest, UpdateResult, ZoneType,
    },
    proto::{
        op::ResponseCode,
        rr::{LowerName, Name, RecordType},
        serialize::txt::Parser,
    },
    server::RequestInfo,
    store::in_memory::InMemoryAuthority,
    ServerFuture,
};
use std::{path::PathBuf, sync::Arc};
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

pub struct NostrAuthority {
    nostr_root: LowerName,
}

#[derive(Debug, PartialEq, Eq)]
struct NameInfo {
    token: LowerName,

    /// {token}.{root}
    zone: LowerName,
}

impl NostrAuthority {
    pub fn new() -> Self {
        Self {
            nostr_root: "nostr.dns.name.".parse().unwrap(),
        }
    }

    async fn create_zone_authority(&self, name: &LowerName) -> Option<InMemoryAuthority> {
        let NameInfo { token, zone } = self.extract_name_info(name)?;
        let filename = dbg!(self.token_to_filename(&token));
        let buf = String::from_utf8(tokio::fs::read(&filename).await.ok()?).ok()?;
        let (_, records) = Parser::new(buf, None, Some(name.base_name().into()))
            .parse()
            .inspect_err(|e| {
                format!("failed to parse {}: {:?}", filename.to_string_lossy(), e);
            })
            .ok()?;
        let authority =
            InMemoryAuthority::new(zone.clone().into(), records, ZoneType::Primary, false)
                .inspect_err(|e| {
                    format!(
                        "failed to create authority for {}: {:?}",
                        zone.to_string(),
                        e
                    );
                })
                .ok()?;
        Some(authority)
    }

    fn extract_name_info(&self, name: &LowerName) -> Option<NameInfo> {
        // name should be <{subdomain}.>{token}.{root}
        if !self.origin().zone_of(name) || name.num_labels() < self.origin().num_labels() + 1 {
            return None;
        }
        let zone = Name::from_labels(
            Name::from(name)
                .iter()
                .skip((name.num_labels() - self.origin().num_labels()).into())
                .collect::<Vec<_>>(),
        )
        .ok()?;
        let token = Name::from_labels(
            Name::from(name)
                .iter()
                .skip((name.num_labels() - self.origin().num_labels() - 1).into())
                .next(),
        )
        .ok()?;
        Some(NameInfo {
            token: token.into(),
            zone: zone.into(),
        })
    }

    fn token_to_filename(&self, token: &LowerName) -> PathBuf {
        let mut copy: Name = token.clone().into();
        copy.set_fqdn(false);
        PathBuf::from(format!("./data/zone-files/{copy}.zone"))
    }
}

#[async_trait::async_trait]
impl Authority for NostrAuthority {
    /// Result of a lookup
    type Lookup = <InMemoryAuthority as Authority>::Lookup;

    fn zone_type(&self) -> ZoneType {
        ZoneType::Primary
    }

    fn is_axfr_allowed(&self) -> bool {
        false
    }

    async fn update(&self, _update: &MessageRequest) -> UpdateResult<bool> {
        Err(ResponseCode::NotImp)
    }

    fn origin(&self) -> &LowerName {
        &self.nostr_root
    }

    async fn lookup(
        &self,
        name: &LowerName,
        rtype: RecordType,
        lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        let authority = self
            .create_zone_authority(name)
            .await
            .ok_or_else(|| LookupError::from(ResponseCode::NXDomain))?;
        authority.lookup(name, rtype, lookup_options).await
    }

    /// Using the specified query, perform a lookup against this zone.
    ///
    /// # Arguments
    ///
    /// * `query` - the query to perform the lookup with.
    /// * `is_secure` - if true, then RRSIG records (if this is a secure zone) will be returned.
    ///
    /// # Return value
    ///
    /// Returns a vector containing the results of the query, it will be empty if not found. If
    ///  `is_secure` is true, in the case of no records found then NSEC records will be returned.
    async fn search(
        &self,
        request: RequestInfo<'_>,
        lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        let authority = self
            .create_zone_authority(request.query.name())
            .await
            .ok_or_else(|| LookupError::from(ResponseCode::NXDomain))?;
        authority.search(request, lookup_options).await
    }

    /// Get the NS, NameServer, record for the zone
    async fn ns(&self, lookup_options: LookupOptions) -> Result<Self::Lookup, LookupError> {
        self.lookup(self.origin(), RecordType::NS, lookup_options)
            .await
    }

    /// Return the NSEC records based on the given name
    ///
    /// # Arguments
    ///
    /// * `name` - given this name (i.e. the lookup name), return the NSEC record that is less than
    ///            this
    /// * `is_secure` - if true then it will return RRSIG records as well
    async fn get_nsec_records(
        &self,
        _: &LowerName,
        _: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        Result::Err(LookupError::ResponseCode(ResponseCode::NotImp))
    }

    /// Returns the SOA of the authority.
    ///
    /// *Note*: This will only return the SOA, if this is fulfilling a request, a standard lookup
    ///  should be used, see `soa_secure()`, which will optionally return RRSIGs.
    async fn soa(&self) -> Result<Self::Lookup, LookupError> {
        // SOA should be origin|SOA
        self.lookup(self.origin(), RecordType::SOA, LookupOptions::default())
            .await
    }

    /// Returns the SOA record for the zone
    async fn soa_secure(&self, lookup_options: LookupOptions) -> Result<Self::Lookup, LookupError> {
        self.lookup(self.origin(), RecordType::SOA, lookup_options)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token() {
        let authority = NostrAuthority::new();

        let name = "token.nostr.dns.name".parse().unwrap();
        let token = authority.extract_name_info(&name);
        assert_eq!(
            token,
            Some(NameInfo {
                token: "token".parse().unwrap(),
                zone: "nostr.dns.name".parse().unwrap(),
            })
        );

        let name = "subdomain.token.nostr.dns.name.".parse().unwrap();
        let token = authority.extract_name_info(&name);
        assert_eq!(
            token,
            Some(NameInfo {
                token: "token".parse().unwrap(),
                zone: "nostr.dns.name".parse().unwrap(),
            })
        );

        let name = "nostr.dns.name.".parse().unwrap();
        let token = authority.extract_name_info(&name);
        assert_eq!(token, None);
    }

    #[test]
    fn test_token_to_filename() {
        let authority = NostrAuthority::new();

        assert_eq!(
            authority.token_to_filename(&"token".parse().unwrap()),
            PathBuf::from("./data/zone-files/token.zone")
        );

        assert_eq!(
            authority.token_to_filename(&"token.".parse().unwrap()),
            PathBuf::from("./data/zone-files/token.zone")
        );
    }
}
