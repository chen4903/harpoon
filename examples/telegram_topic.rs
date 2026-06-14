use harpoon::{
    action_submitter::TelegramSubmitter, executor::telegram_message::MessageBuilder, interface::IActionSubmitter,
};

use clap::Parser;
use dotenv::dotenv;
use std::env;

// cargo run --example telegram_topic -- --thread-id 6

#[derive(Parser)]
#[command(about = "Send a test message to a specific Telegram forum topic")]
struct Args {
    /// Forum topic thread id
    #[arg(long)]
    thread_id: i64,
}

#[tokio::main]
async fn main() {
    dotenv::from_filename("examples/.env").ok();
    dotenv().ok();

    let args = Args::parse();

    let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    let chat_id = env::var("TELEGRAM_BOT_CHAT_ID").expect("TELEGRAM_BOT_CHAT_ID must be set");

    let submitter = TelegramSubmitter::new_with_redirect(bot_token, chat_id, Some(args.thread_id));

    submitter.submit(
        MessageBuilder::new()
            .text(format!("test message to topic {}", args.thread_id))
            .build(),
    );

    println!("Sent to topic thread_id: {}", args.thread_id);
}
