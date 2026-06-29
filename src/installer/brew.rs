use std::process::Command;

use super::check_command_public;
use super::r#trait::PackageManager;

pub struct BrewManager;

impl PackageManager for BrewManager {
    fn install(&self, packages: &[String]) -> Vec<Result<(), String>> {
        packages
            .iter()
            .map(|pkg| {
                log::info!("brew: installing package '{}'", pkg);
                let status = Command::new("brew")
                    .arg("install")
                    .arg(pkg)
                    .status()
                    .map_err(|e| format!("failed to execute brew: {}", e));

                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(_) => Err(format!("brew install '{}' failed", pkg)),
                    Err(e) => Err(e),
                }
            })
            .collect()
    }

    fn name(&self) -> &'static str {
        "brew"
    }

    fn is_available() -> bool {
        check_command_public("brew")
    }
}
