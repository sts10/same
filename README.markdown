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
    same [Inputted Directories]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <Inputted Directories>...    Directories to hash and compare
```


## Hashing function used

To hash files, this project uses [BLAKE3](https://github.com/BLAKE3-team/BLAKE3). You may also be interested in [the b3sum utility](https://github.com/BLAKE3-team/BLAKE3#the-b3sum-utility).
