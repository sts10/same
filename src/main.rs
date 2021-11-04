use glob::glob;
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

    /// Exclude files from comparison
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
    let paths_to_exclude = match opt.exclude {
        Some(exclude_strings) => Some(get_paths_to_exclude(exclude_strings)),
        None => None,
    };

    let mut hashes = vec![];
    for directory in &opt.inputted_directories {
        hashes.push(hash_dir(
            directory,
            opt.thoroughness,
            &paths_to_exclude,
            opt.verbose,
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

fn get_paths_to_exclude(strings_to_exclude: Vec<String>) -> Vec<PathBuf> {
    let mut paths_to_exclude = vec![];
    for string in strings_to_exclude {
        println!("String is {:?}", string);
        for entry in glob(&string).expect("Failed to read exclude pattern") {
            match entry {
                Ok(path) => {
                    println!("{:?}", path.display());
                    paths_to_exclude.push(path);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }
        }
    }
    println!("I'd exclude the following:\n{:?}", paths_to_exclude);
    paths_to_exclude
}
