# Same 

Compare two or more directories to see if they're the same, with a set amount of thoroughness.

## Disclaimers

This is a toy for now, **so don't actually use this!**

## Installation

1. [Install Rust](https://www.rust-lang.org/tools/install) (version 1.56+) if you haven't already
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

When user sets `thoroughness` to...

- 1, the program compares given directories by checking file names (not paths).
- 2 by checking file _paths_ relative to inputted directories.
- 3 by checking the sizes of all files, as well as file paths relative to inputted directories.
- 4 by checking relative paths, file sizes, and hashes of every file in inputted directories, recursively. This takes a bit longer.

(Note for Rust developers: If you have an idea of how to use file modified times for a check, make an issue!)

## Tests and benches

This project includes some tests and test files, though some of them are hard-coded to directories on my personal computer. You can try to run them with `cargo test`.

I also wrote some benchmarks, one for hashing a file with different non-cryptographic hashes, and another that hashes directories. You can run all of them with `cargo bench`.

## Why I wrote this code

I was preparing to restore a Rsync back-up of a large directory from an external USB device. I was searching for way to confirm that all files had been restored from the back-up.

## Notes on status of the project

As I write this, the code is this repo _may_ even be useful.

### Alternative methods of checking if two directories are the same (not using this project)

Another way to check if directories are the same is to do [a dry-run of Rsync](https://unix.stackexchange.com/questions/57305/rsync-compare-directories), using `rsync`'s `-n` flag. Adding a `-c` flag tells Rsync to compare files using a checksum, similar to what `same` does if you set `thoroughness` to `4` (so the approximately equivalent commands would by `same -t 4 <directory1> <directory2>` vs `rsync -ncr <directory1> <directory2>`.

While this Rsync dry-run method relies on the much more refined codebase of Rsync, it is (I'm a bit proud to say) a bit slower than using `same`. You can test this yourself with a benchmarking tool like [hyperfine](https://github.com/sharkdp/hyperfine): `hyperfine -w 5 -m 20 'same -t 4 ~/code ~/code' 'rsync -nrc ~/code ~code'`. On my machine, `same -t 4` runs almost 7x faster than `rsync -nrc`.

### Another approach

Another approach to the problem that I like (but currently can't figure out how integrate into the current codebase) is to use parallel _recursion_ to hash directories (trees), [like this wonderful code](https://gist.github.com/rust-play/d1609c4758d17771bc57f71a81a0239f) written by [Sergey Bugaev](https://mastodon.technology/@bugaevc). Sergey's code runs a bit faster than this project, though lacks features like `--exclude` flag, which I'm not sure how I'd implement while also using parallel recursion. Maybe you can unify both approaches for a pull request!

## Hashing function used

To hash file names, paths, and files, this project uses [AHash](https://docs.rs/ahash/0.7.6/ahash/index.html). Note that AHash is **not** a cryptographically secure hash.

You can compare multiple "fast" hashes with this project by running `cargo bench --bench hash_file`. Feel free to add other appropriate hashing algorithms in a PR if you think that would be helpful.
