use crate::dns::{
    core::{Question, Record},
    handler::{HandlerError, QueryRequest, QueryResponse},
};

type MessageId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    /// A standard query (QUERY)
    Query,

    /// An inverse query (IQUERY)
    IQuery,

    /// A server status request (STATUS)
    Status,

    /// This value is reserved for future use.
    Unkown(u8),
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => OpCode::Query,
            1 => OpCode::IQuery,
            2 => OpCode::Status,
            _ => OpCode::Unkown(value),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::Query => 0,
            OpCode::IQuery => 1,
            OpCode::Status => 2,
            OpCode::Unkown(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseCode {
    /// No error condition
    NoError,

    /// The name server was unable to interpret the query.
    FormatError,

    /// The name server was unable to process this query due to a problem with
    /// the name server.
    ServerFailure,

    /// Meaningful only for responses from an authoritative name server, this
    /// code signifies that the domain name referenced in the query does not exist.
    NameError,

    /// The name server does not support the requested kind of query.
    NotImplemented,

    /// The name server refuses to perform the specified operation for policy reasons.
    Refused,

    /// This value is reserved for future use.
    Unkown(u8),
}

impl From<u8> for ResponseCode {
    fn from(value: u8) -> Self {
        match value {
            0 => ResponseCode::NoError,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            _ => ResponseCode::Unkown(value),
        }
    }
}

impl From<ResponseCode> for u8 {
    fn from(value: ResponseCode) -> Self {
        match value {
            ResponseCode::NoError => 0,
            ResponseCode::FormatError => 1,
            ResponseCode::ServerFailure => 2,
            ResponseCode::NameError => 3,
            ResponseCode::NotImplemented => 4,
            ResponseCode::Refused => 5,
            ResponseCode::Unkown(value) => value,
        }
    }
}

/// All communications inside of the domain protocol are carried in a single
/// format called a message.
#[derive(Debug, Clone)]
pub struct Message {
    /// This identifier is copied the corresponding reply and can be used by the
    /// requester to match up replies to outstanding queries.
    id: MessageId,

    /// QR (Query/Response) flag.
    is_response: bool,

    /// A four bit field that specifies kind of query in this message. This value
    /// is set by the originator of a query and copied into the response.
    opcode: OpCode,

    /// This bit is valid in responses, and specifies that the responding name
    /// server is an authority for the domain name in question section.
    is_authoritative_answer: bool,

    /// Specifies that this message was truncated due to length greater than
    /// that permitted on the transmission channel.
    is_truncation: bool,

    /// This bit may be set in a query and is copied into the response. If RD is
    /// set, it directs the name server to pursue the query recursively.
    recursion_desired: bool,

    /// This be is set or cleared in a response, and denotes whether recursive
    /// query support is available in the name server.
    recursion_available: bool,

    /// this 4 bit field is set as part of responses.
    response_code: ResponseCode,

    /// The parameters that define what is being asked.
    questions: Vec<Question>,

    answers: Vec<Record>,

    authorities: Vec<Record>,

    additional: Vec<Record>,
}

#[derive(Debug, PartialEq)]
pub enum Request {
    Query(QueryRequest),
}

impl Message {
    pub fn new(id: MessageId) -> Self {
        Self {
            id,
            is_response: false,
            opcode: OpCode::Query,
            is_authoritative_answer: false,
            is_truncation: false,
            recursion_desired: false,
            recursion_available: false,
            response_code: ResponseCode::NoError,
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            additional: Vec::new(),
        }
    }

    pub fn id(&self) -> MessageId {
        self.id
    }

    pub fn set_is_response(mut self, is_response: bool) -> Self {
        self.is_response = is_response;
        self
    }

    pub fn is_response(&self) -> bool {
        self.is_response
    }

    pub fn set_opcode(mut self, opcode: OpCode) -> Self {
        self.opcode = opcode;
        self
    }

    pub fn opcode(&self) -> OpCode {
        self.opcode
    }

    pub fn set_is_authoritative_answer(mut self, is_authoritative_answer: bool) -> Self {
        self.is_authoritative_answer = is_authoritative_answer;
        self
    }

    pub fn is_authoritative_answer(&self) -> bool {
        self.is_authoritative_answer
    }

    pub fn set_is_truncation(mut self, is_truncation: bool) -> Self {
        self.is_truncation = is_truncation;
        self
    }

    pub fn is_truncation(&self) -> bool {
        self.is_truncation
    }

    pub fn set_recursion_desired(mut self, recursion_desired: bool) -> Self {
        self.recursion_desired = recursion_desired;
        self
    }

    pub fn recursion_desired(&self) -> bool {
        self.recursion_desired
    }

    pub fn set_recursion_available(mut self, recursion_available: bool) -> Self {
        self.recursion_available = recursion_available;
        self
    }

    pub fn recursion_available(&self) -> bool {
        self.recursion_available
    }

    pub fn set_response_code(mut self, response_code: ResponseCode) -> Self {
        self.response_code = response_code;
        self
    }

    pub fn response_code(&self) -> ResponseCode {
        self.response_code
    }

    pub fn set_questions(mut self, questions: Vec<Question>) -> Self {
        self.questions = questions;
        self
    }

    pub fn questions(&self) -> &Vec<Question> {
        &self.questions
    }

    pub fn set_answers(mut self, answers: Vec<Record>) -> Self {
        self.answers = answers;
        self
    }

    pub fn answers(&self) -> &Vec<Record> {
        &self.answers
    }

    pub fn set_authorities(mut self, authorities: Vec<Record>) -> Self {
        self.authorities = authorities;
        self
    }

    pub fn authorities(&self) -> &Vec<Record> {
        &self.authorities
    }

    pub fn set_additional(mut self, additional: Vec<Record>) -> Self {
        self.additional = additional;
        self
    }

    pub fn additional(&self) -> &Vec<Record> {
        &self.additional
    }

    pub fn to_empty_response(&self) -> Self {
        Self::new(self.id)
            .set_opcode(self.opcode())
            .set_is_response(true)
            .set_recursion_desired(self.recursion_desired)
    }

    pub fn into_request(&self) -> Result<Request, Message> {
        if self.is_response() {
            Err(self
                .to_empty_response()
                .set_response_code(ResponseCode::FormatError))
        } else if self.opcode() != OpCode::Query {
            Err(self
                .to_empty_response()
                .set_response_code(ResponseCode::NotImplemented))
        } else if let Some(question) = self.questions.first() {
            let query_request = QueryRequest::new(question.clone());
            Ok(Request::Query(query_request))
        } else {
            Err(self
                .to_empty_response()
                .set_response_code(ResponseCode::FormatError))
        }
    }

    pub fn from_query_reponse(request_message: &Message, response: &QueryResponse) -> Self {
        request_message
            .to_empty_response()
            .set_answers(response.answers().clone())
            .set_authorities(response.authorities().clone())
            .set_additional(response.additional().clone())
    }

    pub fn from_handler_error(id: MessageId, error: HandlerError) -> Self {
        match error {
            HandlerError::NotImplemented => Message::new(id)
                .set_is_response(true)
                .set_response_code(ResponseCode::NotImplemented),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::dns::core::{Class, Name, QueryClass, QueryType, RecordType};

    use super::*;

    #[test]
    fn test_into_request() {
        let question_mock = Question::new(
            Name::root(),
            QueryType::Type(RecordType::A),
            QueryClass::Class(Class::IN),
        );

        // query request message
        let message = Message::new(0)
            .set_is_response(false)
            .set_opcode(OpCode::Query)
            .set_questions(vec![question_mock.clone()]);
        let expected_query = QueryRequest::new(question_mock.clone());
        assert_eq!(
            message.into_request().unwrap(),
            Request::Query(expected_query)
        );

        // response message
        let message = Message::new(0)
            .set_is_response(true)
            .set_opcode(OpCode::Query)
            .set_questions(vec![question_mock.clone()]);
        let error_message = message.into_request().unwrap_err();
        assert_eq!(error_message.response_code(), ResponseCode::FormatError);
        assert_eq!(error_message.is_response(), true);
        assert_eq!(error_message.opcode(), OpCode::Query);
    }
}
