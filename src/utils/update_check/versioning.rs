pub fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    match (version_tuple(a), version_tuple(b)) {
        (Some(va), Some(vb)) => va.cmp(&vb),
        _ => a.cmp(b),
    }
}

pub(super) fn version_tuple(input: &str) -> Option<(u64, u64, u64)> {
    let core = input.trim().trim_start_matches('v');
    let core = core.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    Some((major, minor, patch))
}
