use crate::dns::core::Message;
use async_trait::async_trait;

#[async_trait]
pub trait Resolver: 'static {
    async fn resolve(&self, request: &Message) -> Message;
}
