use super::normalize_backend_args;

#[test]
fn normalize_backend_args_supports_comma_and_space_forms() {
    let input = vec![
        "pnpm,yarn".to_string(),
        "bun".to_string(),
        "  ".to_string(),
        "paru, yay".to_string(),
    ];
    let normalized = normalize_backend_args(&input);
    assert_eq!(
        normalized,
        vec![
            "pnpm".to_string(),
            "yarn".to_string(),
            "bun".to_string(),
            "paru".to_string(),
            "yay".to_string()
        ]
    );
}
