use harpoon::misc::logger::{LogLevel, init_logger};
use std::collections::HashMap;

fn main() {
    // Initialize logger with Info level
    let logger = init_logger(LogLevel::Info);

    // Test separator
    logger.separator();
    logger.info("Logger test started");
    logger.separator();

    // Test basic log levels
    logger.info("This is an info message");
    logger.success("Operation completed successfully");
    logger.warn("This is a warning message");
    logger.error("This is an error message");
    logger.debug("This debug message won't show at Info level");

    // Test process and events
    logger.separator();
    logger.process("Processing transaction...");
    logger.event("New message received");
    logger.tx("Transaction confirmed on chain");

    // Test item and sub_item
    logger.separator();
    logger.item("Main task item");
    logger.sub_item("Subtask 1 completed");
    logger.sub_item("Subtask 2 in progress");
    logger.item("Another main task");

    // Test table output
    logger.separator();
    let mut data = HashMap::new();
    data.insert("Name".to_string(), "Harpoon Logger".to_string());
    data.insert("Version".to_string(), "0.1.0".to_string());
    data.insert("Language".to_string(), "Rust".to_string());
    data.insert(
        "Description".to_string(),
        "A powerful logging utility with multiple output formats and colored console support".to_string(),
    );
    data.insert(
        "Features".to_string(),
        "Console output, File logging, Table formatting, Multiple log levels, Color support".to_string(),
    );
    logger.table("Logger Information", &data);

    // Test another table with longer text
    logger.separator();
    let mut tx_data = HashMap::new();
    tx_data.insert(
        "TxHash".to_string(),
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
    );
    tx_data.insert(
        "From".to_string(),
        "0xABCDEF1234567890ABCDEF1234567890ABCDEF12".to_string(),
    );
    tx_data.insert(
        "To".to_string(),
        "0x1234567890ABCDEF1234567890ABCDEF12345678".to_string(),
    );
    tx_data.insert("Value".to_string(), "1.5 ETH".to_string());
    tx_data.insert("Gas".to_string(), "21000".to_string());
    tx_data.insert("Status".to_string(), "Confirmed".to_string());
    logger.table("Transaction Details", &tx_data);

    // Change log level to Debug
    logger.separator();
    logger.info("Changing log level to Debug");
    logger.set_level(LogLevel::Debug);
    logger.debug("Now debug messages will show");
    logger.debug("Debug mode enabled - showing detailed information");

    // Change log level to Error
    logger.separator();
    logger.set_level(LogLevel::Error);
    logger.info("This info message won't show at Error level");
    logger.error("Only error messages show at Error level");

    logger.separator();
    println!("\nLogs have been written to:");
    println!("  - logs/normal/info.log");
    println!("  - logs/normal/error.log");
    println!("  - logs/normal/debug.log");
    println!("  - logs/refine/*.log (categorized by type)");
}
