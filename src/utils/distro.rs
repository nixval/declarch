use std::path::Path;

/// Distro type detection for cross-distro package management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistroType {
    Arch,        // Arch Linux and derivatives (EndeavourOS, Manjaro, etc.)
    Debian,      // Debian and derivatives (Ubuntu, Mint, etc.)
    Fedora,      // Fedora and derivatives (RHEL, CentOS, etc.)
    Unknown,     // Unable to determine
}

impl DistroType {
    /// Detect the current distro by checking /etc/os-release
    pub fn detect() -> Self {
        // Check for Arch-based systems
        if Self::is_arch_based() {
            return DistroType::Arch;
        }

        // Check for Debian-based systems
        if Self::is_debian_based() {
            return DistroType::Debian;
        }

        // Check for Fedora-based systems
        if Self::is_fedora_based() {
            return DistroType::Fedora;
        }

        DistroType::Unknown
    }

    /// Check if the system is Arch-based
    fn is_arch_based() -> bool {
        // Check /etc/os-release
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let content_lower = content.to_lowercase();
            if content_lower.contains("arch linux")
                || content_lower.contains("endeavouros")
                || content_lower.contains("manjaro")
                || content_lower.contains("garuda linux")
                || content_lower.contains("arco linux")
                || content_lower.contains("artix linux") {
                return true;
            }
        }

        // Fallback: check for pacman
        Path::new("/usr/bin/pacman").exists()
    }

    /// Check if the system is Debian-based
    fn is_debian_based() -> bool {
        // Check /etc/os-release
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let content_lower = content.to_lowercase();
            if content_lower.contains("debian")
                || content_lower.contains("ubuntu")
                || content_lower.contains("mint")
                || content_lower.contains("pop!_os")
                || content_lower.contains("elementary os") {
                return true;
            }
        }

        // Fallback: check for apt or dpkg
        Path::new("/usr/bin/apt").exists() || Path::new("/usr/bin/dpkg").exists()
    }

    /// Check if the system is Fedora-based
    fn is_fedora_based() -> bool {
        // Check /etc/os-release
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let content_lower = content.to_lowercase();
            if content_lower.contains("fedora")
                || content_lower.contains("red hat")
                || content_lower.contains("rhel")
                || content_lower.contains("centos") {
                return true;
            }
        }

        // Fallback: check for dnf or yum
        Path::new("/usr/bin/dnf").exists() || Path::new("/usr/bin/yum").exists()
    }

    /// Check if the current distro supports AUR
    pub fn supports_aur(&self) -> bool {
        matches!(self, DistroType::Arch)
    }

    /// Check if the current distro should use Soar as fallback
    pub fn should_use_soar(&self) -> bool {
        !matches!(self, DistroType::Arch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distro_detect() {
        let distro = DistroType::detect();
        // We can't test specific distros in CI, but we can verify it doesn't panic
        match distro {
            DistroType::Arch => println!("Detected Arch-based system"),
            DistroType::Debian => println!("Detected Debian-based system"),
            DistroType::Fedora => println!("Detected Fedora-based system"),
            DistroType::Unknown => println!("Unable to detect distro"),
        }
    }

    #[test]
    fn test_supports_aur() {
        let arch = DistroType::Arch;
        assert!(arch.supports_aur());

        let debian = DistroType::Debian;
        assert!(!debian.supports_aur());
    }

    #[test]
    fn test_should_use_soar() {
        let arch = DistroType::Arch;
        assert!(!arch.should_use_soar());

        let debian = DistroType::Debian;
        assert!(debian.should_use_soar());
    }
}
