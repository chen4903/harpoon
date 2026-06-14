use std::sync::Arc;

use crate::IActionSubmitter;
use crate::executor::telegram_message::{Message, TelegramMessageDispatcher, TopicRegistry};

pub struct TelegramSubmitter {
    executor: Arc<TelegramMessageDispatcher>,

    redirect_to: Option<(String, String, Option<i64>)>,
}

impl TelegramSubmitter {
    pub fn new_with_redirect(bot_token: String, chat_id: String, thread_id: Option<i64>) -> Self {
        let executor = Arc::new(TelegramMessageDispatcher::default());

        Self {
            executor,
            redirect_to: Some((bot_token, chat_id, thread_id)),
        }
    }

    /// Redirect bot_token and chat_id only. Set thread_id per message.
    pub fn new_with_chat(bot_token: String, chat_id: String) -> Self {
        Self::new_with_redirect(bot_token, chat_id, None)
    }

    pub fn new_with_topics(bot_token: String, chat_id: String, topic_registry: TopicRegistry) -> Self {
        let executor = Arc::new(TelegramMessageDispatcher::default().with_topic_registry(topic_registry));

        Self {
            executor,
            redirect_to: Some((bot_token, chat_id, None)),
        }
    }
}

impl Default for TelegramSubmitter {
    fn default() -> Self {
        let executor = Arc::new(TelegramMessageDispatcher::default());
        Self {
            executor,
            redirect_to: None,
        }
    }
}

impl IActionSubmitter<Message> for TelegramSubmitter {
    fn submit(&self, action: Message) {
        let action = if let Some((bot_token, chat_id, thread_id)) = &self.redirect_to {
            Message {
                bot_token: bot_token.clone(),
                chat_id: chat_id.clone(),
                thread_id: action.thread_id.or(*thread_id),
                ..action
            }
        } else {
            action
        };

        let executor = self.executor.clone();

        std::thread::spawn(move || {
            send_message(executor, action);
        })
        .join()
        .unwrap();
    }
}

#[tokio::main(flavor = "current_thread")]
async fn send_message(executor: Arc<TelegramMessageDispatcher>, action: Message) {
    executor.send_message(action).await;
}
