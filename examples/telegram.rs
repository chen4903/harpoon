use harpoon::{
    action_submitter::telegram::TelegramSubmitter, executor::telegram_message::MessageBuilder,
    interface::ActionSubmitterInterface,
};

use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv::from_filename("examples/.env").ok();

    dotenv().ok();

    let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    let chat_id = env::var("TELEGRAM_BOT_CHAT_ID").expect("TELEGRAM_BOT_CHAT_ID must be set");

    // Create dispatcher for sending messages
    let dispatcher = TelegramSubmitter::default();

    // Test 1: Send a simple text message
    let simple_message = MessageBuilder::new()
        .bot_token(&bot_token)
        .chat_id(&chat_id)
        .disable_notification(true)
        .text("simple message")
        .build();

    dispatcher.submit(simple_message);

    // Test 2: Send a formatted message with all options
    let formatted_message = MessageBuilder::new()
        .bot_token(&bot_token)
        .chat_id(&chat_id)
        .text("*Bold text*\n_Italic text_\n`Code block`\n[Link](https://example.com)")
        .parse_mode("MarkdownV2")
        .disable_notification(true)
        .protect_content(true)
        .disable_link_preview(false)
        .build();

    dispatcher.submit(formatted_message);

    // Test 3: Send a message with HTML formatting
    let html_message = MessageBuilder::new()
        .bot_token(&bot_token)
        .chat_id(&chat_id)
        .disable_notification(true)
        .text("<b>Bold text</b>\n<i>Italic text</i>\n<code>Code block</code>\n<a href=\"https://example.com\">Link</a>")
        .parse_mode("HTML")
        .build();

    dispatcher.submit(html_message);
}
