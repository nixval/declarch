use std::fs;
use std::path::Path;

pub(super) fn detect_default_backend() -> &'static str {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        let id = content
            .lines()
            .find(|line| line.starts_with("ID="))
            .and_then(|line| line.strip_prefix("ID="))
            .map(|s| s.trim_matches('"'));

        return match id {
            Some("debian") | Some("ubuntu") | Some("linuxmint") | Some("pop") => "apt",
            Some("fedora") | Some("rhel") | Some("centos") | Some("rocky") | Some("almalinux") => {
                "dnf"
            }
            Some("opensuse") | Some("opensuse-tumbleweed") | Some("suse") => "zypper",
            Some("arch") | Some("manjaro") | Some("endeavouros") | Some("cachyos") => "aur",
            _ => "aur",
        };
    }

    if Path::new("/usr/bin/pacman").exists() {
        "aur"
    } else if Path::new("/usr/bin/apt").exists() {
        "apt"
    } else if Path::new("/usr/bin/dnf").exists() {
        "dnf"
    } else {
        "aur"
    }
}
