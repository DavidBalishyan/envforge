use std::process::Command;

use super::check_command_public;
use super::r#trait::PackageManager;

pub struct PacmanManager;

impl PackageManager for PacmanManager {
    fn install(&self, packages: &[String]) -> Vec<Result<(), String>> {
        packages
            .iter()
            .map(|pkg| {
                log::info!("pacman: installing package '{}'", pkg);
                let status = Command::new("sudo")
                    .arg("pacman")
                    .arg("-S")
                    .arg("--noconfirm")
                    .arg(pkg)
                    .status()
                    .map_err(|e| format!("failed to execute pacman: {}", e));

                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(_) => Err(format!("pacman install '{}' failed", pkg)),
                    Err(e) => Err(e),
                }
            })
            .collect()
    }

    fn remove(&self, packages: &[String]) -> Vec<Result<(), String>> {
        packages
            .iter()
            .map(|pkg| {
                log::info!("pacman: removing package '{}'", pkg);
                let status = Command::new("sudo")
                    .arg("pacman")
                    .arg("-Rs")
                    .arg("--noconfirm")
                    .arg(pkg)
                    .status()
                    .map_err(|e| format!("failed to execute pacman: {}", e));

                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(_) => Err(format!("pacman remove '{}' failed", pkg)),
                    Err(e) => Err(e),
                }
            })
            .collect()
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("pacman")
            .arg("-Q")
            .arg(package)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn name(&self) -> &'static str {
        "pacman"
    }

    fn is_available() -> bool {
        check_command_public("pacman")
    }
}
