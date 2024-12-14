use std::{net::Ipv4Addr, str::FromStr};

use crate::dns::{
    core::{
        AData, Class, Data, Label, Name, QueryClass, QueryType, Question, Record, RecordType, Ttl,
    },
    net::{Message, OpCode, ResponseCode},
};

pub(super) struct MessageBuffer<'slice> {
    slice: &'slice [u8],
    pos: usize,
}

impl<'a> MessageBuffer<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        MessageBuffer {
            slice: buffer,
            pos: 0,
        }
    }

    pub fn read_message(&mut self) -> Option<Message> {
        self.pos = 0;
        BufferRead::read(self)
    }

    fn read_u8(&mut self) -> Option<u8> {
        let value = self.slice.get(self.pos)?;
        self.pos += 1;
        Some(value.clone())
    }
}

trait BufferRead: Sized {
    fn read(buffer: &mut MessageBuffer) -> Option<Self>;

    fn read_all<CountT: Into<usize>>(
        buffer: &mut MessageBuffer,
        count: CountT,
    ) -> Option<Vec<Self>> {
        (0..count.into()).map(|_| Self::read(buffer)).collect()
    }

    fn read_while<F: FnMut(&Self) -> bool>(
        buffer: &mut MessageBuffer,
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

impl BufferRead for u8 {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        buffer.read_u8()
    }
}

impl BufferRead for u16 {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let u8_0: u8 = BufferRead::read(buffer)?;
        let u8_1: u8 = BufferRead::read(buffer)?;
        let u16: u16 = ((u8_0 as u16) << 8) | (u8_1 as u16);
        Some(u16)
    }
}

impl BufferRead for Label {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let length: u8 = BufferRead::read(buffer)?;
        if length == 0 {
            Some(Label::null())
        } else {
            let bytes: Vec<u8> = BufferRead::read_all(buffer, length)?;
            let label_str = String::from_utf8(bytes).ok()?;
            dbg!(&label_str);
            let label = Label::from_str(&label_str).ok()?;
            Some(label)
        }
    }
}

impl BufferRead for Name {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let labels: Vec<Label> = BufferRead::read_while(buffer, |label: &Label| !label.is_null())?;
        let name = Name::create(labels).ok()?;
        Some(name)
    }
}

impl BufferRead for QueryType {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let octect: u16 = BufferRead::read(buffer)?;
        let query_type = QueryType::from(octect);
        Some(query_type)
    }
}

impl BufferRead for QueryClass {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let octect: u16 = BufferRead::read(buffer)?;
        let query_class = QueryClass::from(octect);
        Some(query_class)
    }
}

impl BufferRead for Question {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let name: Name = BufferRead::read(buffer)?;
        let query_type: QueryType = BufferRead::read(buffer)?;
        let query_class: QueryClass = BufferRead::read(buffer)?;
        let question: Question = Question::new(name, query_type, query_class);
        Some(question)
    }
}

impl BufferRead for RecordType {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let bytes: u16 = BufferRead::read(buffer)?;
        let record_type = RecordType::from(bytes);
        Some(record_type)
    }
}

impl BufferRead for Class {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let bytes: u16 = BufferRead::read(buffer)?;
        let class = Class::from(bytes);
        Some(class)
    }
}

impl BufferRead for u32 {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let u16_1: u16 = BufferRead::read(buffer)?;
        let u16_2: u16 = BufferRead::read(buffer)?;
        let u32 = ((u16_1 as u32) << 16) | (u16_2 as u32);
        Some(u32)
    }
}

impl BufferRead for Ipv4Addr {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let octet_1: u8 = BufferRead::read(buffer)?;
        let octet_2: u8 = BufferRead::read(buffer)?;
        let octet_3: u8 = BufferRead::read(buffer)?;
        let octet_4: u8 = BufferRead::read(buffer)?;
        let ipv4_addr = Ipv4Addr::new(octet_1, octet_2, octet_3, octet_4);
        Some(ipv4_addr)
    }
}

impl BufferRead for Record {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let name: Name = BufferRead::read(buffer)?;
        let record_type: RecordType = BufferRead::read(buffer)?;
        let class: Class = BufferRead::read(buffer)?;
        let ttl: Ttl = BufferRead::read(buffer)?;
        let data_length: u16 = BufferRead::read(buffer)?;
        let data = match record_type {
            RecordType::A => {
                let ipv4_addr: Ipv4Addr = BufferRead::read(buffer)?;
                let a_data = AData::new(ipv4_addr);
                Some(Data::A(a_data))
            }
            RecordType::Unknown(type_value) => {
                let bytes: Vec<u8> = BufferRead::read_all(buffer, data_length)?;
                let unknown_data = Data::Unknown(RecordType::Unknown(type_value), bytes);
                Some(unknown_data)
            }
        }?;
        Some(Record::new(name, class, ttl, data))
    }
}

impl BufferRead for Message {
    fn read(buffer: &mut MessageBuffer) -> Option<Self> {
        let bytes_0_1: u16 = BufferRead::read(buffer)?;
        let byte2: u8 = BufferRead::read(buffer)?;
        let byte3: u8 = BufferRead::read(buffer)?;
        let message = Message::new(bytes_0_1)
            .set_is_response((byte2 & 0b1000_0000) != 0)
            .set_opcode(OpCode::from((byte2 & 0b0111_1000) >> 3))
            .set_is_authoritative_answer((byte2 & 0b0000_0100) != 0)
            .set_is_truncation((byte2 & 0b0000_0010) != 0)
            .set_recursion_desired((byte2 & 0b0000_0001) != 0)
            .set_recursion_available((byte3 & 0b1000_0000) != 0)
            .set_response_code(ResponseCode::from(byte3 & 0b0000_1111));
        let question_count: u16 = BufferRead::read(buffer)?;
        let answer_count: u16 = BufferRead::read(buffer)?;
        let authority_count: u16 = BufferRead::read(buffer)?;
        let additional_count: u16 = BufferRead::read(buffer)?;
        let questions: Vec<Question> = BufferRead::read_all(buffer, question_count)?;
        let answers: Vec<Record> = BufferRead::read_all(buffer, answer_count)?;
        let authorities: Vec<Record> = BufferRead::read_all(buffer, authority_count)?;
        let additional: Vec<Record> = BufferRead::read_all(buffer, additional_count)?;
        let message = message
            .set_questions(questions)
            .set_answers(answers)
            .set_authorities(authorities)
            .set_additional(additional);
        Some(message)
    }
}
