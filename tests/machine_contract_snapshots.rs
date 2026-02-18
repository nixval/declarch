use assert_cmd::Command;
use serde_json::{Value, json};
use std::fs;
use tempfile::TempDir;

fn declarch() -> Command {
    Command::new(env!("CARGO_BIN_EXE_declarch"))
}

struct ContractEnv {
    _tmp: TempDir,
    home_dir: std::path::PathBuf,
    xdg_config_home: std::path::PathBuf,
    xdg_state_home: std::path::PathBuf,
    xdg_cache_home: std::path::PathBuf,
    mock_bin_dir: std::path::PathBuf,
}

impl ContractEnv {
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
        fs::create_dir_all(config_dir.join("backends")).expect("mkdir backends");
        fs::create_dir_all(&xdg_state_home).expect("mkdir state");
        fs::create_dir_all(&xdg_cache_home).expect("mkdir cache");
        fs::create_dir_all(&mock_bin_dir).expect("mkdir bin");

        fs::write(
            config_dir.join("declarch.kdl"),
            "backends \"backends/mockpm.kdl\"\npkg { mockpm { alpha } }\n",
        )
        .expect("write root config");

        fs::write(
            config_dir.join("backends/mockpm.kdl"),
            r#"
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
"#,
        )
        .expect("write backend config");

        fs::write(
            mock_bin_dir.join("mockpm"),
            r#"#!/usr/bin/env bash
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
"#,
        )
        .expect("write mockpm script");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = mock_bin_dir.join("mockpm");
            let mut perms = fs::metadata(&path).expect("metadata").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).expect("chmod");
        }

        Self {
            _tmp: tmp,
            home_dir,
            xdg_config_home,
            xdg_state_home,
            xdg_cache_home,
            mock_bin_dir,
        }
    }

    fn apply(&self, cmd: &mut Command) {
        let old_path = std::env::var("PATH").unwrap_or_default();
        cmd.env("HOME", &self.home_dir)
            .env("XDG_CONFIG_HOME", &self.xdg_config_home)
            .env("XDG_STATE_HOME", &self.xdg_state_home)
            .env("XDG_CACHE_HOME", &self.xdg_cache_home)
            .env(
                "PATH",
                format!("{}:{}", self.mock_bin_dir.display(), old_path),
            );
    }
}

fn run_json(env: &ContractEnv, args: &[&str]) -> Value {
    let mut cmd = declarch();
    env.apply(&mut cmd);
    let assert = cmd.args(args).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    let mut parsed: Value = serde_json::from_str(&stdout).expect("json envelope");
    parsed["meta"]["generated_at"] = Value::String("<normalized>".to_string());
    parsed
}

#[test]
fn snapshot_info_v1_envelope() {
    let env = ContractEnv::new();
    let actual = run_json(
        &env,
        &["info", "--format", "json", "--output-version", "v1"],
    );
    let expected = json!({
      "version": "v1",
      "command": "info",
      "ok": true,
      "data": [],
      "warnings": [],
      "errors": [],
      "meta": { "generated_at": "<normalized>" }
    });
    assert_eq!(actual, expected);
}

#[test]
fn snapshot_info_list_v1_envelope() {
    let env = ContractEnv::new();
    let actual = run_json(
        &env,
        &[
            "info",
            "--list",
            "--format",
            "json",
            "--output-version",
            "v1",
        ],
    );
    let expected = json!({
      "version": "v1",
      "command": "info --list",
      "ok": true,
      "data": [],
      "warnings": [],
      "errors": [],
      "meta": { "generated_at": "<normalized>" }
    });
    assert_eq!(actual, expected);
}

#[test]
fn snapshot_lint_v1_envelope() {
    let env = ContractEnv::new();
    let actual = run_json(
        &env,
        &["lint", "--format", "json", "--output-version", "v1"],
    );
    let expected = json!({
      "version": "v1",
      "command": "lint",
      "ok": true,
      "data": {
        "mode": "all",
        "files_checked": 2,
        "total_issues": 0,
        "warnings_count": 0,
        "errors_count": 0,
        "issues": []
      },
      "warnings": [],
      "errors": [],
      "meta": { "generated_at": "<normalized>" }
    });
    assert_eq!(actual, expected);
}

#[test]
fn snapshot_search_v1_envelope() {
    let env = ContractEnv::new();
    let actual = run_json(
        &env,
        &[
            "search",
            "alpha",
            "--backends",
            "mockpm",
            "--format",
            "json",
            "--output-version",
            "v1",
        ],
    );
    let expected = json!({
      "version": "v1",
      "command": "search",
      "ok": true,
      "data": {
        "query": "alpha",
        "local": false,
        "requested_backends": ["mockpm"],
        "total_matches": 2,
        "shown_results": 2,
        "results": [
          {
            "backend": "mockpm",
            "name": "alpha",
            "version": null,
            "description": "1.0.0",
            "installed": false
          },
          {
            "backend": "mockpm",
            "name": "alpha-extra",
            "version": null,
            "description": "2.0.0",
            "installed": false
          }
        ]
      },
      "warnings": [],
      "errors": [],
      "meta": { "generated_at": "<normalized>" }
    });
    assert_eq!(actual, expected);
}

#[test]
fn snapshot_sync_dry_run_v1_envelope() {
    let env = ContractEnv::new();
    let actual = run_json(
        &env,
        &[
            "--dry-run",
            "sync",
            "--target",
            "mockpm",
            "--format",
            "json",
            "--output-version",
            "v1",
        ],
    );
    let expected = json!({
      "version": "v1",
      "command": "sync",
      "ok": true,
      "data": {
        "dry_run": true,
        "prune": false,
        "update": false,
        "target": "backend:mockpm",
        "install_count": 0,
        "remove_count": 0,
        "adopt_count": 1,
        "to_install": [],
        "to_remove": [],
        "to_adopt": ["mockpm:alpha"]
      },
      "warnings": [],
      "errors": [],
      "meta": { "generated_at": "<normalized>" }
    });
    assert_eq!(actual, expected);
}
