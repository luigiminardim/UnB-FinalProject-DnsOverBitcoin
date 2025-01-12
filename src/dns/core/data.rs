use super::{record_type::RecordType, Name};
use std::net::Ipv6Addr;

mod a_data;
pub use a_data::AData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsData {
    name_server: Name,
}

impl NsData {
    pub fn new(name_server: Name) -> Self {
        Self { name_server }
    }

    pub fn name_server(&self) -> &Name {
        &self.name_server
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnameData {
    cname: Name,
}

impl CnameData {
    pub fn new(name: Name) -> Self {
        Self { cname: name }
    }

    pub fn cname(&self) -> &Name {
        &self.cname
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MxData {
    /// A 16 bit integer which specifies the preference given to this RR among
    /// others at the same owner.  Lower values are preferred.
    preference: u16,

    /// A <domain-name> which specifies a host willing to act as a mail
    /// exchange for the owner name.
    exchange: Name,
}

impl MxData {
    pub fn new(preference: u16, exchange: Name) -> Self {
        Self {
            preference,
            exchange,
        }
    }

    pub fn preference(&self) -> u16 {
        self.preference
    }

    pub fn exchange(&self) -> &Name {
        &self.exchange
    }
}

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
