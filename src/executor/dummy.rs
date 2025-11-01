use crate::IExecutor;
use async_trait::async_trait;

/// This executor is primarily used for testing purposes to verify if collectors and strategies
/// are working properly without executing real operations.
///
/// # Examples
///
/// ```
/// use crate::{Engine, executor::Dummy};
///
/// let engine = Engine::new()
///     .add_collector(some_collector)
///     .add_strategy(some_strategy)
///     .add_executor(Dummy);
/// ```
///
/// # Note
///
/// This executor will always return `Ok(())` regardless of the input action.
pub struct Dummy;

#[async_trait]
impl<A: Send + Sync + 'static> IExecutor<A> for Dummy {
    fn name(&self) -> &str {
        "Dummy"
    }

    async fn execute(&self, action: A) -> eyre::Result<()> {
        let _ = action;

        Ok(())
    }
}
