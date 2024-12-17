use crate::dns::core::{Message, ResponseCode};

use super::Resolver;
use async_trait::async_trait;

pub struct EmptyResolver {}

impl EmptyResolver {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Resolver for EmptyResolver {
    async fn resolve(&self, request: &Message) -> Message {
        request.into_response(ResponseCode::NoError)
    }
}
