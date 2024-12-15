use async_trait::async_trait;

use super::{Handler, HandlerError, QueryRequest, QueryResponse};

pub struct EmptyHandler {}

impl EmptyHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Handler for EmptyHandler {
    async fn handle_query(&self, _: QueryRequest) -> Result<QueryResponse, HandlerError> {
        Ok(QueryResponse::default())
    }
}
