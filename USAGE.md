# Usage walkthrough

This guide walks through real envforge usage from start to finish.

## Setup

```bash
# Compile the binary
cargo build --release

# Add to your PATH (or use ./target/release/envforge directly)
export PATH="$PWD/target/release:$PATH"
```

## Creating a profile from scratch

```bash
envforge init rust-dev
```

This creates `~/.envforge/rust-dev.yaml` with a template. Open it and fill in
the details:

```yaml
name: rust-dev

packages:
  - rustc
  - cargo
  - gdb

env:
  RUST_BACKTRACE: "1"
  CARGO_INCREMENTAL: "1"

setup:
  - cargo init --name my-project . 2>/dev/null || true
  - echo "Rust environment ready"
```

## Importing a YAML file

If someone shares a config file, use `create` to install it:

```bash
# Download or receive a yaml file
envforge create ./cpp-dev.yaml
```

This copies the file into `~/.envforge/` using the `name` field from the YAML.

## Activating an environment

```bash
envforge enter rust-dev
```

You will see:

```
[INFO] loading profile 'rust-dev'
[INFO] detected package manager: apt
[INFO] installing 3 packages...
[INFO]   - rustc
[INFO]   - cargo
[INFO]   - gdb
[INFO] running setup commands...
[INFO] activating environment...
```

Your shell prompt changes to `(envforge:rust-dev) ` to show you are inside the
environment. When you are done, type `exit` or press Ctrl-D to return to your
normal shell.

## Export mode (no subshell)

If you prefer to set variables in your current shell without spawning a
subshell, use the `--export` flag:

```bash
eval "$(envforge enter --export rust-dev)"
```

This prints `export` statements. The `eval` applies them to your current shell.
Note: setup commands are NOT run in export mode.

## Listing profiles

```bash
envforge list
```

Shows each profile with its package count, env var count, and config path:

```
available profiles:
  cpp-dev    (packages: 4, env vars: 4, config: /home/user/.envforge/cpp-dev.yaml)
  rust-dev   (packages: 3, env vars: 2, config: /home/user/.envforge/rust-dev.yaml)
```

## Removing a profile

```bash
envforge remove cpp-dev
```

Deletes the corresponding `.yaml` file from `~/.envforge/`.

## Checking system health

```bash
envforge doctor
```

Prints a diagnostic report:

```
envforge doctor - system diagnostics

config directory: /home/user/.envforge
  profile files found: 2

package managers:
  apt-get available
  pacman not found
  brew not found

active package manager detected

default shell: /usr/bin/zsh
```

## Dry-run mode

Pass `-n` or `--dry-run` to any command to see what would happen without
actually doing anything:

```bash
envforge -n enter rust-dev
```

This is useful for verifying config files before committing to a full
activation.

## Verbose mode

Pass `-v` to see detailed logs, including stdout/stderr from commands:

```bash
envforge -v enter rust-dev
```

## Using .envforge.yaml locally

Place a `.envforge.yaml` file in your project root. When you run
`envforge enter .envforge.yaml`, it will detect and load the local file if no
matching profile exists in `~/.envforge/`.

This works well for project-specific environments that you want to commit to
version control.

## What happens step-by-step on enter

When you run `envforge enter <profile>`:

1. **Load config** - reads and parses the YAML file
2. **Install packages** - detects your system package manager (apt/pacman/brew)
   and installs each package listed in `packages`
3. **Run setup** - executes each command in `setup` sequentially via `sh -c`
4. **Export env vars** - sets each variable from `env` in the shell
5. **Modify prompt** - prepends `(envforge:<name>) ` to PS1
6. **Exec subshell** - replaces the current process with your login shell

All env vars and setup changes are scoped to the subshell. Nothing leaks when
you exit.
