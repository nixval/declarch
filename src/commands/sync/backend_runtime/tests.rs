use super::*;
use crate::core::types::{Backend as CoreBackend, PackageMetadata};
use crate::packages::traits::PackageManager;
use chrono::Utc;
use std::collections::HashMap;

struct MockManager {
    backend: CoreBackend,
    available: bool,
    installed: HashMap<String, PackageMetadata>,
}

impl PackageManager for MockManager {
    fn backend_type(&self) -> CoreBackend {
        self.backend.clone()
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        Ok(self.installed.clone())
    }

    fn install(&self, _packages: &[String]) -> Result<()> {
        Ok(())
    }

    fn remove(&self, _packages: &[String]) -> Result<()> {
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn get_required_by(&self, _package: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

#[test]
fn refresh_installed_snapshot_skips_unavailable_backends() {
    let mut managers: ManagerMap = HashMap::new();
    let available_backend = CoreBackend::from("flatpak");
    let unavailable_backend = CoreBackend::from("brew");

    let mut installed = HashMap::new();
    installed.insert(
        "com.spotify.Client".to_string(),
        PackageMetadata {
            version: Some("1.2.65".to_string()),
            variant: None,
            installed_at: Utc::now(),
            source_file: None,
        },
    );

    managers.insert(
        available_backend.clone(),
        Box::new(MockManager {
            backend: available_backend.clone(),
            available: true,
            installed,
        }),
    );
    managers.insert(
        unavailable_backend.clone(),
        Box::new(MockManager {
            backend: unavailable_backend,
            available: false,
            installed: HashMap::new(),
        }),
    );

    let snapshot = refresh_installed_snapshot(&managers);
    assert_eq!(snapshot.len(), 1);
    assert!(snapshot.keys().any(|k| k.backend == available_backend));
}
