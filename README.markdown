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
    same [FLAGS] [OPTIONS] --thoroughness <thoroughness> [--] [Inputted Directories]...

FLAGS:
    -h, --help             Prints help information
        --ignore-hidden    Ignore hidden files
    -V, --version          Prints version information
    -v, --verbose          Give verbose output

OPTIONS:
        --exclude <exclude>...           Exclude files from comparison, relative to the given directories
    -t, --thoroughness <thoroughness>    How thorough to be when comparing directories. 1 checks file names; 2 checks
                                         paths relative to inputted directory; 3 checks file size; 4 checks the actual
                                         files

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

To hash file names, paths, and files, this project uses [AHash](https://docs.rs/ahash/0.7.6/ahash/index.html). Note that AHash is **not** a cryptographically secure hash.

You can compare multiple "fast" hashes with this project by running `cargo bench --bench hash_file`.
