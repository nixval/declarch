use colored::*;
use miette::{miette, IntoDiagnostic, Result};
use std::collections::HashMap;
use std::process::Command;

pub fn get_installed_packages(quiet: bool) -> Result<HashMap<String, String>> {
    if !quiet { println!("\n{}", "-> Fetching installed packages from pacman...".blue()); }
    
    let pacman_output = Command::new("pacman")
        .arg("-Q")
        .output()
        .into_diagnostic()?;

    if !pacman_output.status.success() {
        let error_msg = String::from_utf8_lossy(&pacman_output.stderr);
        return Err(miette!("'pacman -Q' failed: {}", error_msg));
    }

    let output_str = String::from_utf8(pacman_output.stdout)
        .expect("Error: 'pacman -Q' output was not valid UTF-8.");
    
    let installed_map: HashMap<String, String> = output_str
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let name = parts.next()?.to_string();
            let version = parts.next()?.to_string();
            Some((name, version))
        })
        .collect();

    if !quiet { println!("-> Found {} installed packages.", installed_map.len()); }
    Ok(installed_map)
}

pub fn install_packages(packages: &Vec<String>, quiet: bool) -> Result<()> {
    if packages.is_empty() {
        if !quiet { println!("\n{}", "-> System is already in sync!".green()); }
        return Ok(());
    }

    println!("\n{}", "-> Packages to install/downgrade:".green());
    for pkg in packages {
        println!("   - {}", pkg.cyan());
    }

    println!("\n{}", "-> Running installation command...".blue());
    let mut install_cmd = Command::new("paru");
    install_cmd.arg("-S");
    install_cmd.arg("--needed");
    install_cmd.arg("--noconfirm");
    install_cmd.args(packages);

    let status = install_cmd.status().into_diagnostic()?;

    if status.success() {
        println!("\n{}", "-> Installation successful!".green());
        Ok(())
    } else {
        Err(miette!("Installation failed!"))
    }
}

pub fn remove_packages(packages_to_remove: &Vec<String>, quiet: bool) -> Result<()> {
    if packages_to_remove.is_empty() {
        if !quiet { println!("\n{}", "-> No packages to prune.".green()); }
        return Ok(());
    }
    
    let mut real_packages_to_remove: Vec<String> = Vec::new();
    if !quiet { println!("\n{}", "-> Resolving packages to prune...".blue()); }

    for pkg_name in packages_to_remove {
        let output = Command::new("pacman")
            .arg("-Q")
            .arg(pkg_name)
            .output()
            .into_diagnostic()?;
        
        if output.status.success() {
            let real_pkg_line = String::from_utf8(output.stdout)
                .expect("Error: 'pacman -Q' output was not valid UTF-8.");
            let real_pkg_name = real_pkg_line
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            
            if !real_pkg_name.is_empty() {
                real_packages_to_remove.push(real_pkg_name);
            }
        } else {
            if !quiet {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                if error_msg.contains("package") && error_msg.contains("was not found") {
                    println!("   - {} Package '{}' not found on system. Skipping removal.", "[Info]".dimmed(), pkg_name.cyan());
                } else {
                    println!("   - {} Failed to query package '{}': {}", "[Warning]".yellow(), pkg_name.cyan(), error_msg);
                }
            }
        }
    }

    if real_packages_to_remove.is_empty() {
        if !quiet { println!("\n{}", "-> No managed packages found on system to prune.".green()); }
        return Ok(());
    }

    println!("\n{}", "-> Pruning packages:".yellow());
    for pkg in &real_packages_to_remove {
        println!("   - {}", pkg.red());
    }
    
    println!("\n{}", "-> Running removal command...".blue());
    let mut prune_cmd = Command::new("paru");
    prune_cmd.arg("-Rns");
    prune_cmd.arg("--noconfirm");
    prune_cmd.args(&real_packages_to_remove);

    let status = prune_cmd.status().into_diagnostic()?;

    if status.success() {
        println!("\n{}", "-> Pruning successful!".green());
        Ok(())
    } else {
        Err(miette!("Pruning failed!"))
    }
}
