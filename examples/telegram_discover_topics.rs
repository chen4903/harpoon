use dotenv::dotenv;
use std::collections::HashMap;
use std::env;

// run `cargo run --example telegram_discover_topics` to discover topics

fn infer_topic_name(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() || text.starts_with('@') {
        return None;
    }
    let first_line = text.lines().next()?.trim();
    if first_line.len() > 32 {
        return None;
    }
    if !first_line
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    Some(first_line.to_lowercase())
}

fn to_env_key(name: &str) -> String {
    format!("TELEGRAM_TOPIC_{}", name.to_uppercase().replace([' ', '-'], "_"))
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv::from_filename("examples/.env").ok();
    dotenv().ok();

    let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");

    let url = format!("https://api.telegram.org/bot{bot_token}/getUpdates");
    let response: serde_json::Value = reqwest::get(&url).await?.json().await?;

    if !response["ok"].as_bool().unwrap_or(false) {
        eyre::bail!("getUpdates failed: {response}");
    }

    let updates = response["result"].as_array().cloned().unwrap_or_default();
    if updates.is_empty() {
        println!("No updates found.");
        println!();
        println!("Steps:");
        println!("1. Add bot to the group and disable Group Privacy in @BotFather");
        println!("2. In each topic, send the topic name as message (e.g. sonar, freqtrade)");
        println!("3. Run this script again");
        return Ok(());
    }

    // thread_id -> topic name
    let mut topics: HashMap<i64, String> = HashMap::new();
    let mut chat_id: Option<i64> = None;

    for update in &updates {
        let message = update
            .get("message")
            .or_else(|| update.get("channel_post"))
            .or_else(|| update.get("edited_message"));

        let Some(message) = message else {
            continue;
        };

        let chat = &message["chat"];
        if chat["is_forum"].as_bool() != Some(true) {
            continue;
        }

        chat_id = chat["id"].as_i64();

        let thread_id = match message["message_thread_id"].as_i64() {
            Some(id) => id,
            None => continue,
        };

        if let Some(created) = message.get("forum_topic_created") {
            if let Some(name) = created["name"].as_str() {
                topics.insert(thread_id, name.to_lowercase());
                continue;
            }
        }

        let text = message["text"].as_str().unwrap_or("");
        if let Some(name) = infer_topic_name(text) {
            topics.insert(thread_id, name);
        }
    }

    if topics.is_empty() {
        println!("No forum topics found.");
        println!("In each topic, send the topic name as message (e.g. sonar, freqtrade), then re-run.");
        return Ok(());
    }

    println!("# Paste into examples/.env:\n");
    if let Some(id) = chat_id {
        println!("TELEGRAM_BOT_CHAT_ID={id}");
    }

    let mut entries: Vec<_> = topics.into_iter().collect();
    entries.sort_by_key(|(id, _)| *id);

    for (thread_id, name) in &entries {
        println!("{}={thread_id}", to_env_key(name));
    }

    println!("\n# Found {} topic(s).", entries.len());
    println!("# Test: cargo run --example telegram_topic -- --thread-id 6");

    Ok(())
}
