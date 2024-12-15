use async_trait::async_trait;

use super::{Resolver, ResolverError, QueryRequest, QueryResponse};

pub struct EmptyResolver {}

impl EmptyResolver {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Resolver for EmptyResolver {
    async fn handle_query(&self, _: QueryRequest) -> Result<QueryResponse, ResolverError> {
        Ok(QueryResponse::default())
    }
}
