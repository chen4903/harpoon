use async_trait::async_trait;
use eyre::Result;

#[async_trait]
pub trait Executor<A>: Send + Sync {
    fn name(&self) -> &str {
        "Unnamed"
    }

    async fn execute(&self, action: A) -> Result<()>;
}
