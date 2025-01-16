use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use crate::dns::core::{
    AData, AaaaData, Class, CnameData, Data, Label, Message, MxData, Name, NsData, OpCode,
    QueryClass, QueryType, Question, Record, RecordType, ResponseCode, Ttl, TxtData,
};

/// Messages carried by UDP are restricted to 512 bytes.
pub const UDP_LENGTH_LIMIT: usize = 512;

pub struct DatagramReader<'slice> {
    datagram: &'slice [u8],
    pos: usize,
}

impl<'slice> DatagramReader<'slice> {
    pub fn new(datagram: &'slice [u8]) -> Self {
        DatagramReader { datagram, pos: 0 }
    }

    pub fn read_message(mut self) -> Option<Message> {
        ReadableWritable::read(&mut self)
    }

    fn read_u8(&mut self) -> Option<u8> {
        let value = self.datagram.get(self.pos);
        let value = match value {
            Some(value) => value,
            None => {
                eprintln!("Failed to read u8");
                return None;
            }
        };
        self.pos += 1;
        Some(value.clone())
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn set_position(&mut self, offset: usize) {
        self.pos = offset;
    }
}

pub struct DatagramWriter<'slice> {
    buffer: &'slice mut [u8],
    pos: usize,
}

impl<'slice> DatagramWriter<'slice> {
    pub fn new(buffer: &'slice mut [u8]) -> Self {
        DatagramWriter { buffer, pos: 0 }
    }

    pub fn write_message(mut self, message: &Message) -> Option<usize> {
        // If writing the message fails, it means that the buffer is too small.
        // In this case, we set the truncation bit in the message header.
        if let None = ReadableWritable::write(message, &mut self) {
            if let Some(byte_2) = self.buffer.get_mut(2) {
                *byte_2 |= 0b0000_0010;
            }
        }
        Some(self.pos)
    }

    fn write_u8(&mut self, value: u8) -> Option<()> {
        let slice = self.buffer.get_mut(self.pos)?;
        *slice = value;
        self.pos += 1;
        Some(())
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn set_position(&mut self, offset: usize) {
        self.pos = offset;
    }
}

trait ReadableWritable: Sized {
    fn read(buffer: &mut DatagramReader) -> Option<Self>;

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()>;

    fn read_all<CountT: Into<usize>>(
        buffer: &mut DatagramReader,
        count: CountT,
    ) -> Option<Vec<Self>> {
        (0..count.into()).map(|_| Self::read(buffer)).collect()
    }

    fn write_all(vec: &Vec<Self>, buffer: &mut DatagramWriter) -> Option<()> {
        vec.iter().map(|value| value.write(buffer)).collect()
    }
}

impl ReadableWritable for u8 {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        buffer.read_u8()
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        buffer.write_u8(*self)?;
        Some(())
    }
}

impl ReadableWritable for u16 {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let u8_0 = u8::read(buffer)?;
        let u8_1 = u8::read(buffer)?;
        let u16: u16 = ((u8_0 as u16) << 8) | (u8_1 as u16);
        Some(u16)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let u8_0 = (*self >> 8) as u8;
        let u8_1 = *self as u8;
        u8_0.write(buffer)?;
        u8_1.write(buffer)?;
        Some(())
    }
}

impl ReadableWritable for Label {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let length = u8::read(buffer)?;
        if length == 0 {
            Some(Label::null())
        } else {
            let bytes = u8::read_all(buffer, length)?;
            let label_str = match String::from_utf8(bytes) {
                Ok(label_str) => label_str,
                Err(err) => {
                    eprintln!("Failed to parse label: {:?}", err);
                    return None;
                }
            };
            let label = Label::from_str(&label_str).ok()?;
            Some(label)
        }
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let length = self.len() as u8;
        length.write(buffer)?;
        self.to_string()
            .bytes()
            .map(|byte: u8| byte.write(buffer))
            .collect::<Option<_>>()?;
        Some(())
    }
}

#[cfg(test)]
mod test_label {
    use super::*;

    #[test]
    fn test_read() {
        let buffer = [3, b'f', b'o', b'o'];
        let label = Label::read(&mut DatagramReader::new(&buffer)).unwrap();
        assert_eq!(label, "foo".parse().unwrap());
    }

    #[test]
    fn test_write() {
        let label = Label::from_str("foo").unwrap();
        let mut buffer = [0; UDP_LENGTH_LIMIT];
        let mut writer = DatagramWriter::new(&mut buffer);
        label.write(&mut writer).unwrap();
        assert_eq!(writer.pos, 4);
        assert_eq!(buffer[0..4], [3, b'f', b'o', b'o']);
    }
}

impl ReadableWritable for Name {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let first_label_flag = {
            let current_pos = buffer.position();
            let flag = u8::read(buffer)? & 0b1100_0000;
            buffer.set_position(current_pos);
            flag
        };
        // invalid flag
        if first_label_flag == 0b1000_0000 || first_label_flag == 0b0100_0000 {
            eprintln!("Invalid label flag: {}", first_label_flag);
            None
        }
        // compression flag
        else if first_label_flag == 0b1100_0000 {
            let current_pos = buffer.position();
            let offset = (u16::read(buffer)? & 0b0011_1111_1111_1111) as usize;
            let next_pos = buffer.position();
            if offset >= current_pos {
                eprintln!(
                    "Invalid name compression offset({}) >= current position({})",
                    offset, current_pos
                );
                return None;
            }
            buffer.set_position(offset as usize);
            let super_name = Name::read(buffer)?;
            buffer.set_position(next_pos);
            Some(super_name)
        } else {
            let first_label = Label::read(buffer)?;
            if first_label == Label::null() {
                return Some(Name::root());
            }
            let super_name = Name::read(buffer)?;
            let labels = [first_label]
                .iter()
                .chain(super_name.labels().iter())
                .cloned()
                .collect();
            Name::create(labels).ok()
        }
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        ReadableWritable::write_all(self.labels(), buffer)?;
        ReadableWritable::write(&Label::null(), buffer)
    }
}

#[cfg(test)]
mod test_name {
    use super::*;

    #[test]
    fn test_read_root() {
        let datagram = [0];
        let mut buffer = DatagramReader::new(&datagram);
        let name = Name::read(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 1);
        assert_eq!(name, Name::root());
    }

    #[test]
    fn test_read_simple() {
        let datagram = [
            3, b'f', b'o', b'o', // pos = 0; "foo"
            0,    // pos = 4; ""
        ];
        let mut buffer = DatagramReader::new(&datagram);
        let name = Name::read(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 5);
        assert_eq!(name, "foo.".parse().unwrap());
    }

    #[test]
    fn test_read_double() {
        let buffer = [
            3, b'f', b'o', b'o', // pos = 0; "foo"
            3, b'b', b'a', b'r', // pos = 4; "bar"
            0,    // pos = 8; ""
        ];
        let mut reader = DatagramReader::new(&buffer);
        let name = Name::read(&mut reader).unwrap();
        assert_eq!(reader.pos, 9);
        assert_eq!(name, "foo.bar.".parse().unwrap());
    }

    #[test]
    fn test_read_compression() {
        let buffer = [
            // pos = 0; "foo."
            3,
            b'f',
            b'o',
            b'o',
            0,
            // pos = 5; "bar->foo."
            3,
            b'b',
            b'a',
            b'r',
            0b1100_0000 | 0,
            0,
            // pos = 11; "zar->bar->foo."
            3,
            b'z',
            b'a',
            b'r',
            0b1100_0000 | 0,
            5,
        ];
        let mut buffer = DatagramReader::new(&buffer);
        let foo_name = Name::read(&mut buffer).unwrap();
        let bar_name = Name::read(&mut buffer).unwrap();
        let zar_name = Name::read(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 17);
        assert_eq!(foo_name, "foo.".parse().unwrap());
        assert_eq!(bar_name, "bar.foo.".parse().unwrap());
        assert_eq!(zar_name, "zar.bar.foo.".parse().unwrap());
    }

    #[test]
    fn test_write_root() {
        let name = Name::root();
        let mut buffer = [0; UDP_LENGTH_LIMIT];
        let mut writer = DatagramWriter::new(&mut buffer);
        name.write(&mut writer).unwrap();
        assert_eq!(writer.pos, 1);
        assert_eq!(buffer[0], 0);
    }

    #[test]
    fn test_write_simple() {
        let name = "foo.".parse::<Name>().unwrap();
        let mut buffer = [0; UDP_LENGTH_LIMIT];
        let mut writer = DatagramWriter::new(&mut buffer);
        name.write(&mut writer).unwrap();
        assert_eq!(writer.pos, 5);
        assert_eq!(buffer[0..5], [3, b'f', b'o', b'o', 0]);
    }

    #[test]
    fn test_write_double() {
        let name = "foo.bar.".parse::<Name>().unwrap();
        let mut buffer = [0; UDP_LENGTH_LIMIT];
        let mut writer = DatagramWriter::new(&mut buffer);
        name.write(&mut writer).unwrap();
        assert_eq!(writer.pos, 9);
        assert_eq!(buffer[0..9], [3, b'f', b'o', b'o', 3, b'b', b'a', b'r', 0]);
    }
}

impl ReadableWritable for QueryType {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let octect = u16::read(buffer)?;
        let query_type = QueryType::from(octect);
        Some(query_type)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let octect: u16 = self.to_owned().into();
        octect.write(buffer)
    }
}

#[cfg(test)]
mod test_query_type {
    use super::*;

    #[test]
    fn test_read_type() {
        let datagram = [0x00, 0x01];
        let mut buffer = DatagramReader::new(&datagram);
        let query_type = QueryType::read(&mut buffer).unwrap();
        assert_eq!(query_type, QueryType::Type(RecordType::A));
        assert_eq!(buffer.pos, 2);
    }

    #[test]
    fn test_write_type() {
        let query_type = QueryType::Type(RecordType::A);
        let mut datagram = [0; UDP_LENGTH_LIMIT];
        let mut buffer = DatagramWriter::new(&mut datagram);
        query_type.write(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 2);
        assert_eq!(datagram[0..2], [0x00, 0x01]);
    }
}

impl ReadableWritable for QueryClass {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let octect = u16::read(buffer)?;
        let query_class = QueryClass::from(octect);
        Some(query_class)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let octect: u16 = self.to_owned().into();
        octect.write(buffer)
    }
}

#[cfg(test)]
mod test_query_class {
    use super::*;

    #[test]
    fn test_read_class() {
        let datagram = [0x00, 0x01];
        let mut buffer = DatagramReader::new(&datagram);
        let query_class = QueryClass::read(&mut buffer).unwrap();
        assert_eq!(query_class, QueryClass::Class(Class::In));
        assert_eq!(buffer.pos, 2);
    }

    #[test]
    fn test_write_class() {
        let query_class = QueryClass::Class(Class::In);
        let mut datagram = [0; UDP_LENGTH_LIMIT];
        let mut buffer = DatagramWriter::new(&mut datagram);
        query_class.write(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 2);
        assert_eq!(datagram[0..2], [0x00, 0x01]);
    }
}

impl ReadableWritable for Question {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let name = Name::read(buffer)?;
        let query_type = QueryType::read(buffer)?;
        let query_class = QueryClass::read(buffer)?;
        let question = Question::new(name, query_type, query_class);
        Some(question)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        self.name().write(buffer)?;
        self.query_type().write(buffer)?;
        self.query_class().write(buffer)
    }
}

#[cfg(test)]
mod test_question {
    use super::*;

    #[test]
    fn test_read_question() {
        let datagram = [
            0x03, 0x63, 0x6F, 0x6D, 0x00, // "com."
            0x00, 0x01, // A
            0x00, 0x01, // IN
        ];
        let mut buffer = DatagramReader::new(&datagram);
        let question = Question::read(&mut buffer).unwrap();
        assert_eq!(question.name(), &"com.".parse::<Name>().unwrap());
        assert_eq!(question.query_type(), QueryType::Type(RecordType::A));
        assert_eq!(question.query_class(), QueryClass::Class(Class::In));
        assert_eq!(buffer.pos, 9);
    }

    #[test]
    fn test_write() {
        let question = Question::new(
            "com.".parse().unwrap(),
            QueryType::Type(RecordType::A),
            QueryClass::Class(Class::In),
        );
        let mut datagram = [0; UDP_LENGTH_LIMIT];
        let mut buffer = DatagramWriter::new(&mut datagram);
        question.write(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 9);
        assert_eq!(
            datagram[0..9],
            [
                0x03, 0x63, 0x6F, 0x6D, 0x00, // "com."
                0x00, 0x01, // A
                0x00, 0x01 // IN
            ]
        );
    }
}

impl ReadableWritable for RecordType {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let bytes = u16::read(buffer)?;
        let record_type = RecordType::from(bytes);
        Some(record_type)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let bytes: u16 = self.to_owned().into();
        bytes.write(buffer)
    }
}

impl ReadableWritable for Class {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let bytes = u16::read(buffer)?;
        let class = Class::from(bytes);
        Some(class)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let bytes: u16 = self.to_owned().into();
        bytes.write(buffer)
    }
}

impl ReadableWritable for u32 {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let u16_1 = u16::read(buffer)?;
        let u16_2 = u16::read(buffer)?;
        let u32 = ((u16_1 as u32) << 16) | (u16_2 as u32);
        Some(u32)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let u16_1 = (*self >> 16) as u16;
        let u16_2 = *self as u16;
        u16_1.write(buffer)?;
        u16_2.write(buffer)
    }
}

impl ReadableWritable for Ipv4Addr {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let octet_1 = u8::read(buffer)?;
        let octet_2 = u8::read(buffer)?;
        let octet_3 = u8::read(buffer)?;
        let octet_4 = u8::read(buffer)?;
        let ipv4_addr = Ipv4Addr::new(octet_1, octet_2, octet_3, octet_4);
        Some(ipv4_addr)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let [octect_1, octect_2, octect_3, octect_4] = self.octets();
        octect_1.write(buffer)?;
        octect_2.write(buffer)?;
        octect_3.write(buffer)?;
        octect_4.write(buffer)
    }
}

impl ReadableWritable for AData {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let ipv4_addr = Ipv4Addr::read(buffer)?;
        let a_data = AData::new(ipv4_addr);
        Some(a_data)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        self.address().write(buffer)
    }
}

#[cfg(test)]
mod test_adata {
    use super::*;

    #[test]
    fn test_read() {
        let datagram = [192, 12, 0, 61];
        let mut buffer = DatagramReader::new(&datagram);
        let a_data = AData::read(&mut buffer).unwrap();
        assert_eq!(a_data.address(), Ipv4Addr::new(192, 12, 0, 61));
        assert_eq!(buffer.pos, 4);
    }

    #[test]
    fn test_write() {
        let a_data = AData::new(Ipv4Addr::new(192, 12, 0, 61));
        let mut datagram = [0; UDP_LENGTH_LIMIT];
        let mut buffer = DatagramWriter::new(&mut datagram);
        a_data.write(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 4);
        assert_eq!(datagram[0..4], [192, 12, 0, 61]);
    }
}

impl ReadableWritable for NsData {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let name = Name::read(buffer)?;
        let ns_data = NsData::new(name);
        Some(ns_data)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        self.name_server().write(buffer)
    }
}

impl ReadableWritable for AaaaData {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let segment_0 = u16::read(buffer)?;
        let segment_1 = u16::read(buffer)?;
        let segment_2 = u16::read(buffer)?;
        let segment_3 = u16::read(buffer)?;
        let segment_4 = u16::read(buffer)?;
        let segment_5 = u16::read(buffer)?;
        let segment_6 = u16::read(buffer)?;
        let segment_7 = u16::read(buffer)?;
        let ipv6_addr = Ipv6Addr::new(
            segment_0, segment_1, segment_2, segment_3, segment_4, segment_5, segment_6, segment_7,
        );
        let aaaa_data = AaaaData::new(ipv6_addr);
        Some(aaaa_data)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let segments = self.address().segments();
        segments
            .iter()
            .map(|segment| segment.write(buffer))
            .collect::<Option<_>>()
    }
}

#[cfg(test)]
mod test_aaaa_data {

    use super::*;

    #[test]
    fn test_read() {
        let datagram = [
            0x26, 0x06, 0x28, 0x00, 0x02, 0x1f, 0xcb, 0x07, 0x68, 0x20, 0x80, 0xda, 0xaf, 0x6b,
            0x8b, 0x2c,
        ];
        let mut buffer = DatagramReader::new(&datagram);
        let aaaa_data = AaaaData::read(&mut buffer).unwrap();
        assert_eq!(
            aaaa_data.address(),
            Ipv6Addr::new(0x2606, 0x2800, 0x021f, 0xcb07, 0x6820, 0x80da, 0xaf6b, 0x8b2c)
        );
        assert_eq!(buffer.pos, 16);
    }

    #[test]
    fn test_write() {
        let aaaa_data = AaaaData::new(Ipv6Addr::new(
            0x2606, 0x2800, 0x021f, 0xcb07, 0x6820, 0x80da, 0xaf6b, 0x8b2c,
        ));
        let mut datagram = [0; UDP_LENGTH_LIMIT];
        let mut buffer = DatagramWriter::new(&mut datagram);
        aaaa_data.write(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 16);
        assert_eq!(
            datagram[0..16],
            [
                0x26, 0x06, 0x28, 0x00, 0x02, 0x1f, 0xcb, 0x07, 0x68, 0x20, 0x80, 0xda, 0xaf, 0x6b,
                0x8b, 0x2c,
            ]
        );
    }
}

impl ReadableWritable for CnameData {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let name = Name::read(buffer)?;
        let cname_data = CnameData::new(name);
        Some(cname_data)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        self.cname().write(buffer)
    }
}

#[cfg(test)]
mod test_cname_data {
    use super::*;

    #[test]
    fn test_read() {
        let datagram = [3, b'f', b'o', b'o', 0];
        let mut buffer = DatagramReader::new(&datagram);
        let cname_data = CnameData::read(&mut buffer).unwrap();
        assert_eq!(cname_data.cname(), &"foo.".parse::<Name>().unwrap());
        assert_eq!(buffer.pos, 5);
    }

    #[test]
    fn test_write() {
        let cname_data = CnameData::new("foo.".parse().unwrap());
        let mut datagram = [0; UDP_LENGTH_LIMIT];
        let mut buffer = DatagramWriter::new(&mut datagram);
        cname_data.write(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 5);
        assert_eq!(datagram[0..5], [3, b'f', b'o', b'o', 0]);
    }
}

impl ReadableWritable for MxData {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let preference = u16::read(buffer)?;
        let exchange = Name::read(buffer)?;
        let mx_data = MxData::new(preference, exchange);
        Some(mx_data)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        self.preference().write(buffer)?;
        self.exchange().write(buffer)
    }
}

#[cfg(test)]
mod test_mx_data {
    use super::*;

    #[test]
    fn test_read() {
        let datagram = [
            0x00, 0x0a, // preference = 10
            3, b'f', b'o', b'o', 0, // exchange = "foo."
        ];
        let mut buffer = DatagramReader::new(&datagram);
        let mx_data = MxData::read(&mut buffer).unwrap();
        assert_eq!(mx_data.preference(), 0x000a);
        assert_eq!(mx_data.exchange(), &"foo.".parse::<Name>().unwrap());
        assert_eq!(buffer.pos, 7);
    }

    #[test]
    fn test_write() {
        let mx_data = MxData::new(10, "foo.".parse().unwrap());
        let mut datagram = [0; UDP_LENGTH_LIMIT];
        let mut buffer = DatagramWriter::new(&mut datagram);
        mx_data.write(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 7);
        assert_eq!(
            datagram[0..7],
            [
                0x00, 0x0a, // preference = 10
                3, b'f', b'o', b'o', 0, // exchange = "foo."
            ]
        );
    }
}

impl ReadableWritable for Record {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let name = Name::read(buffer)?;
        let record_type = RecordType::read(buffer)?;
        let class = Class::read(buffer)?;
        let ttl = Ttl::read(buffer)?;
        let data_length = u16::read(buffer)?;
        let data = match record_type {
            RecordType::A => AData::read(buffer).map(Data::A),
            RecordType::Ns => NsData::read(buffer).map(Data::Ns),
            RecordType::Cname => CnameData::read(buffer).map(Data::Cname),
            RecordType::Aaaa => AaaaData::read(buffer).map(Data::Aaaa),
            RecordType::Mx => MxData::read(buffer).map(Data::Mx),
            RecordType::Txt => {
                let bytes = u8::read_all(buffer, data_length)?;
                let string = String::from_utf8(bytes).ok()?;
                let txt_data = TxtData::new(string);
                let data = Data::Txt(txt_data);
                Some(data)
            }
            RecordType::Unknown(type_value) => {
                let bytes = u8::read_all(buffer, data_length)?;
                let unknown_data = Data::Unknown(RecordType::Unknown(type_value), bytes);
                Some(unknown_data)
            }
        }?;
        Some(Record::new(name, class, ttl, data))
    }

    fn write(&self, mut buffer: &mut DatagramWriter) -> Option<()> {
        self.name().write(buffer)?;
        self.record_type().write(buffer)?;
        self.class().write(buffer)?;
        self.ttl().write(buffer)?;
        let data_length_pos = buffer.position();
        // write data length as 0 for now
        u16::write(&0, buffer)?;
        let data_start_pos = buffer.position();
        match self.data() {
            Data::A(a_data) => a_data.write(buffer),
            Data::Ns(ns_data) => ns_data.write(buffer),
            Data::Cname(cname_data) => cname_data.write(buffer),
            Data::Aaaa(aaaa_data) => aaaa_data.write(buffer),
            Data::Mx(mx_data) => mx_data.write(buffer),
            Data::Txt(txt_data) => {
                let bytes = txt_data.text().as_bytes();
                bytes
                    .iter()
                    .map(|byte: &u8| byte.write(buffer))
                    .collect::<Option<_>>()
            }
            Data::Unknown(_, bytes) => bytes
                .iter()
                .map(|byte: &u8| byte.write(buffer))
                .collect::<Option<_>>(),
        }?;
        let data_end_pos = buffer.position();
        let data_length = (data_end_pos - data_start_pos) as u16;
        buffer.set_position(data_length_pos);
        data_length.write(&mut buffer)?;
        buffer.set_position(data_end_pos);
        Some(())
    }
}

#[cfg(test)]
mod test_record {
    use super::*;

    #[test]
    fn test_read_txt() {
        let datagram = [
            3, b'f', b'o', b'o', 0, // "foo."
            0, 0x10, // TXT
            0, 0x01, // IN
            0, 0, 0, 0, // TTL
            0, 3, // data length
            b'b', b'a', b'r', // "bar"
        ];
        let mut buffer = DatagramReader::new(&datagram);
        let record = Record::read(&mut buffer).unwrap();
        assert_eq!(record.name(), &"foo.".parse::<Name>().unwrap());
        assert_eq!(record.record_type(), RecordType::Txt);
        assert_eq!(record.class(), Class::In);
        assert_eq!(record.ttl(), 0 as Ttl);
        assert_eq!(record.data(), &Data::Txt(TxtData::new("bar".to_string())));
    }

    #[test]
    fn test_write_txt() {
        let record = Record::new(
            "foo.".parse().unwrap(),
            Class::In,
            0 as Ttl,
            Data::Txt(TxtData::new("bar".to_string())),
        );
        let mut datagram = [0; UDP_LENGTH_LIMIT];
        let mut buffer = DatagramWriter::new(&mut datagram);
        record.write(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 18);
        assert_eq!(
            datagram[0..18],
            [
                3, b'f', b'o', b'o', 0, // "foo."
                0, 0x10, // TXT
                0, 0x01, // IN
                0, 0, 0, 0, // TTL
                0, 3, // data length
                b'b', b'a', b'r', // "bar"
            ]
        );
    }

    #[test]
    fn test_read_unknown() {
        let datagram = [
            3, b'f', b'o', b'o', 0, // "foo."
            0, 0x03, // MD
            0, 0x01, // IN
            0, 0, 0, 0, // TTL
            0, 5, // data length
            3, b'b', b'a', b'r', 0, // "bar."
        ];
        let mut buffer = DatagramReader::new(&datagram);
        let record = Record::read(&mut buffer).unwrap();
        assert_eq!(record.name(), &"foo.".parse::<Name>().unwrap());
        assert_eq!(record.record_type(), RecordType::Unknown(0x03));
        assert_eq!(record.class(), Class::In);
        assert_eq!(record.ttl(), 0 as Ttl);
        assert_eq!(
            record.data(),
            &Data::Unknown(RecordType::Unknown(0x03), vec![3, b'b', b'a', b'r', 0])
        );
    }

    #[test]
    fn test_write_unknown() {
        let record = Record::new(
            "foo.".parse().unwrap(),
            Class::In,
            0 as Ttl,
            Data::Unknown(RecordType::Unknown(0x03), vec![3, b'b', b'a', b'r', 0]),
        );
        let mut datagram = [0; UDP_LENGTH_LIMIT];
        let mut buffer = DatagramWriter::new(&mut datagram);
        record.write(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 20);
        assert_eq!(
            datagram[0..20],
            [
                3, b'f', b'o', b'o', 0, // "foo."
                0, 0x03, // MD
                0, 0x01, // IN
                0, 0, 0, 0, // TTL
                0, 5, // data length
                3, b'b', b'a', b'r', 0, // "bar."
            ]
        );
    }
}

impl ReadableWritable for Message {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let bytes_0_1 = u16::read(buffer)?;
        let byte2 = u8::read(buffer)?;
        let byte3 = u8::read(buffer)?;
        let message = Message::new(bytes_0_1)
            .set_is_response((byte2 & 0b1000_0000) != 0)
            .set_opcode(OpCode::from((byte2 & 0b0111_1000) >> 3))
            .set_is_authoritative_answer((byte2 & 0b0000_0100) != 0)
            .set_is_truncation((byte2 & 0b0000_0010) != 0)
            .set_recursion_desired((byte2 & 0b0000_0001) != 0)
            .set_recursion_available((byte3 & 0b1000_0000) != 0)
            .set_response_code(ResponseCode::from(byte3 & 0b0000_1111));
        let question_count = u16::read(buffer)?;
        let answer_count = u16::read(buffer)?;
        let authority_count = u16::read(buffer)?;
        let additional_count = u16::read(buffer)?;
        let questions = Question::read_all(buffer, question_count)?;
        let answers = Record::read_all(buffer, answer_count)?;
        let authorities = Record::read_all(buffer, authority_count)?;
        let additional = Record::read_all(buffer, additional_count)?;
        let message = message
            .set_questions(questions)
            .set_answers(answers)
            .set_authorities(authorities)
            .set_additional(additional);
        Some(message)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let bytes_0_1: u16 = self.id();
        let byte2: u8 = {
            let mut byte = 0;
            if self.is_response() {
                byte |= 0b1000_0000;
            }
            byte |= (Into::<u8>::into(self.opcode())) << 3;
            if self.is_authoritative_answer() {
                byte |= 0b0000_0100;
            }
            if self.is_truncation() {
                byte |= 0b0000_0010;
            }
            if self.recursion_desired() {
                byte |= 0b0000_0001;
            }
            byte
        };
        let byte3: u8 = {
            let mut byte = 0;
            if self.recursion_available() {
                byte |= 0b1000_0000;
            }
            byte |= Into::<u8>::into(self.response_code());
            byte
        };
        bytes_0_1.write(buffer)?;
        byte2.write(buffer)?;
        byte3.write(buffer)?;
        (self.questions().len() as u16).write(buffer)?;
        (self.answers().len() as u16).write(buffer)?;
        (self.authorities().len() as u16).write(buffer)?;
        (self.additional().len() as u16).write(buffer)?;
        ReadableWritable::write_all(self.questions(), buffer)?;
        ReadableWritable::write_all(self.answers(), buffer)?;
        ReadableWritable::write_all(self.authorities(), buffer)?;
        ReadableWritable::write_all(self.additional(), buffer)
    }
}

#[cfg(test)]
mod test_message {
    use super::*;

    #[test]
    fn test_read_header() {
        let datagram = [
            0x6A as u8, 0x20, // id = 0x6A20
            0x81, // QR = 1, Opcode = 0, AA = 0, TC = 0, RD = 1
            0x80, // RA = 1, Z = 0, RCODE = 0
            0x00, 0x00, // QDCOUNT = 0
            0x00, 0x00, // ANCOUNT = 0
            0x00, 0x00, // NSCOUNT = 0
            0x00, 0x00, // ARCOUNT = 0
        ];
        let mut buffer = DatagramReader::new(&datagram);
        let message = Message::read(&mut buffer).unwrap();
        assert_eq!(message.id(), 0x6A20);
        assert_eq!(message.is_response(), true);
        assert_eq!(message.opcode(), OpCode::Query);
        assert_eq!(message.is_authoritative_answer(), false);
        assert_eq!(message.is_truncation(), false);
        assert_eq!(message.recursion_desired(), true);
        assert_eq!(message.recursion_available(), true);
        assert_eq!(message.response_code(), ResponseCode::NoError);
        assert_eq!(message.questions().len(), 0);
        assert_eq!(message.answers().len(), 0);
        assert_eq!(message.authorities().len(), 0);
        assert_eq!(message.additional().len(), 0);
        assert_eq!(buffer.pos, 12);
    }

    #[test]
    fn test_write() {
        let message = Message::new(0x6A20)
            .set_is_response(true)
            .set_opcode(OpCode::Query)
            .set_is_authoritative_answer(false)
            .set_is_truncation(false)
            .set_recursion_desired(true)
            .set_recursion_available(true)
            .set_response_code(ResponseCode::NoError);
        let mut datagram = [0; UDP_LENGTH_LIMIT];
        let mut buffer = DatagramWriter::new(&mut datagram);
        message.write(&mut buffer).unwrap();
        assert_eq!(buffer.pos, 12);
        assert_eq!(
            datagram[0..12],
            [
                0x6A as u8, 0x20, // id = 0x6A20
                0x81, // QR = 1, Opcode = 0, AA = 0, TC = 0, RD = 1
                0x80, // RA = 1, Z = 0, RCODE = 0
                0x00, 0x00, // QDCOUNT = 0
                0x00, 0x00, // ANCOUNT = 0
                0x00, 0x00, // NSCOUNT = 0
                0x00, 0x00, // ARCOUNT = 0
            ]
        );
    }

    #[test]
    fn test_read() {
        let buffer: [u8; 105] = [
            106, 32, 129, 128, 0, 1, 0, 0, 0, 1, 0, 1, // id = 0x6A20
            3, 99, 111, 109, 0, 0, 1, 0, 1, 192, 12, 0, 6, 0, 1, 0, 0, 3, 132, 0, 61, 1, 97, 12,
            103, 116, 108, 100, 45, 115, 101, 114, 118, 101, 114, 115, 3, 110, 101, 116, 0, 5, 110,
            115, 116, 108, 100, 12, 118, 101, 114, 105, 115, 105, 103, 110, 45, 103, 114, 115, 192,
            12, 103, 98, 9, 231, 0, 0, 7, 8, 0, 0, 3, 132, 0, 9, 58, 128, 0, 0, 3, 132, 0, 0, 41,
            4, 208, 0, 0, 0, 0, 0, 0,
        ];
        let message = Message::read(&mut DatagramReader::new(&buffer)).unwrap();
        assert_eq!(message.id(), 0x6A20);
    }
}
