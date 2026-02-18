#[test]
fn test_offset_to_line_col() {
    use super::offset_to_line_col;

    let content = "line1\nline2\nline3";
    assert_eq!(offset_to_line_col(content, 0), (1, 1));
    assert_eq!(offset_to_line_col(content, 6), (2, 1));
    assert_eq!(offset_to_line_col(content, 12), (3, 1));
}
