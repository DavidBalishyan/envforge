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

    fn name(&self) -> &'static str {
        "pacman"
    }

    fn is_available() -> bool {
        check_command_public("pacman")
    }
}
