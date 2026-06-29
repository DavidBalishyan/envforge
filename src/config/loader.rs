use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileConfig {
    pub name: String,

    #[serde(default)]
    pub packages: Vec<String>,

    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub setup: Vec<String>,
}

pub fn profiles_dir() -> Result<PathBuf> {
    let base = dirs::home_dir().context("unable to determine home directory")?;
    let dir = base.join(".envforge");
    fs::create_dir_all(&dir).context("failed to create ~/.envforge directory")?;
    Ok(dir)
}

pub fn load_profile(name: &str) -> Result<ProfileConfig> {
    let dir = profiles_dir()?;
    let path = dir.join(format!("{}.yaml", name));
    load_profile_from_path(&path)
}

pub fn load_profile_from_path(path: &PathBuf) -> Result<ProfileConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;

    let config: ProfileConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse YAML config: {}", path.display()))?;

    if config.name.is_empty() {
        anyhow::bail!("profile name must not be empty");
    }

    Ok(config)
}

pub fn list_profiles() -> Result<Vec<String>> {
    let dir = profiles_dir()?;
    let mut profiles = Vec::new();

    if !dir.exists() {
        return Ok(profiles);
    }

    for entry in fs::read_dir(&dir).context("failed to read profiles directory")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "yaml") {
            if let Some(stem) = path.file_stem() {
                profiles.push(stem.to_string_lossy().to_string());
            }
        }
    }

    profiles.sort();
    Ok(profiles)
}

pub fn remove_profile(name: &str) -> Result<()> {
    let dir = profiles_dir()?;
    let path = dir.join(format!("{}.yaml", name));

    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("failed to remove profile: {}", name))?;
    }

    Ok(())
}
