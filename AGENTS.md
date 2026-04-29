# AGENTS.md

Rust re-implementation of [tomnomnom/anew](https://github.com/tomnomnom/anew). CLI tool to write non-duplicate lines to a file.

## Structure

```
.
├── src/
│   ├── main.rs    # Entry point, stdin/stdout handling
│   └── app.rs     # CLI args parsing and core logic
├── Cargo.toml     # Dependencies and package config
├── rustfmt.toml   # Code formatting rules
├── .github/workflows/
│   ├── ci.yml         # PR lint + test
│   └── release.yml    # Tag-based releases
└── devbox.json    # Dev environment config
```

## Commands

```bash
cargo build              # Debug build
cargo build --release    # Optimized release (in target/release/anew)
cargo test               # Run tests
cargo clippy             # Lint
cargo clippy -- -D warnings  # Strict lint (warnings = errors)
cargo fmt                # Format code
cargo fmt --check        # Check formatting
```

## Testing

- Unit tests live in `src/app.rs` alongside the code they test
- Run: `cargo test --all-features --verbose`

## Code Style

- **Formatting**: `rustfmt.toml` - 2-space indent, 100 char line width
- **Linting**: Clippy with strict mode (`-D warnings`)
- **Naming**: Rust conventions (snake_case functions, PascalCase types)

## Git Workflow

| Action | Command |
|--------|---------|
| Branch | `feat/<name>`, `fix/<name>` |
| Commit | Conventional Commits |
| Release | Tag `v<version>` (e.g., `v0.2.3`) |

- CI runs on all PRs and pushes to main
- Releases auto-generated from tags matching `v[0-9]+.*`

## Three-Tier Boundaries

### ALWAYS
- Run `cargo clippy -- -D warnings` and `cargo test` before committing
- Format code with `cargo fmt` before committing
- Run tests on all OSes (CI matrix: ubuntu, macos, windows)

### USUALLY / ASK FIRST
- Add dependencies (affects lock file and binary size)
- Modify release workflow or CI configuration
- Change CLI argument structure (affects user-facing API)

### NEVER
- Commit secrets, API keys, or credentials
- Disable clippy linting in CI
- Modify lock files directly (use `cargo update`)
