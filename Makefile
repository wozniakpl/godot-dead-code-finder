# gdcf (godot-dead-code-finder) – Makefile wrappers for build, test, lint, coverage
#
# Coverage: install one of:
#   cargo install cargo-llvm-cov   (recommended)
#   cargo install cargo-tarpaulin

# Default: show available commands
help::
	@echo "gdcf – Godot dead code finder (Rust)"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build          Build release binary (target/release/godot-dead-code)"
	@echo "  test           Run tests (unit + integration)"
	@echo "  lint           Run clippy and fmt check"
	@echo "  format         Format code (cargo fmt)"
	@echo "  coverage       Run tests with coverage (cargo-llvm-cov)"
	@echo "  coverage-html  Coverage and open HTML report"
	@echo "  clean          Remove target/"
	@echo "  install        Install binary (cargo install --path .)"
	@echo "  all            build + test + lint"

all:: build test lint

build::
	cargo build --release

test::
	cargo test

lint::
	cargo clippy --all-targets -- -D warnings
	cargo fmt --all -- --check

# Apply formatter (fix code style). Use this to fix format; lint only checks.
format::
	cargo fmt --all

# Coverage: uses cargo-llvm-cov (cargo install cargo-llvm-cov). Exclude binary (main.rs) so we report library coverage.
coverage::
	cargo llvm-cov test --lcov --output-path lcov.info --ignore-filename-regex 'main\.rs'
	cargo llvm-cov report --ignore-filename-regex 'main\.rs'

# Generate HTML coverage report
coverage-html::
	cargo llvm-cov test --html --ignore-filename-regex 'main\.rs'
	@echo "Report: target/llvm-cov/html/index.html"

clean::
	cargo clean

install::
	cargo install --path .
