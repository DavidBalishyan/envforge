# envforge

Create and manage reproducible development environments from YAML files.
A lightweight mix of direnv and provisioning tools, but local and simple.

## Quick start

```bash
cargo build --release
envforge init cpp-dev
# edit ~/.envforge/cpp-dev.yaml with your packages and settings
envforge enter cpp-dev
```

See [USAGE.md](USAGE.md) for a full walkthrough.

## YAML format

```yaml
name: cpp-dev

packages:
  - gcc
  - cmake
  - gdb

env:
  CC: gcc
  CXX: g++

setup:
  - mkdir -p build
  - touch src/main.cpp
```

Only `name` is required. Example configs in `examples/`.

## Commands

| Command | Description |
|---------|-------------|
| `init <name>` | Create a new profile in ~/.envforge/ |
| `create <path>` | Import a YAML file into ~/.envforge/ |
| `enter <profile>` | Activate a profile (spawn subshell) |
| `list` | Show all installed profiles |
| `remove <profile>` | Delete a profile |
| `doctor` | Check system dependencies |

Global flags: `-n` (dry-run), `-v` (verbose), `--export` on enter.

## Architecture

See [WALKTHROUGH.md](WALKTHROUGH.md) for developer documentation.

## Dependencies

clap, serde + serde_yaml, anyhow, log + env_logger, dirs

## License

[GPLv3](LICENSE)
