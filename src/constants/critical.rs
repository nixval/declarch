/// Critical system packages protected from removal
///
/// These packages should never be removed via `--prune` even if not in config.
/// Used in:
/// - sync command (prune protection)
/// - future: config validation
/// - future: docs generation
///
/// Only packages critical to system operation are included.
/// User-choice packages (system package manager helpers like paru, yay) are NOT included.
pub const ALL: &[&str] = &[
    // Kernels & base (6)
    "linux",
    "linux-lts",
    "linux-zen",
    "linux-hardened",
    "linux-api-headers",
    "linux-firmware",
    // Firmware (2)
    "amd-ucode",
    "intel-ucode",
    // Boot loaders (4)
    "grub",
    "systemd-boot",
    "efibootmgr",
    "os-prober",
    // System essentials (2)
    "base",
    "base-devel",
    // System daemons (6)
    "systemd",
    "systemd-libs",
    "systemd-sysvcompat",
    "networkmanager",
    "iwd",
    "wpa_supplicant",
    // Core libraries (5)
    "glibc",
    "gcc-libs",
    "zlib",
    "openssl",
    "readline",
    // Shells & auth (8)
    "bash",
    "zsh",
    "fish",
    "sh",
    "sudo",
    "doas",
    "pam",
    "shadow",
    // System utilities (2)
    "util-linux",
    "coreutils",
    // Display & graphics (4)
    "mesa",
    "nvidia",
    "nvidia-utils",
    "nvidia-dkms",
    // Filesystems (4)
    "btrfs-progs",
    "e2fsprogs",
    "dosfstools",
    "ntfs-3g",
    // Declarch itself (2)
    "declarch",
    "declarch-bin",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_critical_count() {
        assert_eq!(ALL.len(), 45);
    }

    #[test]
    fn test_contains_linux() {
        assert!(ALL.contains(&"linux"));
    }

    #[test]
    fn test_contains_systemd() {
        assert!(ALL.contains(&"systemd"));
    }

    #[test]
    fn test_contains_declarch() {
        assert!(ALL.contains(&"declarch"));
        assert!(ALL.contains(&"declarch-bin"));
    }

    #[test]
    fn test_not_contains_paru() {
        assert!(!ALL.contains(&"paru"));
    }

    #[test]
    fn test_not_contains_yay() {
        assert!(!ALL.contains(&"yay"));
    }

    #[test]
    fn test_not_contains_git() {
        assert!(!ALL.contains(&"git"));
    }
}
