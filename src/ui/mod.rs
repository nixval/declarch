use colored::Colorize;
use std::io::{self, Write};
use std::sync::OnceLock;

pub mod progress;

static COLOR_MODE: OnceLock<ColorMode> = OnceLock::new();

#[derive(Clone, Copy, PartialEq)]
enum ColorMode {
    Auto,
    Always,
    Never,
}

/// Initialize color mode from settings
/// Should be called once at startup
pub fn init_colors() {
    if let Ok(settings) = crate::config::settings::Settings::load() {
        let mode = match settings.get("color").map(|s| s.as_str()) {
            Some("always") => ColorMode::Always,
            Some("never") => ColorMode::Never,
            _ => ColorMode::Auto, // default
        };
        COLOR_MODE.get_or_init(|| mode);
    } else {
        COLOR_MODE.get_or_init(|| ColorMode::Auto);
    }
}

/// Check if colors should be applied based on current mode
fn should_colorize() -> bool {
    let mode = COLOR_MODE.get().copied().unwrap_or(ColorMode::Auto);

    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            // Check if we're in a TTY
            atty::is(atty::Stream::Stdout)
        }
    }
}

/// Helper function to conditionally apply color
fn color_str(s: &str, colorizer: impl Fn(&str) -> colored::ColoredString) -> String {
    if should_colorize() {
        colorizer(s).to_string()
    } else {
        s.to_string()
    }
}

pub fn header(title: &str) {
    println!("\n{}", color_str(title, |s| s.bold().underline()));
}

pub fn success(msg: &str) {
    println!("{}", color_str(msg, |s| s.green()));
}

pub fn info(msg: &str) {
    println!("{}", color_str(msg, |s| s.blue()));
}

pub fn warning(msg: &str) {
    let symbol = color_str("⚠", |s| s.yellow().bold());
    eprintln!("{} {}", symbol, msg);
}

pub fn error(msg: &str) {
    let symbol = color_str("✗", |s| s.red().bold());
    eprintln!("{} {}", symbol, msg);
}

pub fn separator() {
    println!("{}", color_str(&"─".repeat(60), |s| s.bright_black()));
}

pub fn keyval(key: &str, val: &str) {
    println!("{}: {}", color_str(key, |s| s.bold()), val);
}

pub fn tag(label: &str, val: &str) {
    let tag = color_str(label, |s| s.bold().white().on_blue());
    println!("{} {}", tag, val);
}

pub fn indent(msg: &str, level: usize) {
    let spaces = " ".repeat(level * 2);
    println!("{}{}", spaces, msg);
}

pub fn prompt_yes_no(question: &str) -> bool {
    let symbol = color_str("?", |s| s.yellow().bold());
    print!("{} {} [Y/n] ", symbol, question);

    // Attempt to flush stdout, default to true if terminal is broken
    if let Err(e) = io::stdout().flush() {
        eprintln!("\nWarning: Failed to flush terminal: {}", e);
        return true; // Default to true on terminal failure
    }

    let mut input = String::new();

    // Attempt to read line, default to true if stdin is broken
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let input = input.trim().to_lowercase();

            if input.is_empty() {
                return true;
            }

            input == "y" || input == "yes"
        }
        Err(e) => {
            eprintln!("\nWarning: Failed to read input: {}", e);
            true // Default to true on read failure (fail-open for non-interactive)
        }
    }
}
