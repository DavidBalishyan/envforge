use std::collections::HashMap;

use anyhow::Result;

use crate::config::ProfileConfig;
use crate::executor::ShellExecutor;
use crate::installer::detect_package_manager;

pub enum ActivationMode {
    Subshell,
    Export,
}

pub fn activate_environment(
    config: &ProfileConfig,
    mode: ActivationMode,
    executor: &ShellExecutor,
    installed_packages: &[String],
) -> Result<()> {
    log::info!("activating profile: {}", config.name);

    match mode {
        ActivationMode::Subshell => spawn_subshell(config, executor, installed_packages),
        ActivationMode::Export => print_exports(config),
    }
}

fn spawn_subshell(
    config: &ProfileConfig,
    _executor: &ShellExecutor,
    installed_packages: &[String],
) -> Result<()> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let shell_name = std::path::Path::new(&shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("bash");

    log::info!(
        "spawning {} subshell for profile '{}'",
        shell_name,
        config.name
    );

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

    // Create a shell init script that sources the user's rc and sets PS1,
    // so the envforge prompt prefix survives shell rc file overrides.
    let ps1_prefix = format!("(envforge:{}) ", config.name);
    let pid = std::process::id();
    let rc_path = format!("/tmp/envforge_{}_{}", config.name, pid);
    let mut extra_cleanup = String::new();

    let shell_invoke = match shell_name {
        "bash" => {
            let content = format!("source ~/.bashrc 2>/dev/null\nPS1=\"{}$PS1\"\n", ps1_prefix);
            std::fs::write(&rc_path, &content)
                .map_err(|e| anyhow::anyhow!("failed to write bash rc: {}", e))?;
            extra_cleanup = format!("rm -f {}", rc_path);
            format!("{} --rcfile {}", shell, rc_path)
        }
        "zsh" => {
            let zdotdir = format!("{}_zd", rc_path);
            std::fs::create_dir_all(&zdotdir)
                .map_err(|e| anyhow::anyhow!("failed to create zdotdir: {}", e))?;
            let content = format!("source ~/.zshrc 2>/dev/null\nPS1=\"{}$PS1\"\n", ps1_prefix);
            std::fs::write(format!("{}/.zshrc", zdotdir), &content)
                .map_err(|e| anyhow::anyhow!("failed to write zsh rc: {}", e))?;
            extra_cleanup = format!("rm -rf {}", zdotdir);
            format!("export ZDOTDIR={}; {}", zdotdir, shell)
        }
        _ => {
            format!("export PS1=\"{}$PS1\"; {}", &ps1_prefix, shell)
        }
    };

    // Build cleanup trap runs when the subshell exits
    // Only removes packages that envforge actually installed (not pre-existing ones)
    let cleanup_trap = if installed_packages.is_empty() && extra_cleanup.is_empty() {
        String::new()
    } else {
        let mut trap_body = String::new();
        if !extra_cleanup.is_empty() {
            trap_body.push_str(&extra_cleanup);
            trap_body.push_str("; ");
        }
        if !installed_packages.is_empty() {
            if let Some(pm) = detect_package_manager() {
                let cmd = match pm.name() {
                    "apt" => format!("sudo apt-get remove -y {}", installed_packages.join(" ")),
                    "pacman" => {
                        format!(
                            "sudo pacman -Rs --noconfirm {}",
                            installed_packages.join(" ")
                        )
                    }
                    "brew" => format!("brew remove {}", installed_packages.join(" ")),
                    _ => String::new(),
                };
                if !cmd.is_empty() {
                    trap_body.push_str(&cmd);
                }
            }
        }
        if trap_body.is_empty() {
            String::new()
        } else {
            format!("trap '{}' EXIT; ", trap_body.trim_end_matches("; "))
        }
    };

    let mut parts: Vec<String> = Vec::new();
    if !export_cmds.is_empty() {
        parts.push(export_cmds.trim_end_matches("; ").to_string());
    }
    if !setup_cmds.is_empty() {
        parts.push(setup_cmds.to_string());
    }
    if !cleanup_trap.is_empty() {
        parts.push(cleanup_trap.trim_end_matches("; ").to_string());
    }
    parts.push(shell_invoke);

    let init = parts.join("; ");

    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&init)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to spawn subshell: {}", e))?;

    // Clean up rc files if the trap didn't fire (e.g., SIGKILL)
    let _ = std::fs::remove_file(&rc_path);
    let _ = std::fs::remove_dir_all(format!("{}_zd", rc_path));

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
