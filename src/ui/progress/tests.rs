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
