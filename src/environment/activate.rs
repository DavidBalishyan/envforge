use std::collections::HashMap;

use anyhow::Result;

use crate::config::ProfileConfig;
use crate::executor::ShellExecutor;

pub enum ActivationMode {
    Subshell,
    Export,
}

pub fn activate_environment(
    config: &ProfileConfig,
    mode: ActivationMode,
    executor: &ShellExecutor,
) -> Result<()> {
    log::info!("activating profile: {}", config.name);

    match mode {
        ActivationMode::Subshell => spawn_subshell(config, executor),
        ActivationMode::Export => print_exports(config),
    }
}

fn spawn_subshell(config: &ProfileConfig, _executor: &ShellExecutor) -> Result<()> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let shell_name = std::path::Path::new(&shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("bash");

    log::info!("spawning {} subshell for profile '{}'", shell_name, config.name);

    let env_exports = build_env_exports(&config.env);

    let export_cmds = if env_exports.is_empty() {
        String::new()
    } else {
        let mut cmds = String::new();
        for (k, v) in &env_exports {
            cmds.push_str(&format!("export {}={}; ", escape_value(k), escape_value(v)));
        }
        cmds
    };

    let setup_cmds = if config.setup.is_empty() {
        String::new()
    } else {
        let cmds: Vec<String> = config.setup.iter().map(|c| c.to_string()).collect();
        cmds.join("; ")
    };

    let ps1 = format!("(envforge:{}) ", config.name);
    let ps1_export = format!("export PS1=\"{}$PS1\"; ", &ps1);

    let init = format!(
        "{}{}{}exec {}",
        export_cmds,
        ps1_export,
        if setup_cmds.is_empty() {
            String::new()
        } else {
            format!("{}; ", setup_cmds)
        },
        shell
    );

    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&init)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to spawn subshell: {}", e))?;

    if !status.success() {
        anyhow::bail!("subshell exited with code: {:?}", status.code());
    }

    Ok(())
}

fn print_exports(config: &ProfileConfig) -> Result<()> {
    let exports = build_env_exports(&config.env);
    for (k, v) in &exports {
        println!("export {}={}", k, v);
    }

    if !config.setup.is_empty() {
        eprintln!("warning: setup commands are not run in export mode");
    }

    Ok(())
}

fn build_env_exports(env: &HashMap<String, String>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for (k, v) in env {
        result.insert(k.clone(), v.clone());
    }
    result
}

fn escape_value(s: &str) -> String {
    if s.contains(' ') || s.contains('"') || s.contains('$') || s.contains('\\') {
        format!(
            "\"{}\"",
            s.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('$', "\\$")
        )
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_env_exports() {
        let mut env = HashMap::new();
        env.insert("CC".to_string(), "gcc".to_string());
        env.insert("CXX".to_string(), "g++".to_string());

        let exports = build_env_exports(&env);
        assert_eq!(exports.get("CC").unwrap(), "gcc");
        assert_eq!(exports.get("CXX").unwrap(), "g++");
    }

    #[test]
    fn test_escape_value() {
        assert_eq!(escape_value("simple"), "simple");
        assert_eq!(escape_value("has space"), r#""has space""#);
    }
}
