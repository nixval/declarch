use std::io::{self, Write};
use std::time::{Duration, Instant};
use colored::Colorize;
use crate::ui;

/// Progress indicator for long-running operations
pub struct ProgressBar {
    total: usize,
    current: usize,
    message: String,
    start_time: Instant,
    width: usize,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(total: usize, message: &str) -> Self {
        Self {
            total,
            current: 0,
            message: message.to_string(),
            start_time: Instant::now(),
            width: 50,
        }
    }

    /// Increment the progress by 1
    pub fn inc(&mut self) {
        if self.current < self.total {
            self.current += 1;
            self.draw();
        }
    }

    /// Set the current progress
    pub fn set(&mut self, value: usize) {
        self.current = value.min(self.total);
        self.draw();
    }

    /// Finish the progress bar
    pub fn finish(self) {
        // Draw final state
        self.draw();
        println!();
    }

    /// Draw the progress bar
    fn draw(&self) {
        let percent = if self.total > 0 {
            (self.current * 100) / self.total
        } else {
            100
        };

        let filled = if self.total > 0 {
            (self.current * self.width) / self.total
        } else {
            self.width
        };

        let bar = "█".repeat(filled);
        let empty = "░".repeat(self.width.saturating_sub(filled));

        let elapsed = self.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();

        let eta = if self.current > 0 && elapsed_secs > 0.0 {
            let remaining = self.total.saturating_sub(self.current);
            let rate = self.current as f64 / elapsed_secs;
            if rate > 0.0 {
                Duration::from_secs_f64(remaining as f64 / rate)
            } else {
                Duration::ZERO
            }
        } else {
            Duration::ZERO
        };

        let eta_str = if eta.as_secs() > 0 {
            format!("{:.0}s", eta.as_secs())
        } else {
            "--".to_string()
        };

        // Use carriage return to overwrite the line
        print!(
            "\r{} {} {} {}/{} {}% ETA: {}",
            "▸".dimmed(),
            self.message.cyan(),
            format!("[{}{}]", bar.green(), empty.dimmed()),
            self.current.to_string().bold(),
            self.total.to_string().dimmed(),
            percent.to_string().bold(),
            eta_str.dimmed()
        );

        io::stdout().flush().unwrap_or(());
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        // Ensure we finish with a newline
        println!();
    }
}

/// Spinner for operations of unknown duration
pub struct Spinner {
    message: String,
    frames: Vec<&'static str>,
    current_frame: usize,
    active: bool,
}

impl Spinner {
    /// Create a new spinner
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
            active: true,
        }
    }

    /// Update the spinner message
    pub fn update_message(&mut self, message: &str) {
        self.message = message.to_string();
        self.draw();
    }

    /// Stop the spinner with success message
    pub fn finish_with_success(mut self, message: &str) {
        self.stop();
        ui::success(message);
    }

    /// Stop the spinner with error message
    pub fn finish_with_error(mut self, message: &str) {
        self.stop();
        ui::error(message);
    }

    /// Stop the spinner
    fn stop(&mut self) {
        self.active = false;
        // Clear the spinner line
        print!("\r{:width$}\r", "", width = 100);
        io::stdout().flush().unwrap_or(());
    }

    /// Draw the spinner
    fn draw(&self) {
        if !self.active {
            return;
        }

        let frame = self.frames[self.current_frame % self.frames.len()];
        print!(
            "\r{} {} {}",
            frame.cyan().bold(),
            self.message,
            "..."
        );
        io::stdout().flush().unwrap_or(());
    }

    /// Advance to the next frame
    fn advance(&mut self) {
        self.current_frame += 1;
        self.draw();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if self.active {
            self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_creation() {
        let bar = ProgressBar::new(100, "Testing");
        assert_eq!(bar.total, 100);
        assert_eq!(bar.current, 0);
    }

    #[test]
    fn test_progress_bar_increment() {
        let mut bar = ProgressBar::new(10, "Testing");
        bar.inc();
        assert_eq!(bar.current, 1);
        bar.inc();
        assert_eq!(bar.current, 2);
    }

    #[test]
    fn test_progress_bar_set() {
        let mut bar = ProgressBar::new(100, "Testing");
        bar.set(50);
        assert_eq!(bar.current, 50);
        bar.set(150); // Should cap at total
        assert_eq!(bar.current, 100);
    }

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new("Testing");
        assert_eq!(spinner.message, "Testing");
        assert!(spinner.active);
    }
}
