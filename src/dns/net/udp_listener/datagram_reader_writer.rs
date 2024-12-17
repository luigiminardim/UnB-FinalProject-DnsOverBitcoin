use std::{net::Ipv4Addr, str::FromStr};

use crate::dns::core::{
    AData, Class, Data, Label, Message, Name, OpCode, QueryClass, QueryType, Question, Record,
    RecordType, ResponseCode, Ttl,
};

pub(super) struct DatagramReader<'slice> {
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
        let value = self.datagram.get(self.pos)?;
        self.pos += 1;
        Some(value.clone())
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

    fn read_while<F: FnMut(&Self) -> bool>(
        buffer: &mut DatagramReader,
        mut predicate: F,
    ) -> Option<Vec<Self>> {
        std::iter::repeat(0)
            .map(|_| Self::read(buffer))
            .take_while(|maybe_value: &Option<Self>| {
                if let Some(value) = maybe_value {
                    predicate(value)
                } else {
                    false
                }
            })
            .collect()
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
        let u8_0: u8 = ReadableWritable::read(buffer)?;
        let u8_1: u8 = ReadableWritable::read(buffer)?;
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
        let length: u8 = ReadableWritable::read(buffer)?;
        if length == 0 {
            Some(Label::null())
        } else {
            let bytes: Vec<u8> = ReadableWritable::read_all(buffer, length)?;
            let label_str = String::from_utf8(bytes).ok()?;
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

impl ReadableWritable for Name {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let labels: Vec<Label> =
            ReadableWritable::read_while(buffer, |label: &Label| !label.is_null())?;
        let name = Name::create(labels).ok()?;
        Some(name)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        ReadableWritable::write_all(self.labels(), buffer)
    }
}

impl ReadableWritable for QueryType {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let octect: u16 = ReadableWritable::read(buffer)?;
        let query_type = QueryType::from(octect);
        Some(query_type)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let octect: u16 = self.to_owned().into();
        octect.write(buffer)
    }
}

impl ReadableWritable for QueryClass {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let octect: u16 = ReadableWritable::read(buffer)?;
        let query_class = QueryClass::from(octect);
        Some(query_class)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        let octect: u16 = self.to_owned().into();
        octect.write(buffer)
    }
}

impl ReadableWritable for Question {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let name: Name = ReadableWritable::read(buffer)?;
        let query_type: QueryType = ReadableWritable::read(buffer)?;
        let query_class: QueryClass = ReadableWritable::read(buffer)?;
        let question: Question = Question::new(name, query_type, query_class);
        Some(question)
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        self.name().write(buffer)?;
        self.query_type().write(buffer)?;
        self.query_class().write(buffer)
    }
}

impl ReadableWritable for RecordType {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let bytes: u16 = ReadableWritable::read(buffer)?;
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
        let bytes: u16 = ReadableWritable::read(buffer)?;
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
        let u16_1: u16 = ReadableWritable::read(buffer)?;
        let u16_2: u16 = ReadableWritable::read(buffer)?;
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
        let octet_1: u8 = ReadableWritable::read(buffer)?;
        let octet_2: u8 = ReadableWritable::read(buffer)?;
        let octet_3: u8 = ReadableWritable::read(buffer)?;
        let octet_4: u8 = ReadableWritable::read(buffer)?;
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

impl ReadableWritable for Record {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let name: Name = ReadableWritable::read(buffer)?;
        let record_type: RecordType = ReadableWritable::read(buffer)?;
        let class: Class = ReadableWritable::read(buffer)?;
        let ttl: Ttl = ReadableWritable::read(buffer)?;
        let data_length: u16 = ReadableWritable::read(buffer)?;
        let data = match record_type {
            RecordType::A => {
                let ipv4_addr: Ipv4Addr = ReadableWritable::read(buffer)?;
                let a_data = AData::new(ipv4_addr);
                Some(Data::A(a_data))
            }
            RecordType::Unknown(type_value) => {
                let bytes: Vec<u8> = ReadableWritable::read_all(buffer, data_length)?;
                let unknown_data = Data::Unknown(RecordType::Unknown(type_value), bytes);
                Some(unknown_data)
            }
        }?;
        Some(Record::new(name, class, ttl, data))
    }

    fn write(&self, buffer: &mut DatagramWriter) -> Option<()> {
        self.name().write(buffer)?;
        self.record_type().write(buffer)?;
        self.class().write(buffer)?;
        self.ttl().write(buffer)?;
        let data_length: u16 = match self.data() {
            Data::A(_) => 4,
            Data::Unknown(_, bytes) => bytes.len() as u16,
        };
        data_length.write(buffer)?;
        match self.data() {
            Data::A(a_data) => a_data.address().write(buffer),
            Data::Unknown(_, bytes) => bytes
                .iter()
                .map(|byte: &u8| byte.write(buffer))
                .collect::<Option<_>>(),
        }?;
        Some(())
    }
}

impl ReadableWritable for Message {
    fn read(buffer: &mut DatagramReader) -> Option<Self> {
        let bytes_0_1: u16 = ReadableWritable::read(buffer)?;
        let byte2: u8 = ReadableWritable::read(buffer)?;
        let byte3: u8 = ReadableWritable::read(buffer)?;
        let message = Message::new(bytes_0_1)
            .set_is_response((byte2 & 0b1000_0000) != 0)
            .set_opcode(OpCode::from((byte2 & 0b0111_1000) >> 3))
            .set_is_authoritative_answer((byte2 & 0b0000_0100) != 0)
            .set_is_truncation((byte2 & 0b0000_0010) != 0)
            .set_recursion_desired((byte2 & 0b0000_0001) != 0)
            .set_recursion_available((byte3 & 0b1000_0000) != 0)
            .set_response_code(ResponseCode::from(byte3 & 0b0000_1111));
        let question_count: u16 = ReadableWritable::read(buffer)?;
        let answer_count: u16 = ReadableWritable::read(buffer)?;
        let authority_count: u16 = ReadableWritable::read(buffer)?;
        let additional_count: u16 = ReadableWritable::read(buffer)?;
        let questions: Vec<Question> = ReadableWritable::read_all(buffer, question_count)?;
        let answers: Vec<Record> = ReadableWritable::read_all(buffer, answer_count)?;
        let authorities: Vec<Record> = ReadableWritable::read_all(buffer, authority_count)?;
        let additional: Vec<Record> = ReadableWritable::read_all(buffer, additional_count)?;
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
