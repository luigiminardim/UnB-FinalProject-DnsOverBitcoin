use super::record_type::RecordType;
use std::net::Ipv6Addr;

mod a_data;
pub use a_data::AData;

mod ns_data;
pub use ns_data::NsData;

mod cname_data;
pub use cname_data::CnameData;

mod mx_data;
pub use mx_data::MxData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxtData {
    text: String,
}

impl TxtData {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AaaaData {
    address: Ipv6Addr,
}

impl AaaaData {
    pub fn new(address: Ipv6Addr) -> Self {
        Self { address }
    }

    pub fn address(&self) -> Ipv6Addr {
        self.address
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data {
    A(AData),
    Ns(NsData),
    Aaaa(AaaaData),
    Cname(CnameData),
    Mx(MxData),
    Txt(TxtData),
    Unknown(RecordType, Vec<u8>),
}
