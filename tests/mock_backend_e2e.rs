use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn declarch() -> Command {
    Command::new(env!("CARGO_BIN_EXE_declarch"))
}

struct TestEnv {
    _tmp: TempDir,
    home_dir: std::path::PathBuf,
    xdg_config_home: std::path::PathBuf,
    xdg_state_home: std::path::PathBuf,
    xdg_cache_home: std::path::PathBuf,
    config_dir: std::path::PathBuf,
    mock_bin_dir: std::path::PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().to_path_buf();

        let home_dir = root.join("home");
        let xdg_config_home = root.join("config");
        let xdg_state_home = root.join("state");
        let xdg_cache_home = root.join("cache");
        let config_dir = xdg_config_home.join("declarch");
        let mock_bin_dir = root.join("bin");

        fs::create_dir_all(&home_dir).expect("mkdir home");
        fs::create_dir_all(&xdg_config_home).expect("mkdir config");
        fs::create_dir_all(&xdg_state_home).expect("mkdir state");
        fs::create_dir_all(&xdg_cache_home).expect("mkdir cache");
        fs::create_dir_all(&config_dir).expect("mkdir declarch config");
        fs::create_dir_all(config_dir.join("backends")).expect("mkdir backends dir");
        fs::create_dir_all(&mock_bin_dir).expect("mkdir bin dir");

        let root_kdl = r#"
backends "backends/mockpm.kdl"

pkg {
  mockpm { alpha }
}
"#;
        fs::write(config_dir.join("declarch.kdl"), root_kdl).expect("write declarch.kdl");

        let backend_kdl = r#"
backend "mockpm" {
  binary "mockpm"

  list "{binary} list" {
    format "whitespace"
    name_col 0
    version_col 1
  }

  search "{binary} search {query}" {
    format "whitespace"
    name_col 0
    version_col 1
  }

  search_local "{binary} search-local {query}" {
    format "whitespace"
    name_col 0
    version_col 1
  }

  install "{binary} install {packages}"
  remove "{binary} remove {packages}"
  update "{binary} update"
  upgrade "{binary} upgrade"
  cache_clean "{binary} cache-clean"
}
"#;
        fs::write(config_dir.join("backends/mockpm.kdl"), backend_kdl).expect("write backend");

        let mock_bin = mock_bin_dir.join("mockpm");
        let script = r#"#!/usr/bin/env bash
set -euo pipefail
sub="${1:-}"
case "$sub" in
  list)
    echo "alpha 1.0.0"
    ;;
  search)
    q="${2:-alpha}"
    echo "$q 1.0.0"
    echo "alpha-extra 2.0.0"
    ;;
  search-local)
    q="${2:-alpha}"
    echo "$q 1.0.0"
    ;;
  install|remove|update|upgrade|cache-clean)
    exit 0
    ;;
  *)
    exit 0
    ;;
esac
"#;
        fs::write(&mock_bin, script).expect("write mock binary");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&mock_bin).expect("metadata").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&mock_bin, perms).expect("chmod");
        }

        Self {
            _tmp: tmp,
            home_dir,
            xdg_config_home,
            xdg_state_home,
            xdg_cache_home,
            config_dir,
            mock_bin_dir,
        }
    }

    fn apply(&self, cmd: &mut Command) {
        cmd.env("HOME", &self.home_dir)
            .env("XDG_CONFIG_HOME", &self.xdg_config_home)
            .env("XDG_STATE_HOME", &self.xdg_state_home)
            .env("XDG_CACHE_HOME", &self.xdg_cache_home);

        let old_path = std::env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", self.mock_bin_dir.display(), old_path);
        cmd.env("PATH", new_path);
    }
}

#[test]
fn e2e_search_with_mock_backend() {
    let env = TestEnv::new();

    let mut cmd = declarch();
    env.apply(&mut cmd);

    cmd.arg("search")
        .arg("alpha")
        .arg("--backends")
        .arg("mockpm")
        .arg("--limit")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("mockpm:"))
        .stdout(predicate::str::contains("alpha"));
}

#[test]
fn e2e_sync_preview_with_mock_backend() {
    let env = TestEnv::new();

    let mut cmd = declarch();
    env.apply(&mut cmd);

    cmd.arg("sync")
        .arg("preview")
        .arg("--target")
        .arg("mockpm")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("adopt")
                .or(predicate::str::contains("Everything is up to date!")),
        );
}

#[test]
fn e2e_install_no_sync_updates_module_file() {
    let env = TestEnv::new();

    let mut cmd = declarch();
    env.apply(&mut cmd);

    cmd.arg("install")
        .arg("mockpm:beta")
        .arg("--no-sync")
        .assert()
        .success();

    let root_content =
        fs::read_to_string(env.config_dir.join("declarch.kdl")).expect("read declarch.kdl");
    let others_path = env.config_dir.join("modules/others.kdl");
    let others_content = fs::read_to_string(&others_path).unwrap_or_default();

    assert!(
        root_content.contains("beta") || others_content.contains("beta"),
        "expected install command to persist package entry in root or modules/others.kdl"
    );
}
