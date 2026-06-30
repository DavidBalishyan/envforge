use std::process::Command;

fn main() {
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let is_dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let git_suffix = if is_dirty {
        format!("{} (dirty)", git_hash)
    } else {
        git_hash
    };

    let pkg_version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let full = format!("{} (git {})", pkg_version, git_suffix);

    println!("cargo:rustc-env=ENVFORGE_VERSION={}", full);
    println!("cargo:rerun-if-changed=.git/HEAD");
}
