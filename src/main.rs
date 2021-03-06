// use ignore::overrides::Glob;
use same::hash_dir;
use same::is_all_same;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

/// same: Compare directories
#[derive(StructOpt, Debug)]
#[structopt(name = "same")]
struct Opt {
    /// Give verbose output
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Ignore hidden files
    #[structopt(long = "ignore-hidden")]
    ignore_hidden: bool,

    /// Exclude files from comparison, relative to the given directories
    #[structopt(long = "exclude")]
    exclude: Option<Vec<String>>,

    /// How thorough to be when comparing directories. 1 checks file names; 2 checks paths relative
    /// to inputted directory; 3 checks file size; 4 checks the actual files
    #[structopt(short = "t", long = "thoroughness")]
    thoroughness: usize,

    /// Directories to compare for sameness
    #[structopt(name = "Inputted Directories", parse(from_os_str))]
    inputted_directories: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    let mut hashes = vec![];
    for directory in &opt.inputted_directories {
        hashes.push(hash_dir(
            directory,
            opt.thoroughness,
            opt.verbose,
            opt.ignore_hidden,
            &opt.exclude,
        ))
    }

    if hashes.is_empty() {
        panic!("Didn't find anything hash or compare!")
    } else if hashes.len() == 1 {
        println!(
            "hash for {:?} is\n{:?}",
            fs::canonicalize(&opt.inputted_directories[0]).unwrap(),
            hashes[0]
        );
    } else {
        if is_all_same(&hashes) {
            println!("Directories are all the same!");
        } else {
            println!("Directories are NOT the same.");
        }
    }
}
