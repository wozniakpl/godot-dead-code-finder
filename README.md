# gdcf (godot-dead-code-finder)

CLI tool that scans a Godot GDScript codebase and reports **functions that are never called** anywhere.

## Install

**From this repo (no clone needed):**

```bash
cargo install --git https://github.com/wozniakpl/godot-dead-code-finder
```

This installs two binaries: `godot-dead-code` and `gdcf` (same tool).

**From a local clone:**

```bash
git clone https://github.com/wozniakpl/godot-dead-code-finder
cd godot-dead-code-finder
cargo install --path .
# or build only: cargo build --release  → binary at target/release/godot-dead-code
```

Requires [Rust](https://rustup.rs/). Installed binaries go to `~/.cargo/bin` (ensure it’s on your `PATH`).

## Usage

```bash
# Show help (no arguments)
godot-dead-code

# Scan current directory
godot-dead-code .

# Scan a specific project
godot-dead-code /path/to/your/godot/project

# Quiet mode: exit 1 if any unused or test-only functions, no output
godot-dead-code -q /path/to/project

# Custom test directory (relative to project root); can be repeated
godot-dead-code --test-dir tests --test-dir spec /path/to/project
```

Output:
- **Unused (never called):** one line per function: `path/to/script.gd:LINE: function_name`
- **Only called from test code:** main-app functions that are never called from main app, only from test files (always reported)

Test code is detected by default when the path is under a `tests/` or `test/` directory, or the script name is `*_test.gd` or `test_*.gd`. Override with `--test-dir`.

Engine callbacks such as `_ready`, `_process`, `_input`, etc. are always treated as used (they are invoked by the engine). **GUT** (Godot Unit Test) lifecycle hooks (`before_each`, `after_each`, `before_all`, `after_all`, `before_test`, `after_test`) and all `test_*` functions are also treated as used (GUT invokes them).

**Scene files (.tscn):** The tool also scans `.tscn` files for signal connections (`method="..."`). Functions used only as signal handlers (e.g. `_on_quit_dialog_confirmed` connected to a button) are not reported as unused.

### Ignoring functions (tagging)

You can mark functions so they are **not** reported as unused. Use this for callbacks you plan to wire up, or code you are keeping intentionally. Put one of these comments on the **same line** as the function header (after `:`) or on the **next line**:

- `# gdcf-ignore`
- `# dead-code-ignore`
- `# TODO: dead-code`

Example (same line):

```gdscript
func will_use_later(): # gdcf-ignore
    pass
```

Example (next line):

```gdscript
func will_use_later():
    # TODO: dead-code
    pass
```

Tagged functions are excluded from both "Unused (never called)" and "Only called from test code". Use this to shrink the report to functions you still need to either use or remove.

## Build & test

```bash
cargo build
cargo test
cargo clippy
cargo fmt --check
```

### Makefile

Convenience wrappers (run `make help` for a list):

| Command        | Description                          |
|----------------|--------------------------------------|
| `make build`   | Build release binary                 |
| `make test`    | Run tests                            |
| `make lint`    | Clippy + fmt check                   |
| `make format`  | Format code (cargo fmt)              |
| `make coverage`| Test coverage (needs `cargo install cargo-llvm-cov`) |
| `make coverage-html` | Coverage as HTML report         |
| `make clean`   | Remove target/                      |
| `make install` | Install binary (cargo install)       |
| `make all`     | build + test + lint                  |

## Development

Commits and PR titles follow [Conventional Commits](https://www.conventionalcommits.org/) (e.g. `feat(cli): add flag`, `fix: crash on empty dir`). Releases are cut automatically on push to `main` via [semantic-release](https://github.com/semantic-release/semantic-release).

- **Pre-commit:** Install the commit-msg hook to check messages locally:
  ```bash
  pip install pre-commit   # or: brew install pre-commit
  pre-commit install --hook-type commit-msg
  ```
- **PRs:** The CI checks that the PR title matches a conventional type (`feat`, `fix`, `docs`, etc.).

## License

MIT
