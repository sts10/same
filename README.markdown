# Same

Compare two or more directories to see if they're the same.

## Disclaimers

This is a toy for now, **so don't actually use this!**

## Installation

1. [Install Rust](https://www.rust-lang.org/tools/install) if you haven't already
2. Run: `cargo install --git https://github.com/sts10/same --branch main`

## Usage

```
USAGE:
    same --thoroughness <thoroughness> [Inputted Directories]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --thoroughness <thoroughness>    How thorough to be when comparing directories. 1 checks file names; 2 checks paths relative to inputted directory; 3 checks file size; 4 checks the actual files

ARGS:
    <Inputted Directories>...    Directories to compare for sameness
```

## Thoroughness 

Setting `thoroughness` to...

- 1 checks file names (not paths).
- 2 checks file _paths_ relative to inputted directories.
- 3 checks the sizes of all files, as well as file paths relative to inputted directories.
- 4 checks relative paths, file sizes, and hashes of every file in inputted directories.

If you have an idea of how to use file modified times for a check, make an issue!

## Hashing function used

To hash file names, paths, and files, this project uses [BLAKE3](https://github.com/BLAKE3-team/BLAKE3). 

You may also be interested in [the b3sum utility](https://github.com/BLAKE3-team/BLAKE3#the-b3sum-utility).
