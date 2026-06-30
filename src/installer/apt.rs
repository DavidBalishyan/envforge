use std::process::Command;

use super::check_command_public;
use super::r#trait::PackageManager;

pub struct AptManager;

impl PackageManager for AptManager {
    fn install(&self, packages: &[String]) -> Vec<Result<(), String>> {
        packages
            .iter()
            .map(|pkg| {
                log::info!("apt: installing package '{}'", pkg);
                let status = Command::new("sudo")
                    .arg("apt-get")
                    .arg("install")
                    .arg("-y")
                    .arg(pkg)
                    .status()
                    .map_err(|e| format!("failed to execute apt-get: {}", e));

                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(_) => Err(format!("apt-get install '{}' failed", pkg)),
                    Err(e) => Err(e),
                }
            })
            .collect()
    }

    fn remove(&self, packages: &[String]) -> Vec<Result<(), String>> {
        packages
            .iter()
            .map(|pkg| {
                log::info!("apt: removing package '{}'", pkg);
                let status = Command::new("sudo")
                    .arg("apt-get")
                    .arg("remove")
                    .arg("-y")
                    .arg(pkg)
                    .status()
                    .map_err(|e| format!("failed to execute apt-get: {}", e));

                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(_) => Err(format!("apt-get remove '{}' failed", pkg)),
                    Err(e) => Err(e),
                }
            })
            .collect()
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("dpkg")
            .arg("-s")
            .arg(package)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn name(&self) -> &'static str {
        "apt"
    }

    fn is_available() -> bool {
        check_command_public("apt-get")
    }
}
