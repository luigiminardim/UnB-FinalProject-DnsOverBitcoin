use super::record_type::RecordType;

mod a_data;
pub use a_data::AData;

mod ns_data;
pub use ns_data::NsData;

mod cname_data;
pub use cname_data::CnameData;

mod mx_data;
pub use mx_data::MxData;

mod txt_data;
pub use txt_data::TxtData;

mod aaaa_data;
pub use aaaa_data::AaaaData;

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
