# Same

Compare two or more directories to see if they're the same.

## Disclaimers

This is a toy for now, **so don't actually use this!**

## Installation

1. [Install Rust](https://www.rust-lang.org/tools/install) if you haven't already
2. Run: `cargo install --git https://github.com/sts10/tidy --branch main`

## Usage

```
USAGE:
    same --thoroughness <thoroughness> [Inputted Directories]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --thoroughness <thoroughness>    How thorough to be when comparing directories

ARGS:
    <Inputted Directories>...    Directories to hash and compare
```

## Thoroughness 

Setting `thoroughness` to...

- 1 checks file names (not paths)
- 2 checks relative paths
- 3 is not implemented yet
- 4 checks relative paths and hashes every file in both directories and compares all that.

## Hashing function used

To hash file names, paths, and files, this project uses [BLAKE3](https://github.com/BLAKE3-team/BLAKE3). 

You may also be interested in [the b3sum utility](https://github.com/BLAKE3-team/BLAKE3#the-b3sum-utility).
