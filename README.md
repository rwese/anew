# anew - Rust re-implementation

This is a Rust re-implementation of [tomnomnom's anew](https://github.com/tomnomnom/anew) tool.

A simple command-line utility that reads from stdin and writes new (non-duplicate) lines to a destination file.

## Features

- **Deduplication**: Only writes lines that don't already exist in the destination file
- **Sorting**: Optional natural order sorting of lines
- **Trimming**: Optional whitespace trimming
- **Rewrite mode**: Clean up existing files by removing duplicates
- **Dry run**: Preview what would be written without making changes
- **Quiet mode**: Suppress output to stdout

## Installation

### From Source

```bash
cargo install --git https://github.com/wese/rust_anew.git
```

### Building

```bash
cargo build --release
```

The binary will be available at `target/release/anew`.

## Usage

```
simple tool to write non-duplicate lines to a file

Usage: anew [OPTIONS] <PATH>

Arguments:
  <PATH>  destination file

Options:
  -q, --quiet-mode    do not output new lines to stdout
  -s, --sort          sort lines (natsort)
  -t, --trim          trim whitespaces
  -r, --rewrite       rewrite existing destination file to remove duplicates
      --dry-run       do not write to file, only output what would be written
  -h, --help          Print help
  -V, --version       Print version
```

## Examples

### Basic Usage

```bash
echo -e "line1\nline2\nline1" | anew output.txt
# output.txt contains: line1, line2
```

### With Sorting

```bash
echo -e "banana\napple\ncherry" | anew --sort sorted.txt
# sorted.txt contains: apple, banana, cherry
```

### Quiet Mode

```bash
echo "new line" | anew -q existing.txt
# Writes to file but doesn't output to stdout
```

### Dry Run

```bash
echo "potential duplicate" | anew --dry-run file.txt
# Shows what would be written without making changes
```

### Rewrite Existing File

```bash
anew --rewrite deduplicated.txt
# Removes all duplicate lines from deduplicated.txt
```

## Development

### Building

```bash
cargo build              # Debug build
cargo build --release    # Optimized release build
```

### Testing

```bash
cargo test               # Run all tests
cargo test --verbose     # Run tests with detailed output
```

### Linting

```bash
cargo clippy             # Run clippy linter
cargo clippy -- -D warnings  # Treat warnings as errors
```

### Code Formatting

```bash
cargo fmt                # Format code
cargo fmt --check        # Check formatting without changes
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Credits

Based on the original [anew](https://github.com/tomnomnom/anew) tool by Tom Hudson (@tomnomnom).
