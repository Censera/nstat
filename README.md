# nstat

[![License](https://img.shields.io/github/license/Censera/nstat.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)

A `stat` replacement with readable output. Give it a path and it prints the file's type, size, permissions, ownership, and timestamps in aligned, color-coded text. Pass `--explain` to get a plain-English description of each field.

Unix only. The metadata fields nstat reads (inode, hard links, device numbers, uid/gid) have no equivalent on Windows.

## Install

```r
cargo install --path .
```

**From source** (requires rustc 1.85+):

```r
cargo build --release
```

## Usage

```r
nstat [OPTIONS] [PATH]...

Options:
  -h, --help       Print this help message
      --explain    Print a plain-English description of each field
      --no-color   Disable ANSI terminal colors
  -l, --no-follow  Do not follow symlinks

Args:
  [PATH]...        Files or directories to inspect [default: .]
```

Respects the `NO_COLOR` environment variable.

## What nstat shows

For a regular file: size (with disk blocks and IO block size), hard link count, device, inode, permissions in `rwxrwxrwx` and octal, owner, group, and modified/accessed/created timestamps as both an absolute datetime and a human-readable relative age.

Directories, symlinks, named pipes, sockets, and device files get type, permissions, ownership, and timestamps. Symlinks also show the resolved target path.

## Examples

```r
nstat file.txt
nstat --explain file.txt
nstat --no-follow symlink
nstat /dev/sda
nstat file1.txt file2.txt
```

## License

[MIT](LICENSE)
