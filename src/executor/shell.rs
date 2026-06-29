use std::process::Command;

use anyhow::{Context, Result};

pub struct ShellExecutor {
    pub dry_run: bool,
    pub verbose: bool,
}

impl ShellExecutor {
    pub fn new(dry_run: bool, verbose: bool) -> Self {
        Self { dry_run, verbose }
    }

    pub fn run(&self, cmd: &str) -> Result<String> {
        log::info!("exec: {}", cmd);

        if self.dry_run {
            log::info!("[dry-run] would execute: {}", cmd);
            return Ok(String::new());
        }

        if self.verbose {
            log::debug!("running command: {}", cmd);
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .with_context(|| format!("failed to execute: {}", cmd))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !stdout.is_empty() && self.verbose {
            log::debug!("stdout:\n{}", stdout);
        }

        if !stderr.is_empty() {
            if output.status.success() {
                log::warn!("stderr:\n{}", stderr);
            } else {
                log::error!("stderr:\n{}", stderr);
            }
        }

        if !output.status.success() {
            anyhow::bail!(
                "command failed (exit code: {:?}): {}\n  stderr: {}",
                output.status.code(),
                cmd,
                stderr.trim()
            );
        }

        Ok(stdout)
    }

    pub fn run_script(&self, commands: &[String]) -> Result<()> {
        for (i, cmd) in commands.iter().enumerate() {
            log::info!("setup[{}]: {}", i + 1, cmd);
            self.run(cmd)?;
        }
        Ok(())
    }
}
