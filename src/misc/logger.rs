use chrono::Local;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Info,
    Debug,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl LogLevel {
    fn should_log(&self, target_level: &str) -> bool {
        let allowed = match self {
            LogLevel::Error => vec!["error"],
            LogLevel::Info => vec![
                "separator",
                "table",
                "info",
                "success",
                "warn",
                "warning",
                "process",
                "event",
                "tx",
                "error",
                "item",
                "subitem",
            ],
            LogLevel::Debug => vec![
                "debug",
                "warn",
                "warning",
                "error",
                "separator",
                "table",
                "info",
                "success",
                "process",
                "event",
                "tx",
                "item",
                "subitem",
            ],
        };
        allowed.contains(&target_level)
    }
}

struct Colors;

#[allow(dead_code)]
impl Colors {
    const RESET: &'static str = "\x1b[0m";
    const BRIGHT: &'static str = "\x1b[1m";
    const GREEN: &'static str = "\x1b[32m";
    const YELLOW: &'static str = "\x1b[33m";
    const BLUE: &'static str = "\x1b[94m";
    const CYAN: &'static str = "\x1b[36m";
    const RED: &'static str = "\x1b[31m";
    const LIGHT_GRAY: &'static str = "\x1b[37m";
    const LIGHT_BLUE: &'static str = "\x1b[94m";
    const LIGHT_CYAN: &'static str = "\x1b[96m";
    const LIGHT_PINK: &'static str = "\x1b[95m";
}

pub struct Logger {
    level: Arc<Mutex<LogLevel>>,
    files: Arc<Mutex<HashMap<String, File>>>,
}

impl Logger {
    fn new(level: LogLevel) -> Self {
        Self::ensure_log_dirs();
        Self {
            level: Arc::new(Mutex::new(level)),
            files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn ensure_log_dirs() {
        let _ = fs::create_dir_all("logs/normal");
        let _ = fs::create_dir_all("logs/refine");
    }

    fn get_timestamp(&self) -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    fn format_log(&self, message: &str) -> String {
        format!("{} {}", self.get_timestamp(), message)
    }

    fn write_to_file(&self, file_path: &str, message: &str) {
        let mut files = self.files.lock().unwrap();
        let file = files
            .entry(file_path.to_string())
            .or_insert_with(|| OpenOptions::new().create(true).append(true).open(file_path).unwrap());
        let timestamp = self.get_timestamp();
        let _ = writeln!(file, "[{}] {}", timestamp, message);
    }

    fn should_log(&self, level: &str) -> bool {
        let current_level = self.level.lock().unwrap();
        current_level.should_log(level)
    }

    pub fn separator(&self) {
        if !self.should_log("separator") {
            return;
        }
        let separator = "=======================================================================================";
        println!("{} {}", self.get_timestamp(), separator);
    }

    pub fn table(&self, title: &str, data: &HashMap<String, String>) {
        if !self.should_log("table") {
            return;
        }

        let max_key_length = data.keys().map(|k| k.len()).max().unwrap_or(0);
        let max_column_width = 100;
        let padding = 3;
        let value_width = max_column_width - max_key_length - padding - 4;

        let wrap_text = |text: &str, width: usize| -> Vec<String> {
            let mut lines = Vec::new();
            let paragraphs: Vec<&str> = text.split('\n').collect();

            for paragraph in paragraphs {
                if paragraph.trim().is_empty() {
                    lines.push(String::new());
                    continue;
                }

                let words: Vec<&str> = paragraph.trim().split(' ').collect();
                let mut current_line = String::new();

                for word in words {
                    if current_line.len() + (if current_line.is_empty() { 0 } else { 1 }) + word.len() <= width {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    } else {
                        if !current_line.is_empty() {
                            lines.push(current_line.clone());
                        }
                        if word.len() > width {
                            let mut remaining = word;
                            while remaining.len() > width {
                                lines.push(remaining[..width].to_string());
                                remaining = &remaining[width..];
                            }
                            current_line = remaining.to_string();
                        } else {
                            current_line = word.to_string();
                        }
                    }
                }
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
            }
            lines
        };

        let total_width = max_column_width;
        let line = "─".repeat(total_width - 2);

        let mut table_lines = Vec::new();
        let mut console_table_lines = Vec::new();

        console_table_lines.push(format!("{}┌{}┐{}", Colors::BLUE, line, Colors::RESET));
        table_lines.push(format!("┌{}┐", line));

        if !title.is_empty() {
            let title_padding = (total_width - title.len() - 2) / 2;
            let title_line = format!(
                "{}{}{}",
                " ".repeat(title_padding),
                title,
                " ".repeat(total_width - title.len() - title_padding - 2)
            );
            console_table_lines.push(format!(
                "{}│{}{}{}│{}",
                Colors::BLUE,
                Colors::CYAN,
                title_line,
                Colors::BLUE,
                Colors::RESET
            ));
            table_lines.push(format!("│{}│", title_line));
            console_table_lines.push(format!("{}├{}┤{}", Colors::BLUE, line, Colors::RESET));
            table_lines.push(format!("├{}┤", line));
        }

        let entries: Vec<_> = data.iter().collect();
        for (index, (key, value)) in entries.iter().enumerate() {
            let key_padded = format!("{:width$}", key, width = max_key_length);
            let wrapped_lines = wrap_text(value, value_width);

            let first_line_padded = format!(
                "{:width$}",
                wrapped_lines.get(0).unwrap_or(&String::new()),
                width = value_width
            );
            let first_line = format!(" {} │ {} ", key_padded, first_line_padded);
            console_table_lines.push(format!(
                "{}│{}{} {}",
                Colors::BLUE,
                Colors::LIGHT_GRAY,
                first_line,
                Colors::RESET
            ));
            table_lines.push(format!("│{}│", first_line));

            for i in 1..wrapped_lines.len() {
                let line_padded = format!("{:width$}", wrapped_lines[i], width = value_width);
                let content_line = format!(" {} │ {} ", " ".repeat(max_key_length), line_padded);
                console_table_lines.push(format!(
                    "{}│{}{}{}│{}",
                    Colors::BLUE,
                    Colors::LIGHT_GRAY,
                    content_line,
                    Colors::BLUE,
                    Colors::RESET
                ));
                table_lines.push(format!("│{}│", content_line));
            }

            if index < entries.len() - 1 {
                console_table_lines.push(format!("{}├{}┤{}", Colors::BLUE, line, Colors::RESET));
                table_lines.push(format!("├{}┤", line));
            }
        }

        console_table_lines.push(format!("{}└{}┘{}", Colors::BLUE, line, Colors::RESET));
        table_lines.push(format!("└{}┘", line));

        let formatted_table = format!("\n{}", table_lines.join("\n"));
        let formatted_console_table = format!("\n{}", console_table_lines.join("\n"));

        println!("{}{}", self.get_timestamp(), formatted_console_table);
        self.write_to_file("logs/normal/info.log", &formatted_table);
        self.write_to_file("logs/refine/table.log", &formatted_table);
    }

    pub fn info(&self, message: &str) {
        if !self.should_log("info") {
            return;
        }
        let log_message = format!("[Info] {}", message);
        println!("{}", self.format_log(&log_message));
        self.write_to_file("logs/normal/info.log", &log_message);
        self.write_to_file("logs/refine/info.log", &log_message);
    }

    pub fn success(&self, message: &str) {
        if !self.should_log("success") {
            return;
        }
        let log_message = format!("[Success] {}", message);
        println!(
            "{} {}{}{}",
            self.get_timestamp(),
            Colors::GREEN,
            log_message,
            Colors::RESET
        );
        self.write_to_file("logs/normal/info.log", &log_message);
        self.write_to_file("logs/refine/success.log", &log_message);
    }

    pub fn warn(&self, message: &str) {
        if !self.should_log("warn") {
            return;
        }
        let log_message = format!("[Warn] {}", message);
        println!(
            "{} {}{}{}",
            self.get_timestamp(),
            Colors::YELLOW,
            log_message,
            Colors::RESET
        );
        self.write_to_file("logs/normal/info.log", &log_message);
        self.write_to_file("logs/refine/warn.log", &log_message);
    }

    pub fn process(&self, message: &str) {
        if !self.should_log("process") {
            return;
        }
        let log_message = format!("[Process] {}", message);
        println!(
            "{} {}{}{}",
            self.get_timestamp(),
            Colors::BLUE,
            log_message,
            Colors::RESET
        );
        self.write_to_file("logs/normal/info.log", &log_message);
        self.write_to_file("logs/refine/process.log", &log_message);
    }

    pub fn event(&self, message: &str) {
        if !self.should_log("event") {
            return;
        }
        let log_message = format!("[Event 📩] {}", message);
        println!(
            "{} {}{}{}",
            self.get_timestamp(),
            Colors::LIGHT_PINK,
            log_message,
            Colors::RESET
        );
        self.write_to_file("logs/normal/info.log", &log_message);
        self.write_to_file("logs/refine/event.log", &log_message);
    }

    pub fn tx(&self, message: &str) {
        if !self.should_log("tx") {
            return;
        }
        let log_message = format!("[Tx 🎉] {}", message);
        println!(
            "{} {}{}{}",
            self.get_timestamp(),
            Colors::LIGHT_PINK,
            log_message,
            Colors::RESET
        );
        self.write_to_file("logs/normal/info.log", &log_message);
        self.write_to_file("logs/refine/tx.log", &log_message);
    }

    pub fn error(&self, message: &str) {
        if !self.should_log("error") {
            return;
        }
        let log_message = format!("[Error] {}", message);
        eprintln!(
            "{} {}{}{}",
            self.get_timestamp(),
            Colors::RED,
            log_message,
            Colors::RESET
        );
        self.write_to_file("logs/normal/error.log", &log_message);
        self.write_to_file("logs/refine/error.log", &log_message);
    }

    pub fn debug(&self, message: &str) {
        if !self.should_log("debug") {
            return;
        }
        let log_message = format!("[Debug] {}", message);
        println!(
            "{} {}{}{}",
            self.get_timestamp(),
            Colors::LIGHT_GRAY,
            log_message,
            Colors::RESET
        );
        self.write_to_file("logs/normal/debug.log", &log_message);
        self.write_to_file("logs/refine/debug.log", &log_message);
    }

    pub fn item(&self, message: &str) {
        if !self.should_log("item") {
            return;
        }
        let log_message = format!(" > {}", message);
        println!(
            "{} {}{}{}",
            self.get_timestamp(),
            Colors::LIGHT_BLUE,
            log_message,
            Colors::RESET
        );
        self.write_to_file("logs/normal/info.log", &log_message);
        self.write_to_file("logs/refine/item.log", &log_message);
    }

    pub fn sub_item(&self, message: &str) {
        if !self.should_log("subitem") {
            return;
        }
        let log_message = format!("   * {}", message);
        println!("{}", self.format_log(&log_message));
        self.write_to_file("logs/normal/info.log", &log_message);
        self.write_to_file("logs/refine/subitem.log", &log_message);
    }

    pub fn set_level(&self, level: LogLevel) {
        let mut current_level = self.level.lock().unwrap();
        *current_level = level;
    }
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn init_logger(level: LogLevel) -> &'static Logger {
    LOGGER.get_or_init(|| Logger::new(level))
}

pub fn get_logger() -> &'static Logger {
    LOGGER.get().expect("Logger not initialized. Call init_logger first.")
}
