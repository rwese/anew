# anew - Rust re-implementation

This is a re-implementation of [tomnomnom's anew](https://github.com/tomnomnom/anew) tool.

It reads from stdin and writes new lines to the destination file.

## usage

```
simple tool to write non - duplicate lines to a file

Usage: anew [OPTIONS] <PATH>

Arguments:
    <PATH>  destination file

Options:
    -q, --quiet-mode  do not output new lines to stdout
  -s, --sort        sort lines (natsort)
  -t, --trim        trim whitespaces
  -r, --rewrite     rewrite existing destination file to remove duplicates
      --dry-run     do not write to file, only output what would be written
  -h, --help        Print help
  -V, --version     Print version
```
