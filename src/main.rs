mod cli;
mod config;
mod environment;
mod executor;
mod installer;

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use cli::commands::Commands;
use config::{list_profiles, load_profile, load_profile_from_path, remove_profile};
use environment::{ActivationMode, activate_environment};
use executor::ShellExecutor;
use installer::detect_package_manager;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    let executor = ShellExecutor::new(cli.dry_run, cli.verbose);

    match cli.command {
        Commands::Init { name } => cmd_init(&name, &executor),
        Commands::Create { path } => cmd_create(&path),
        Commands::Enter { profile, export } => cmd_enter(&profile, export, &executor),
        Commands::List => cmd_list(),
        Commands::Remove { profile } => cmd_remove(&profile),
        Commands::Doctor => cmd_doctor(),
    }
}

fn cmd_init(name: &str, executor: &ShellExecutor) -> Result<()> {
    let dir = config::profiles_dir()?;
    let path = dir.join(format!("{}.yaml", name));

    if path.exists() {
        anyhow::bail!("profile '{}' already exists at {}", name, path.display());
    }

    let template = format!(
        r#"name: "{}"

packages:
  # - gcc
  # - cmake

env:
  # CC: gcc
  # CXX: g++

setup:
  # - echo "setup complete"
"#,
        name
    );

    if executor.dry_run {
        log::info!("[dry-run] would create profile at: {}", path.display());
        log::info!("[dry-run] contents:\n{}", template);
        return Ok(());
    }

    fs::write(&path, &template)
        .with_context(|| format!("failed to write profile: {}", path.display()))?;

    log::info!("created profile '{}' at {}", name, path.display());
    Ok(())
}

fn cmd_create(path: &str) -> Result<()> {
    let config_path = PathBuf::from(path);

    if !config_path.exists() {
        anyhow::bail!("config file not found: {}", path);
    }

    let config = load_profile_from_path(&config_path)?;
    log::info!("loaded profile: {}", config.name);

    let dir = config::profiles_dir()?;
    let dest = dir.join(format!("{}.yaml", config.name));

    fs::copy(&config_path, &dest)
        .with_context(|| format!("failed to copy config to {}", dest.display()))?;

    log::info!("installed profile '{}' from {}", config.name, path);
    Ok(())
}

fn cmd_enter(profile: &str, export: bool, executor: &ShellExecutor) -> Result<()> {
    let config = if let Ok(cfg) = load_profile(profile) {
        cfg
    } else {
        let local_path = PathBuf::from(".envforge.yaml");
        if local_path.exists() {
            log::info!("profile '{}' not found, trying local .envforge.yaml", profile);
            load_profile_from_path(&local_path)?
        } else {
            anyhow::bail!("profile '{}' not found and no .envforge.yaml in current directory", profile);
        }
    };

    log::info!("loading profile '{}'", config.name);

    let mut installed_packages: Vec<String> = Vec::new();

    if !executor.dry_run {
        if !config.packages.is_empty() {
            if let Some(pm) = detect_package_manager() {
                log::info!("detected package manager: {}", pm.name());

                let to_install: Vec<String> = config
                    .packages
                    .iter()
                    .filter(|p| !pm.is_installed(p))
                    .cloned()
                    .collect();

                if to_install.is_empty() {
                    log::info!("all {} package(s) already installed, none to install", config.packages.len());
                } else {
                    let skipped = config.packages.len() - to_install.len();
                    if skipped > 0 {
                        log::info!("{} package(s) already installed, skipping", skipped);
                    }
                    log::info!("installing {} package(s)...", to_install.len());
                    let results = pm.install(&to_install);
                    for (i, result) in results.iter().enumerate() {
                        match result {
                            Ok(()) => {
                                log::info!("  ok {}", to_install[i]);
                                installed_packages.push(to_install[i].clone());
                            }
                            Err(e) => log::warn!("  fail {}: {}", to_install[i], e),
                        }
                    }
                }
            } else {
                log::warn!("no supported package manager found, skipping package installation");
            }
        }

        if !config.setup.is_empty() {
            log::info!("running setup commands...");
            executor.run_script(&config.setup)?;
        }
    }

    log::info!("activating environment...");

    if executor.dry_run {
        log::info!("[dry-run] would activate profile '{}'", profile);
        return Ok(());
    }

    let mode = if export {
        ActivationMode::Export
    } else {
        ActivationMode::Subshell
    };

    activate_environment(&config, mode, executor, &installed_packages)
}

fn cmd_list() -> Result<()> {
    let profiles = list_profiles()?;

    if profiles.is_empty() {
        println!("no profiles found in ~/.envforge/");
        println!("create one with: envforge init <name>");
        return Ok(());
    }

    println!("available profiles:");
    for profile in &profiles {
        let path = config::profiles_dir()?.join(format!("{}.yaml", profile));
        let config = load_profile(profile).ok();
        let pkg_count = config.as_ref().map(|c| c.packages.len()).unwrap_or(0);
        let env_count = config.as_ref().map(|c| c.env.len()).unwrap_or(0);
        println!(
            "  {}  (packages: {}, env vars: {}, config: {})",
            profile,
            pkg_count,
            env_count,
            path.display()
        );
    }

    Ok(())
}

fn cmd_remove(profile: &str) -> Result<()> {
    let dir = config::profiles_dir()?;
    let path = dir.join(format!("{}.yaml", profile));

    if !path.exists() {
        anyhow::bail!("profile '{}' not found", profile);
    }

    remove_profile(profile)?;
    log::info!("removed profile '{}'", profile);
    Ok(())
}

fn cmd_doctor() -> Result<()> {
    println!("envforge doctor - system diagnostics");
    println!("  version: {}", cli::commands::VERSION);
    println!();

    let config_dir = config::profiles_dir();
    match &config_dir {
        Ok(dir) => {
            println!("  config directory: {}", dir.display());
            if dir.exists() {
                let entries = fs::read_dir(dir).map(|e| e.count()).unwrap_or(0);
                println!("  profile files found: {}", entries);
            }
        }
        Err(e) => {
            println!("  config directory error: {}", e);
        }
    }

    println!();
    println!("package managers:");

    let managers = ["apt-get", "pacman", "brew"];
    for name in &managers {
        let available = installer::check_command_public(name);
        if available {
            println!("  {} available", name);
        } else {
            println!("  {} not found", name);
        }
    }

    println!();
    if let Some(pm) = detect_package_manager() {
        println!("active package manager: {}", pm.name());
    } else {
        println!("no supported package manager found");
    }

    println!();
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());
    println!("default shell: {}", shell);

    Ok(())
}
