use super::load_all_backends;

#[test]
fn test_load_all_backends_empty() {
    let result = load_all_backends();
    assert!(result.is_ok());
}
