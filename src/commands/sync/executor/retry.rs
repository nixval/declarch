use crate::error::{DeclarchError, Result};
use crate::ui as output;
use std::thread;
use std::time::Duration;

pub(super) fn execute_with_retry<F>(
    mut operation: F,
    operation_name: &str,
    max_retries: u32,
    delay_ms: u64,
) -> Result<()>
where
    F: FnMut() -> Result<()>,
{
    let mut last_error = None;

    for attempt in 1..=max_retries {
        match operation() {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    output::warning(&format!(
                        "{} failed (attempt {}/{}), retrying in {}s...",
                        operation_name,
                        attempt,
                        max_retries,
                        delay_ms / 1000
                    ));
                    thread::sleep(Duration::from_millis(delay_ms));
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        DeclarchError::Other(format!(
            "{} failed after {} attempts",
            operation_name, max_retries
        ))
    }))
}

#[cfg(test)]
mod tests {
    use super::execute_with_retry;
    use crate::error::DeclarchError;

    #[test]
    fn retry_succeeds_before_max_attempts() {
        let mut attempts = 0u32;
        let result = execute_with_retry(
            || {
                attempts += 1;
                if attempts < 3 {
                    Err(DeclarchError::Other("temporary".to_string()))
                } else {
                    Ok(())
                }
            },
            "op",
            3,
            0,
        );
        assert!(result.is_ok());
        assert_eq!(attempts, 3);
    }

    #[test]
    fn retry_returns_last_error_when_all_attempts_fail() {
        let mut attempts = 0u32;
        let result = execute_with_retry(
            || {
                attempts += 1;
                Err(DeclarchError::Other(format!("fail-{attempts}")))
            },
            "op",
            2,
            0,
        )
        .expect_err("should fail");

        assert_eq!(attempts, 2);
        assert!(result.to_string().contains("fail-2"));
    }
}
