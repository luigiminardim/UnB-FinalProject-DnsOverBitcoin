use super::message::{Message, OpCode, ResponseCode};
use crate::dns::{
    core::{
        AData, Class, Data, Label, Name, QueryClass, QueryType, Question, Record, RecordType, Ttl,
    },
    handler::Handler,
    net::Request,
};
use std::{net::Ipv4Addr, str::FromStr};
use tokio::net::UdpSocket;

/// Messages carried by UDP are restricted to 512 bytes.
const UDP_LENGTH_LIMIT: usize = 512;

#[derive(Debug)]
pub enum UdpListenerError {
    IoError(std::io::Error),
}

pub struct UdpListener {
    handler: Box<dyn Handler>,
}

impl UdpListener {
    pub fn new(handler: impl Handler) -> Self {
        UdpListener {
            handler: Box::new(handler),
        }
    }

    pub async fn listen(self) -> Result<(), UdpListenerError> {
        let socket = UdpSocket::bind("127.0.0.1:1053")
            .await
            .map_err(UdpListenerError::IoError)?;
        let mut buffer = [0; UDP_LENGTH_LIMIT];
        loop {
            let (message_length, _) = socket
                .recv_from(&mut buffer)
                .await
                .map_err(UdpListenerError::IoError)?;
            let slice = &buffer[..message_length];
            let mut reader = SliceReader::new(&slice);
            let request_message = self.decode_message(&mut reader).unwrap();
            let id = request_message.id();
            let response_message = match request_message.into_request() {
                Ok(Request::Query(query_request)) => {
                    self.handler.handle_query(query_request).await.map_or_else(
                        |error| Message::from_handler_error(id, error),
                        |response| Message::from_query_reponse(&request_message, &response),
                    )
                }
                Err(response_message) => response_message,
            };
            println!("Response: {:?}", response_message);
        }
    }

    fn decode_message(&self, reader: &mut SliceReader) -> Option<Message> {
        let bytes_0_1 = reader.read_u16()?;
        let byte2 = reader.read_u8()?;
        let byte3 = reader.read_u8()?;
        let message = Message::new(bytes_0_1)
            .set_is_response((byte2 & 0b1000_0000) != 0)
            .set_opcode(OpCode::from((byte2 & 0b0111_1000) >> 3))
            .set_is_authoritative_answer((byte2 & 0b0000_0100) != 0)
            .set_is_truncation((byte2 & 0b0000_0010) != 0)
            .set_recursion_desired((byte2 & 0b0000_0001) != 0)
            .set_recursion_available((byte3 & 0b1000_0000) != 0)
            .set_response_code(ResponseCode::from(byte3 & 0b0000_1111));
        let question_count = reader.read_u16()?;
        let answer_count = reader.read_u16()?;
        let authority_count = reader.read_u16()?;
        let additional_count = reader.read_u16()?;
        let questions = (0..question_count)
            .map(|_| self.decode_question(reader))
            .collect::<Option<Vec<_>>>()?;
        let answers = (0..answer_count)
            .map(|_| self.decode_record(reader))
            .collect::<Option<Vec<_>>>()?;
        let authorities = (0..authority_count)
            .map(|_| self.decode_record(reader))
            .collect::<Option<Vec<_>>>()?;
        let additional = (0..additional_count)
            .map(|_| self.decode_record(reader))
            .collect::<Option<Vec<_>>>()?;
        let message = message
            .set_questions(questions)
            .set_answers(answers)
            .set_authorities(authorities)
            .set_additional(additional);

        Some(message)
    }

    /// Decode a question from a buffer.
    /// Returns the decoded question and the position in the buffer after the question.
    fn decode_question(&self, reader: &mut SliceReader) -> Option<Question> {
        let name = self.decode_name(reader)?;
        let query_type = QueryType::from(reader.read_u16()?);
        let query_class = QueryClass::from(reader.read_u16()?);
        let question = Question::new(name, query_type, query_class);
        Some(question)
    }

    fn decode_record(&self, reader: &mut SliceReader) -> Option<Record> {
        let name = self.decode_name(reader)?;
        let record_type = RecordType::from(reader.read_u16()?);
        let class = Class::from(reader.read_u16()?);
        let ttl = reader.read_u32()? as Ttl;
        let data = self.decode_data(record_type, reader)?;
        Some(Record::new(name, class, ttl, data))
    }

    fn decode_name(&self, reader: &mut SliceReader) -> Option<Name> {
        let labels = std::iter::repeat(0)
            .map(|_| self.decode_label(reader))
            .take_while(|maybe_label| {
                if let Some(label) = maybe_label {
                    !label.is_null()
                } else {
                    true
                }
            })
            .collect::<Option<Vec<_>>>()?;
        let name = Name::create(labels).ok()?;
        Some(name)
    }

    fn decode_label(&self, reader: &mut SliceReader) -> Option<Label> {
        let length = reader.read_u8()?;
        if length == 0 {
            Some(Label::null())
        } else {
            let bytes = (0..length)
                .map(|_| reader.read_u8())
                .collect::<Option<Vec<_>>>()?;
            let label_str = String::from_utf8(bytes).ok()?;
            dbg!(&label_str);
            let label = Label::from_str(&label_str).ok()?;
            Some(label)
        }
    }

    fn decode_data(&self, record_type: RecordType, reader: &mut SliceReader) -> Option<Data> {
        let data_length = reader.read_u16()?;
        match record_type {
            RecordType::A => {
                let ipv4_addr = Ipv4Addr::from_bits(reader.read_u32()?);
                let a_data = AData::new(ipv4_addr);
                Some(Data::A(a_data))
            }
            RecordType::Unknown(type_value) => {
                let bytes = (0..data_length)
                    .map(|_| reader.read_u8())
                    .collect::<Option<Vec<_>>>()?;
                Some(Data::Unknown(RecordType::Unknown(type_value), bytes))
            }
        }
    }
}

struct SliceReader<'slice> {
    slice: &'slice [u8],
    pos: usize,
}

impl<'a> SliceReader<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        SliceReader {
            slice: buffer,
            pos: 0,
        }
    }

    fn read_u8(&mut self) -> Option<u8> {
        let value = self.slice.get(self.pos)?;
        self.pos += 1;
        Some(value.clone())
    }

    fn read_u16(&mut self) -> Option<u16> {
        let u8_0 = self.read_u8()? as u16;
        let u8_1 = self.read_u8()? as u16;
        let u16 = (u8_0 << 8) | u8_1;
        Some(u16)
    }

    fn read_u32(&mut self) -> Option<u32> {
        let u16_1 = self.read_u16()? as u32;
        let u16_2 = self.read_u16()? as u32;
        let u32 = (u16_1 << 16) | u16_2;
        Some(u32)
    }
}
