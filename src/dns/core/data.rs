use std::net::{Ipv4Addr, Ipv6Addr};

use super::record_type::RecordType;

// use super::name::Name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AData {
    address: Ipv4Addr,
}

impl AData {
    pub fn new(address: Ipv4Addr) -> Self {
        Self { address }
    }

    pub fn address(&self) -> Ipv4Addr {
        self.address
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AAAAData {
    address: Ipv6Addr,
}

impl AAAAData {
    pub fn new(address: Ipv6Addr) -> Self {
        Self { address }
    }

    pub fn address(&self) -> Ipv6Addr {
        self.address
    }
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct CnameData {
//     cname: Name,
// }

// impl CnameData {
//     pub fn new(name: Name) -> Self {
//         Self { cname: name }
//     }

//     pub fn cname(&self) -> &Name {
//         &self.cname
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data {
    A(AData),
    AAAA(AAAAData),
    // Cname(CnameData),
    Unknown(RecordType, Vec<u8>),
}
