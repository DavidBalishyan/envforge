pub mod apt;
pub mod brew;
pub mod pacman;
pub mod r#trait;

use std::process::Command;

pub use r#trait::PackageManager;

pub fn detect_package_manager() -> Option<Box<dyn PackageManager>> {
    if apt::AptManager::is_available() {
        Some(Box::new(apt::AptManager))
    } else if pacman::PacmanManager::is_available() {
        Some(Box::new(pacman::PacmanManager))
    } else if brew::BrewManager::is_available() {
        Some(Box::new(brew::BrewManager))
    } else {
        None
    }
}

pub fn check_command_public(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
