use async_trait::async_trait;

use super::{Handler, HandlerError, QueryResponse};

pub struct EmptyHandler {}

impl EmptyHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Handler for EmptyHandler {
    async fn handle_query(
        &self,
        request: super::QueryRequest,
    ) -> Result<QueryResponse, HandlerError> {
        dbg!(&request);
        Ok(QueryResponse::default())
    }
}

struct ExceptionHandler {}

impl ExceptionHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Handler for ExceptionHandler {
    async fn handle_query(
        &self,
        request: super::QueryRequest,
    ) -> Result<QueryResponse, HandlerError> {
        dbg!(&request);
        Err(HandlerError::NotImplemented)
    }
}
