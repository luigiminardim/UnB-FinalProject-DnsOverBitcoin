use crate::dns::core::{Question, Record};
use async_trait::async_trait;

#[derive(Debug, PartialEq)]
pub struct QueryRequest {
    question: Question,
}

impl QueryRequest {
    pub fn new(question: Question) -> Self {
        QueryRequest { question }
    }

    pub fn question(&self) -> &Question {
        &self.question
    }
}

#[derive(Default)]
pub struct QueryResponse {
    answers: Vec<Record>,

    authorities: Vec<Record>,

    additional: Vec<Record>,
}

impl QueryResponse {
    pub fn add_answer(mut self, record: Record) -> Self {
        self.answers.push(record);
        self
    }

    pub fn answers(&self) -> &Vec<Record> {
        &self.answers
    }

    pub fn add_authority(mut self, record: Record) -> Self {
        self.authorities.push(record);
        self
    }

    pub fn authorities(&self) -> &Vec<Record> {
        &self.authorities
    }

    pub fn add_additional(mut self, record: Record) -> Self {
        self.additional.push(record);
        self
    }

    pub fn additional(&self) -> &Vec<Record> {
        &self.additional
    }
}

pub enum ResolverError {
    NotImplemented,
}

#[async_trait]
pub trait Resolver: 'static {
    async fn handle_query(&self, request: QueryRequest) -> Result<QueryResponse, ResolverError>;
}
