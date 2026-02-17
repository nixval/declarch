use super::parse_bool_option;

#[test]
fn parse_bool_option_variants() {
    assert_eq!(parse_bool_option("true"), Some(true));
    assert_eq!(parse_bool_option("yes"), Some(true));
    assert_eq!(parse_bool_option("on"), Some(true));
    assert_eq!(parse_bool_option("1"), Some(true));
    assert_eq!(parse_bool_option("false"), Some(false));
    assert_eq!(parse_bool_option("no"), Some(false));
    assert_eq!(parse_bool_option("off"), Some(false));
    assert_eq!(parse_bool_option("0"), Some(false));
    assert_eq!(parse_bool_option("maybe"), None);
}
