use colored::Colorize;
use std::io::{self, IsTerminal, Write};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod progress;

static COLOR_MODE: OnceLock<ColorMode> = OnceLock::new();
static QUIET_MODE: AtomicBool = AtomicBool::new(false);
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

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

/// Enable or disable quiet mode globally.
pub fn set_quiet(enabled: bool) {
    QUIET_MODE.store(enabled, Ordering::Relaxed);
}

/// Mark an interruption request (e.g. Ctrl+C).
pub fn mark_interrupted() {
    INTERRUPTED.store(true, Ordering::Relaxed);
}

/// Check whether interruption was requested.
pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::Relaxed)
}

fn is_quiet() -> bool {
    QUIET_MODE.load(Ordering::Relaxed)
}

/// Check if colors should be applied based on current mode
fn should_colorize() -> bool {
    let mode = COLOR_MODE.get().copied().unwrap_or(ColorMode::Auto);

    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            // Check if we're in a TTY
            io::stdout().is_terminal()
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
    if is_quiet() {
        return;
    }
    println!("\n{}", color_str(title, |s| s.bold().underline()));
}

pub fn success(msg: &str) {
    if is_quiet() {
        return;
    }
    println!("{}", color_str(msg, |s| s.green()));
}

pub fn info(msg: &str) {
    if is_quiet() {
        return;
    }
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
    if is_quiet() {
        return;
    }
    println!("{}", color_str(&"─".repeat(60), |s| s.bright_black()));
}

pub fn keyval(key: &str, val: &str) {
    if is_quiet() {
        return;
    }
    println!("{}: {}", color_str(key, |s| s.bold()), val);
}

pub fn tag(label: &str, val: &str) {
    if is_quiet() {
        return;
    }
    let tag = color_str(label, |s| s.bold().white().on_blue());
    println!("{} {}", tag, val);
}

pub fn indent(msg: &str, level: usize) {
    if is_quiet() {
        return;
    }
    let spaces = " ".repeat(level * 2);
    println!("{}{}", spaces, msg);
}

pub fn prompt_yes_no(question: &str) -> bool {
    prompt_yes_no_default(question, true)
}

pub fn prompt_yes_no_default(question: &str, default: bool) -> bool {
    if is_interrupted() {
        return false;
    }

    let suffix = if default { "[Y/n]" } else { "[y/N]" };
    let symbol = color_str("?", |s| s.yellow().bold());
    print!("{} {} {} ", symbol, question, suffix);

    // Attempt to flush stdout, fail-closed on terminal failures
    if let Err(e) = io::stdout().flush() {
        eprintln!("\nWarning: Failed to flush terminal: {}", e);
        return false;
    }

    let mut input = String::new();

    // Attempt to read line, fail-closed if stdin is broken/interrupted
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            if is_interrupted() {
                return false;
            }
            let input = input.trim().to_lowercase();

            if input.is_empty() {
                return default;
            }

            input == "y" || input == "yes"
        }
        Err(e) => {
            if e.kind() == io::ErrorKind::Interrupted || is_interrupted() {
                return false;
            }
            eprintln!("\nWarning: Failed to read input: {}", e);
            false
        }
    }
}
