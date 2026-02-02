
    // Existing tests (unchanged for backward compatibility)

    #[test]
    fn aliases_inline() {
        let kdl = r#"
            aliases-pkg pipewire pipewire-jack2
            aliases-pkg python-poetry python-poetry-core
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.package_mappings.len(), 2);
        assert_eq!(
            config.package_mappings.get("pipewire"),
            Some(&"pipewire-jack2".to_string())
        );
        assert_eq!(
            config.package_mappings.get("python-poetry"),
            Some(&"python-poetry-core".to_string())
        );
    }

    #[test]
    fn aliases_block() {
        let kdl = r#"
            aliases-pkg {
                pipewire pipewire-jack2
                python-poetry python-poetry-core
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.package_mappings.len(), 2);
        assert_eq!(
            config.package_mappings.get("pipewire"),
            Some(&"pipewire-jack2".to_string())
        );
        assert_eq!(
            config.package_mappings.get("python-poetry"),
            Some(&"python-poetry-core".to_string())
        );
    }

    #[test]
    fn mixed_config() {
        let kdl = r#"
            packages {
                neovim
                hyprland
            }

            aliases-pkg pipewire pipewire-jack2

            excludes bad-package
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aur_packages().len(), 2);
        assert_eq!(config.package_mappings.len(), 1);
        assert_eq!(config.excludes.len(), 1);
        assert_eq!(
            config.package_mappings.get("pipewire"),
            Some(&"pipewire-jack2".to_string())
        );
    }

    #[test]
    fn empty_aliases() {
        let kdl = r#"
            packages neovim
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.package_mappings.is_empty());
    }

    #[test]
    fn default_packages() {
        let kdl = r#"
            packages {
                hyprland
                waybar
                swww
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aur_packages().len(), 3);
        assert!(config.aur_packages().iter().any(|p| p.name == "hyprland"));
        assert!(config.aur_packages().iter().any(|p| p.name == "waybar"));
        assert!(config.aur_packages().iter().any(|p| p.name == "swww"));
    }

    #[test]
    fn cross_distro() {
        let kdl = r#"
            // Cross-distro configuration example

            // AUR packages (default, Arch-only)
            packages {
                hyprland
                waybar
                swww
            }

            // Soar packages (cross-distro static binaries)
            soar-packages {
                bat
                exa
                fd
                ripgrep
            }

            // Flatpak packages (cross-distro)
            flatpak-packages {
                com.spotify.Client
                org.telegram.desktop
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aur_packages().len(), 3);
        assert_eq!(config.soar_packages().len(), 4);
        assert_eq!(config.flatpak_packages().len(), 2);
    }

    // New syntax tests

    #[test]
    fn soar_colon() {
        let kdl = r#"
            packages:soar {
                bat
                exa
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages().len(), 2);
        assert!(config.soar_packages().iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages().iter().any(|p| p.name == "exa"));
    }

    #[test]
    fn aur_colon() {
        let kdl = r#"
            packages:aur {
                hyprland
                waybar
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aur_packages().len(), 2);
        assert!(config.aur_packages().iter().any(|p| p.name == "hyprland"));
        assert!(config.aur_packages().iter().any(|p| p.name == "waybar"));
    }

    #[test]
    fn flatpak_colon() {
        let kdl = r#"
            packages:flatpak {
                com.spotify.Client
                org.mozilla.firefox
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.flatpak_packages().len(), 2);
        assert!(
            config
                .flatpak_packages()
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
        assert!(
            config
                .flatpak_packages()
                .iter()
                .any(|p| p.name == "org.mozilla.firefox")
        );
    }

    #[test]
    fn nested_blocks() {
        let kdl = r#"
            packages {
                hyprland
                waybar
                soar {
                    bat
                    exa
                }
                flatpak {
                    com.spotify.Client
                    org.mozilla.firefox
                }
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aur_packages().len(), 2);
        assert!(config.aur_packages().iter().any(|p| p.name == "hyprland"));
        assert!(config.aur_packages().iter().any(|p| p.name == "waybar"));

        assert_eq!(config.soar_packages().len(), 2);
        assert!(config.soar_packages().iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages().iter().any(|p| p.name == "exa"));

        assert_eq!(config.flatpak_packages().len(), 2);
        assert!(
            config
                .flatpak_packages()
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
        assert!(
            config
                .flatpak_packages()
                .iter()
                .any(|p| p.name == "org.mozilla.firefox")
        );
    }

    #[test]
    fn mixed_formats() {
        let kdl = r#"
            // Default packages (AUR)
            packages {
                hyprland
                waybar
            }

            // Colon syntax for Soar
            packages:soar {
                bat
            }

            // Colon syntax for Flatpak
            packages:flatpak {
                com.spotify.Client
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aur_packages().len(), 2);
        assert_eq!(config.soar_packages().len(), 1);
        assert_eq!(config.flatpak_packages().len(), 1);
    }

    #[test]
    fn soar_nested() {
        let kdl = r#"
            packages {
                soar {
                    bat
                    exa
                }
                flatpak {
                    com.spotify.Client
                }
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages().len(), 2);
        assert!(config.soar_packages().iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages().iter().any(|p| p.name == "exa"));

        assert_eq!(config.flatpak_packages().len(), 1);
        assert!(
            config
                .flatpak_packages()
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
    }

    // NEW TESTS: Inline prefix syntax

    #[test]
    fn inline_single() {
        let kdl = r#"
            packages {
                soar:bat
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages().len(), 1);
        assert!(config.soar_packages().iter().any(|p| p.name == "bat"));
    }

    #[test]
    fn inline_multiple() {
        let kdl = r#"
            packages {
                hyprland
                aur:waybar
                soar:bat
                soar:exa
                flatpak:com.spotify.Client
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        // Default (hyprland) + aur:waybar
        assert_eq!(config.aur_packages().len(), 2);
        assert!(config.aur_packages().iter().any(|p| p.name == "hyprland"));
        assert!(config.aur_packages().iter().any(|p| p.name == "waybar"));

        assert_eq!(config.soar_packages().len(), 2);
        assert!(config.soar_packages().iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages().iter().any(|p| p.name == "exa"));

        assert_eq!(config.flatpak_packages().len(), 1);
        assert!(
            config
                .flatpak_packages()
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
    }

    #[test]
    fn inline_nested() {
        let kdl = r#"
            packages {
                hyprland
                aur:waybar
                soar {
                    bat
                }
                flatpak:com.spotify.Client
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aur_packages().len(), 2); // hyprland + waybar
        assert_eq!(config.soar_packages().len(), 1); // bat
        assert_eq!(config.flatpak_packages().len(), 1); // com.spotify.Client
    }

    #[test]
    fn inline_alias() {
        let kdl = r#"
            packages {
                app:bat
                app:exa
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages().len(), 2);
        assert!(config.soar_packages().iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages().iter().any(|p| p.name == "exa"));
    }

    #[test]
    fn string_args() {
        let kdl = r#"
            packages "soar:bat" "aur:hyprland" "flatpak:app.id"
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages().len(), 1);
        assert!(config.soar_packages().iter().any(|p| p.name == "bat"));

        assert_eq!(config.aur_packages().len(), 1);
        assert!(config.aur_packages().iter().any(|p| p.name == "hyprland"));

        assert_eq!(config.flatpak_packages().len(), 1);
        assert!(config.flatpak_packages().iter().any(|p| p.name == "app.id"));
    }

    #[test]
    fn unknown_backend() {
        let kdl = r#"
            packages {
                unknown:package
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        // Unknown backend should be treated as package name with default backend
        assert!(config.aur_packages().iter().any(|p| p.name == "unknown:package"));
    }

    #[test]
    fn complex_config() {
        let kdl = r#"
            packages {
                // Default packages (AUR)
                hyprland
                waybar

                // Inline prefix syntax
                soar:bat
                flatpak:com.spotify.Client

                // Nested blocks
                aur {
                    swww
                }

                // Mixed inline and nested
                soar {
                    exa
                }
                aur:rofi
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // AUR packages: hyprland, waybar, swww, rofi
        assert_eq!(config.aur_packages().len(), 4);
        assert!(config.aur_packages().iter().any(|p| p.name == "hyprland"));
        assert!(config.aur_packages().iter().any(|p| p.name == "waybar"));
        assert!(config.aur_packages().iter().any(|p| p.name == "swww"));
        assert!(config.aur_packages().iter().any(|p| p.name == "rofi"));

        // Soar packages: bat, exa
        assert_eq!(config.soar_packages().len(), 2);
        assert!(config.soar_packages().iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages().iter().any(|p| p.name == "exa"));

        // Flatpak packages: com.spotify.Client
        assert_eq!(config.flatpak_packages().len(), 1);
        assert!(
            config
                .flatpak_packages()
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
    }

    #[test]
    fn backend_registry() {
        let registry = BackendParserRegistry::new();

        // Test finding parsers by name
        assert!(registry.find_parser("aur").is_some());
        assert!(registry.find_parser("soar").is_some());
        assert!(registry.find_parser("flatpak").is_some());
        assert!(registry.find_parser("unknown").is_none());

        // Test aliases
        assert!(registry.find_parser("app").is_some()); // alias for soar
    }

    #[test]
    fn backward_compat() {
        // Ensure all old syntax still works
        let kdl = r#"
            packages {
                hyprland
                waybar
            }

            packages:soar {
                bat
            }

            packages:flatpak {
                com.spotify.Client
            }

            soar-packages {
                exa
            }

            flatpak-packages {
                org.mozilla.firefox
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aur_packages().len(), 2);
        assert_eq!(config.soar_packages().len(), 2); // bat + exa
        assert_eq!(config.flatpak_packages().len(), 2);
    }

    // NEW: Meta block tests

    #[test]
    fn meta_block() {
        let kdl = r#"
            meta {
                description "My Hyprland Setup"
                author "nixval"
                version "1.0.0"
                url "https://github.com/nixval/dotfiles"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(
            config.project_metadata.description,
            Some("My Hyprland Setup".to_string())
        );
        assert_eq!(config.project_metadata.author, Some("nixval".to_string()));
        assert_eq!(config.project_metadata.version, Some("1.0.0".to_string()));
        assert_eq!(
            config.project_metadata.url,
            Some("https://github.com/nixval/dotfiles".to_string())
        );
    }

    #[test]
    fn meta_tags() {
        let kdl = r#"
            meta {
                description "Workstation setup"
                tags "workstation" "hyprland" "development"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.project_metadata.tags.len(), 3);
        assert!(
            config
                .project_metadata
                .tags
                .contains(&"workstation".to_string())
        );
        assert!(
            config
                .project_metadata
                .tags
                .contains(&"hyprland".to_string())
        );
        assert!(
            config
                .project_metadata
                .tags
                .contains(&"development".to_string())
        );
    }

    // NEW: Conflicts tests

    #[test]
    fn conflicts() {
        let kdl = r#"
            conflicts vim neovim
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.conflicts.len(), 1);
        assert_eq!(config.conflicts[0].packages.len(), 2);
        assert!(config.conflicts[0].packages.contains(&"vim".to_string()));
        assert!(config.conflicts[0].packages.contains(&"neovim".to_string()));
    }

    // NEW: Backend options tests

    #[test]
    fn backend_options() {
        let kdl = r#"
            options:aur {
                noconfirm
                helper "paru"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.backend_options.contains_key("aur"));
        let aur_opts = &config.backend_options["aur"];
        assert_eq!(aur_opts.get("noconfirm"), Some(&"true".to_string()));
        assert_eq!(aur_opts.get("helper"), Some(&"paru".to_string()));
    }

    // NEW: Environment variables tests

    #[test]
    fn env_vars() {
        let kdl = r#"
            env EDITOR="nvim" VISUAL="nvim"

            env:aur MAKEFLAGS="-j4"
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.env.contains_key("global"));
        assert!(config.env.contains_key("aur"));

        let global_env = &config.env["global"];
        assert!(global_env.contains(&"EDITOR=nvim".to_string()));
        assert!(global_env.contains(&"VISUAL=nvim".to_string()));

        let aur_env = &config.env["aur"];
        assert!(aur_env.contains(&"MAKEFLAGS=-j4".to_string()));
    }

    // NEW: Repositories tests

    #[test]
    fn repositories() {
        let kdl = r#"
            repos:aur {
                "https://aur.archlinux.org"
            }

            repos:flatpak {
                "https://flathub.org/repo/flathub.flatpakrepo"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.package_sources.contains_key("aur"));
        assert!(config.package_sources.contains_key("flatpak"));

        assert!(config.package_sources["aur"].contains(&"https://aur.archlinux.org".to_string()));
        assert!(
            config.package_sources["flatpak"]
                .contains(&"https://flathub.org/repo/flathub.flatpakrepo".to_string())
        );
    }

    // NEW: Policy tests

    #[test]
    fn policy() {
        let kdl = r#"
            policy {
                protected {
                    linux
                    systemd
                    base-devel
                }
                orphans "ask"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.policy.protected.contains("linux"));
        assert!(config.policy.protected.contains("systemd"));
        assert!(config.policy.protected.contains("base-devel"));
        assert_eq!(config.policy.orphans, Some("ask".to_string()));
    }

    // NEW: Hooks tests

    #[test]
    fn hooks() {
        let kdl = r#"
            hooks {
                post-sync "notify-send 'Packages updated'"
                post-sync "systemctl restart gdm" --sudo
                post-sync "~/.config/declarch/post-sync.sh"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // Filter post-sync hooks
        let post_sync_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::PostSync)
            .collect();

        assert_eq!(post_sync_hooks.len(), 3);

        assert_eq!(post_sync_hooks[0].command, "notify-send 'Packages updated'");
        assert_eq!(post_sync_hooks[0].action_type, ActionType::User);

        assert_eq!(post_sync_hooks[1].command, "systemctl restart gdm");
        assert_eq!(post_sync_hooks[1].action_type, ActionType::Root);

        assert_eq!(
            post_sync_hooks[2].command,
            "~/.config/declarch/post-sync.sh"
        );
        assert_eq!(post_sync_hooks[2].action_type, ActionType::User);
    }

    // NEW: Comprehensive integration test

    #[test]
    fn full_config() {
        let kdl = r#"
            meta {
                description "Full workstation setup"
                author "nixval"
                version "2.0.0"
            }

            packages {
                hyprland
                neovim
                waybar
            }

            packages:soar {
                bat
                exa
            }

            conflicts {
                vim neovim
                pipewire pulseaudio
            }

            options:aur {
                noconfirm
            }

            env EDITOR="nvim"

            policy {
                protected {
                    linux
                    systemd
                }
                orphans "keep"
            }

            hooks {
                post-sync "notify-send 'Sync complete'"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // Check project metadata
        assert_eq!(
            config.project_metadata.description,
            Some("Full workstation setup".to_string())
        );
        assert_eq!(config.project_metadata.author, Some("nixval".to_string()));
        assert_eq!(config.project_metadata.version, Some("2.0.0".to_string()));

        // Check packages
        assert_eq!(config.aur_packages().len(), 3);
        assert!(config.aur_packages().iter().any(|p| p.name == "hyprland"));
        assert!(config.aur_packages().iter().any(|p| p.name == "neovim"));
        assert!(config.aur_packages().iter().any(|p| p.name == "waybar"));

        // Check conflicts (1 conflict entry with 4 packages all mutually exclusive)
        assert_eq!(config.conflicts.len(), 1);
        assert_eq!(config.conflicts[0].packages.len(), 4);

        // Check options
        assert!(config.backend_options.contains_key("aur"));

        // Check env
        assert!(config.env.contains_key("global"));

        // Check policy
        assert!(config.policy.protected.contains("linux"));
        assert_eq!(config.policy.orphans, Some("keep".to_string()));

        // Check hooks
        let post_sync_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::PostSync)
            .collect();
        assert_eq!(post_sync_hooks.len(), 1);
    }

    // NEW: Flat hooks syntax test

    #[test]
    fn hooks_flat() {
        let kdl = r#"
            on-sync "notify-send 'Packages updated'"
            on-sync-sudo "systemctl restart gdm"
            on-pre-sync "echo 'Starting sync...'"
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // Check pre-sync hooks
        let pre_sync_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::PreSync)
            .collect();
        assert_eq!(pre_sync_hooks.len(), 1);
        assert_eq!(pre_sync_hooks[0].command, "echo 'Starting sync...'");
        assert_eq!(pre_sync_hooks[0].action_type, ActionType::User);

        // Check post-sync hooks
        let post_sync_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::PostSync)
            .collect();
        assert_eq!(post_sync_hooks.len(), 2);
        assert_eq!(post_sync_hooks[0].command, "notify-send 'Packages updated'");
        assert_eq!(post_sync_hooks[0].action_type, ActionType::User);
        assert_eq!(post_sync_hooks[1].command, "systemctl restart gdm");
        assert_eq!(post_sync_hooks[1].action_type, ActionType::Root);
    }

    // NEW: Mixed hooks (flat syntax + hooks block)

    #[test]
    fn hooks_mixed() {
        let kdl = r#"
            on-sync "notify-send 'Flat hook'"

            hooks {
                post-sync "notify-send 'Nested hook'"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // Filter post-sync hooks
        let post_sync_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::PostSync)
            .collect();

        // Should have both flat and nested hooks
        assert_eq!(post_sync_hooks.len(), 2);
        assert!(
            post_sync_hooks
                .iter()
                .any(|h| h.command == "notify-send 'Flat hook'")
        );
        assert!(
            post_sync_hooks
                .iter()
                .any(|h| h.command == "notify-send 'Nested hook'")
        );
    }

    // Comprehensive hooks tests

    #[test]
    fn hooks_comprehensive() {
        let kdl = r#"
meta {
    description "Test all hooks features"
    author "nixval"
    version "1.0.0"
}

packages {
    bat
}

hooks {
    // Global hooks
    pre-sync "echo 'Pre-sync'"
    post-sync "echo 'Post-sync'"
    on-success "echo 'Success'"
    on-failure "echo 'Failed'"

    // Package hook (block syntax)
    bat {
        post-install "echo 'Bat installed'"
    }

    // Backend hooks
    aur:post-sync "echo 'AUR synced'"
}
"#;

        let config = parse_kdl_content(kdl).unwrap();

        // Test pre-sync hooks
        let pre_sync_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::PreSync)
            .collect();
        assert_eq!(pre_sync_hooks.len(), 1);
        assert_eq!(pre_sync_hooks[0].command, "echo 'Pre-sync'");
        assert!(pre_sync_hooks[0].package.is_none());

        // Test post-sync hooks
        let post_sync_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::PostSync)
            .collect();
        assert_eq!(post_sync_hooks.len(), 2);
        assert!(
            post_sync_hooks
                .iter()
                .any(|h| h.command == "echo 'Post-sync'")
        );
        assert!(
            post_sync_hooks
                .iter()
                .any(|h| h.command == "echo 'AUR synced'")
        );

        // Test on-success hooks
        let on_success_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::OnSuccess)
            .collect();
        assert_eq!(on_success_hooks.len(), 1);
        assert_eq!(on_success_hooks[0].command, "echo 'Success'");

        // Test on-failure hooks
        let on_failure_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::OnFailure)
            .collect();
        assert_eq!(on_failure_hooks.len(), 1);
        assert_eq!(on_failure_hooks[0].command, "echo 'Failed'");

        // Test post-install hooks
        let post_install_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::PostInstall)
            .collect();
        assert_eq!(post_install_hooks.len(), 1);
        assert_eq!(post_install_hooks[0].command, "echo 'Bat installed'");
        assert_eq!(post_install_hooks[0].package.as_ref().unwrap(), "bat");
    }

    #[test]
    fn hooks_flags() {
        let kdl = r#"
hooks {
    post-sync "echo 'Regular hook'"
    post-sync "systemctl restart gdm" --sudo
    docker:post-install "mkinitcpio -P" --sudo --required
    post-sync "cleanup.sh" --ignore
}
"#;

        let config = parse_kdl_content(kdl).unwrap();

        let all_hooks = &config.lifecycle_actions.actions;

        // Regular hook (no sudo)
        let regular = all_hooks
            .iter()
            .find(|h| h.command.contains("Regular hook"))
            .unwrap();
        assert_eq!(regular.action_type, ActionType::User);
        assert_eq!(regular.error_behavior, ErrorBehavior::Warn);

        // Sudo hook
        let sudo = all_hooks
            .iter()
            .find(|h| h.command.contains("systemctl restart gdm"))
            .unwrap();
        assert_eq!(sudo.action_type, ActionType::Root);

        // Required hook
        let required = all_hooks
            .iter()
            .find(|h| h.command.contains("mkinitcpio"))
            .unwrap();
        assert_eq!(required.action_type, ActionType::Root);
        assert_eq!(required.error_behavior, ErrorBehavior::Required);

        // Ignore hook
        let ignore = all_hooks
            .iter()
            .find(|h| h.command.contains("cleanup.sh"))
            .unwrap();
        assert_eq!(ignore.error_behavior, ErrorBehavior::Ignore);
    }

    #[test]
    fn hooks_shorthand() {
        let kdl = r#"
packages {
    docker
    waybar
}

hooks {
    // Shorthand syntax
    docker:post-install "systemctl enable docker" --sudo
    waybar:post-install "pkill waybar"

    // Backend shorthand
    aur:post-sync "echo 'AUR done'"
    flatpak:post-sync "echo 'Flatpak done'"
}
"#;

        let config = parse_kdl_content(kdl).unwrap();

        // Test package shorthand
        let docker_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.package.as_deref() == Some("docker"))
            .collect();
        assert_eq!(docker_hooks.len(), 1);
        assert_eq!(docker_hooks[0].phase, LifecyclePhase::PostInstall);
        assert_eq!(docker_hooks[0].action_type, ActionType::Root);

        let waybar_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.package.as_deref() == Some("waybar"))
            .collect();
        assert_eq!(waybar_hooks.len(), 1);
        assert_eq!(waybar_hooks[0].phase, LifecyclePhase::PostInstall);

        // Test backend shorthand (should be detected by post-sync phase)
        let aur_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::PostSync)
            .filter(|h| h.command.contains("AUR"))
            .collect();
        assert_eq!(aur_hooks.len(), 1);

        let flatpak_hooks: Vec<_> = config
            .lifecycle_actions
            .actions
            .iter()
            .filter(|h| h.phase == LifecyclePhase::PostSync)
            .filter(|h| h.command.contains("Flatpak"))
            .collect();
        assert_eq!(flatpak_hooks.len(), 1);
    }
