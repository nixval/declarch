use super::{extension_prefix, normalize_ext_name};

#[test]
fn normalize_keeps_unix_name() {
    let sample = format!("{}security", extension_prefix());
    assert_eq!(normalize_ext_name(&sample), sample);
}

#[test]
fn normalize_windows_suffixes() {
    if cfg!(windows) {
        let sample = format!("{}security", extension_prefix());
        assert_eq!(normalize_ext_name(&format!("{}.exe", sample)), sample);
        assert_eq!(normalize_ext_name(&format!("{}.cmd", sample)), sample);
    }
}
