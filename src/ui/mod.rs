use colored::Colorize;
use std::io::{self, Write};

pub fn header(title: &str) {
    println!("\n{}", title.bold().underline());
}

pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

pub fn warning(msg: &str) {
    eprintln!("{} {}", "⚠".yellow().bold(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

pub fn separator() {
    println!("{}", "─".repeat(60).bright_black());
}

pub fn keyval(key: &str, val: &str) {
    println!("{}: {}", key.bold(), val);
}

pub fn tag(label: &str, val: &str) {
    println!("{} {}", label.bold().white().on_blue(), val);
}

pub fn indent(msg: &str, level: usize) {
    let spaces = " ".repeat(level * 2);
    println!("{}{}", spaces, msg);
}

pub fn prompt_yes_no(question: &str) -> bool {
    print!("{} {} [Y/n] ", "?".yellow().bold(), question);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim().to_lowercase();
    
    if input.is_empty() {
        return true;
    }
    
    input == "y" || input == "yes"
}
