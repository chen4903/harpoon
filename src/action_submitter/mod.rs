pub mod map;
pub mod printer;
#[cfg(feature = "telegram")]
pub mod telegram;

use crate::interface::ActionSubmitterInterface;
use std::fmt::Debug;
use tokio::sync::broadcast::Sender;

pub use map::ActionSubmitterMap;
pub use printer::ActionPrinter;
#[cfg(feature = "telegram")]
pub use telegram::TelegramSubmitter;

#[derive(Clone)]
pub struct ActionChannelSubmitter<A> {
    sender: Sender<A>,
}

impl<A> ActionChannelSubmitter<A> {
    pub fn new(sender: Sender<A>) -> Self {
        Self { sender }
    }
}

impl<A> ActionSubmitterInterface<A> for ActionChannelSubmitter<A>
where
    A: Send + Sync + Clone + Debug + 'static,
{
    fn submit(&self, action: A) {
        match self.sender.send(action) {
            Ok(_) => (),
            Err(e) => tracing::error!("error submitting action: {:?}", e),
        }
    }
}
