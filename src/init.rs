use colored::*;
use miette::{ IntoDiagnostic, Result};
use std::fs;
use std::path::Path;

fn create_dir(path: &Path, quiet: bool) -> Result<()> {
    if !path.exists() {
        if !quiet { println!("   - {} Creating directory: {:?}", "[OK]".green(), path); }
        fs::create_dir_all(path).into_diagnostic()?;
    } else {
        if !quiet { println!("   - {} Directory exists, skipping: {:?}", "[Skip]".yellow(), path); }
    }
    Ok(())
}

fn write_template(path: &Path, content: &str, quiet: bool) -> Result<()> {
    if path.exists() {
        if !quiet { println!("   - {} File exists, skipping: {:?}", "[Skip]".yellow(), path); }
        return Ok(());
    }
    if !quiet { println!("   - {} Creating file: {:?}", "[OK]".green(), path); }
    fs::write(path, content).into_diagnostic()?;
    Ok(())
}

pub fn run_init(config_dir: &Path, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{}", "-> Initializing new declarch configuration...".blue());
        println!("{}", format!("   Target directory: {:?}", config_dir).dimmed());
    }

    create_dir(config_dir, quiet)?;
    create_dir(&config_dir.join("modules"), quiet)?;
    create_dir(&config_dir.join("hosts"), quiet)?;

    let hostname = hostname::get()
        .into_diagnostic()?
        .to_string_lossy()
        .to_string();
    if !quiet { println!("-> Auto-detected hostname: {}", hostname.cyan()); }

    if !quiet { println!("-> Writing template files..."); }
    
    let config_kdl = format!("host \"{}\"\nenabled_modules \"base\"\n", hostname);
    write_template(&config_dir.join("config.kdl"), &config_kdl, quiet)?;

    write_template(&config_dir.join("state.json"), "{}\n", quiet)?;

    let base_kdl = "description \"Base packages for all machines\"\npackages git vim\n";
    write_template(&config_dir.join("modules/base.kdl"), base_kdl, quiet)?;
    
    let host_kdl = format!("description \"Packages specific to the {} host\"\npackages\n", hostname);
    write_template(&config_dir.join("hosts").join(format!("{}.kdl", hostname)), &host_kdl, quiet)?;

    if !quiet {
         println!("\n{}", "Initialization complete!".green());
         println!("You can now edit the KDL files in {:?} and run 'declarch sync'.", config_dir);
    }
    
    Ok(())
}
