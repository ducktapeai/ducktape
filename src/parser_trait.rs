use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Parser {
    async fn parse_input(&self, input: &str) -> Result<Option<String>>;
    fn new() -> Result<Self> where Self: Sized;
}
