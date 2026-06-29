# Walkthrough for developers

This document explains how envforge is built and how to extend it.

## Project layout

```
src/
  main.rs                 -- entry point, CLI dispatch, command implementations
  cli/
    commands.rs           -- clap argument definitions
  config/
    loader.rs             -- YAML parsing, profile storage in ~/.envforge/
  installer/
    trait.rs              -- PackageManager trait definition
    mod.rs                -- OS detection and backend selection
    apt.rs                -- apt-get backend
    pacman.rs             -- pacman backend
    brew.rs               -- brew backend
  executor/
    shell.rs              -- runs shell commands via sh -c, captures output
  environment/
    activate.rs           -- subshell spawning, env var exporting
```

## How it works start to finish

When you run `envforge enter <name>`:

1. **main.rs** parses the CLI args and dispatches to `cmd_enter`
2. **config/loader.rs** reads `~/.envforge/<name>.yaml` and deserializes it into a `ProfileConfig` struct
3. **installer/mod.rs** checks which package manager is available (apt, pacman, brew) and installs the listed packages
4. **executor/shell.rs** runs each setup command via `sh -c`, with stdout/stderr capture and error handling
5. **environment/activate.rs** builds a combined shell string with exports, PS1 override, setup commands, and an `exec` of the user's shell

## PackageManager trait

```rust
pub trait PackageManager {
    fn install(&self, packages: &[String]) -> Vec<Result<(), String>>;
    fn name(&self) -> &'static str;
    fn is_available() -> bool where Self: Sized;
}
```

Each backend implements this trait. `is_available()` checks if the package manager binary exists on the system using `which`. `install()` runs the appropriate install commands and returns one result per package.

Adding a new backend (e.g. Docker, Nix):
- Create a new file in `src/installer/`
- Implement `PackageManager` for your struct
- Add it to the detection chain in `installer/mod.rs::detect_package_manager()`

No conditional compilation. OS detection happens at runtime via `which` checks.

## Environment activation

Two modes:

**Subshell (default)**: builds a single shell string like:
```
export KEY=VALUE; export PS1="(envforge:name)$PS1"; setup_cmd1; setup_cmd2; exec /bin/bash
```

Then runs `sh -c "<string>"`. The `exec` replaces sh with the user's shell.
This keeps all env changes scoped -- when you exit, nothing leaks.

**Export**: simply prints `export KEY=VALUE` lines to stdout. The user can
`eval "$(envforge enter --export <name>)"` to apply them. Setup commands are
skipped in this mode.

## Shell executor

`ShellExecutor` wraps `std::process::Command`. It:
- Runs commands via `sh -c` (shell syntax works)
- Captures stdout and stderr separately
- Logs stderr as a warning on success, as an error on failure
- Returns an error (with `anyhow`) on non-zero exit
- Supports `dry_run` mode (logs intent, skips execution)
- Supports `verbose` mode (logs stdout and the command string)

## Config storage

Profiles live in `~/.envforge/<name>.yaml`. The `profiles_dir()` function
creates this directory on first access. The YAML format is:

```yaml
name: profile-name
packages: [apt/pkg names]
env:
  VAR: value
setup:
  - shell command
  - another command
```

Only `name` is mandatory. All other fields default to empty.

## Design decisions

- **No conditional compilation for OS detection** -- using `which` at runtime
  means the binary is portable and adding a new backend does not require
  platform-specific code

- **Subshell isolation** -- env vars and side effects never leak to the
  parent shell

- **One result per package install** -- if one package fails, the others
  still install. Failed packages are logged but do not abort the whole
  activation

- **Module-per-backend** -- each package manager is in its own file.
  Adding a new one is a single-file change plus one line in `detect_package_manager()`

- **Executor vs environment separation** -- the executor handles raw command
  execution with logging. The environment module handles profile-level
  orchestration. They depend on each other through the `ShellExecutor` interface

## Testing

```bash
cargo test
```

Current tests cover env export building and value escaping in
`src/environment/activate.rs`. Tests for config parsing and executor behavior
should be added before making changes to those modules.

## Common changes

**Add a package manager**: create a file modeled on `apt.rs`, implement the trait, register it in `mod.rs::detect_package_manager()`.

**Change the config format**: modify `ProfileConfig` in `config/loader.rs`, update the example YAMLs, update the docs.

**Add a new command**: add a variant to `Commands` in `cli/commands.rs`, add a match arm in `main.rs`, implement the handler function.

**Modify activation behavior**: the activation logic is entirely in `environment/activate.rs`. The `spawn_subshell` function builds the shell init string.
