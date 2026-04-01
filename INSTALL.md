# Installing tep

## Requirements

- Rust 1.85 or later ([install via rustup](https://rustup.rs))

No system SQLite required — `tep` bundles its own via the `rusqlite` crate.

---

## Install from crates.io (recommended)

```bash
cargo install tep
```

Verify:
```bash
tep --version
```

---

## Build from source

```bash
git clone https://github.com/MPopovych/tep.git
cd tep
cargo build --release
```

The binary will be at `target/release/tep`.

Add it to your PATH:
```bash
# option 1 — copy to a directory already in PATH
cp target/release/tep /usr/local/bin/

# option 2 — install via cargo
cargo install --path .
```

---

## Quick build commands

| Command             | What it does                              |
|---------------------|-------------------------------------------|
| `make build`        | Debug build                               |
| `make release`      | Optimized release build                   |
| `make test`         | Run all tests                             |
| `make check`        | Lint + format check + tests               |
| `make install`      | Install to `~/.cargo/bin`                 |
| `make install-dev`  | Install debug build (faster iteration)    |
| `make dist`         | Release build copied to `./bin/`          |
| `make clean`        | Remove build artifacts                    |

---

## Getting started

```bash
# Initialize a workspace in your project
cd your-project/
tep init

# Declare entities in source files by adding:
#   (#!#tep:entity_name)
# Then run:
tep entity auto ./src ./docs

# Add anchor tags in source files:
#   [#!#tep:anchor_name](entity1,entity2)
# Then run:
tep anchor auto ./src ./docs

# Query context for a concept
tep entity context entity_name

# See what's in the workspace
tep entity list
tep anchor list
tep health
```

See [README.md](./README.md) for the full command reference and syntax guide.

---

## Platform support

Tested on:
- macOS (Apple Silicon, x86_64)
- Linux (x86_64)

Windows support is not tested but may work. Contributions welcome.

---

## License

Public domain — [Unlicense](./LICENSE). No strings attached, no warranty.

## Uninstall

```bash
cargo uninstall tep
```
