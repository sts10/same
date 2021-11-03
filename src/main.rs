// use crossbeam_channel::{bounded, select};
// use crossbeam_utils::thread;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rayon::prelude::ParallelSliceMut;

use crossbeam_channel::unbounded;
use ignore::DirEntry;
use std::fs::File;
use std::hash::Hasher;
use std::io::BufRead;
use std::io::BufReader;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;
use std::{fs, io};
use structopt::StructOpt;
// use walkdir::WalkDir;
// use ignore::DirEntry;
use ignore::WalkBuilder;
// use ignore::WalkParallel;
// use ignore::WalkState;

/// same: Compare directories
#[derive(StructOpt, Debug)]
#[structopt(name = "same")]
struct Opt {
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
        hashes.push(hash_dir(directory, opt.thoroughness))
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

fn get_path_relative_to_dir<'a>(dir_path: &Path, full_path: &'a Path) -> &'a Path {
    full_path.strip_prefix(dir_path).unwrap()
}

fn hash_dir(dir_path: &Path, thoroughness: usize) -> u64 {
    if !dir_path.is_dir() {
        panic!("Not a directory! Quitting");
    }
    println!("Checking directory: {:?}", dir_path);

    // https://github.com/BurntSushi/ripgrep/blob/master/crates/ignore/examples/walk.rs
    let (tx, rx) = unbounded();
    // Should probably find a way to find user's number of threads
    // Maybe from this? https://github.com/dtolnay/sha1dir/blob/master/src/main.rs#L86-L87
    let walker = WalkBuilder::new(dir_path).threads(8).build_parallel();
    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            use ignore::WalkState::*;
            let entry = result.unwrap();
            // if entry.metadata().unwrap().is_file() {
            tx.send(entry);
            // }
            Continue
        })
    });
    drop(tx);
    // Collect all messages from the channel.
    // Note that the call to `collect` blocks until the sender is dropped.
    let mut entries: Vec<DirEntry> = rx.iter().collect();
    // Our choice here is whether to sort and iterate through ENTRIES or PATHS
    // Using entries gives us access to more data about each file, including metadat,
    // Using paths seems to be approximately 4% faster in a casual test.
    // let sorted_paths: Vec<&Path> = entries.iter().map(|entry| entry.path()).collect();
    entries.par_sort_by(|a, b| a.path().partial_cmp(b.path()).unwrap());

    let hashes: Vec<u64> = entries
        .par_iter()
        .filter_map(|entry| {
            let path = entry.path(); // if we iterate through sorted_paths we obviously wouldn't need this
            let mut hasher = ahash::AHasher::default();
            if thoroughness == 1 {
                // Compare file names by adding them to the hash
                let file_name = path.file_name();
                hasher.write(file_name.unwrap().as_bytes());
            }
            if thoroughness >= 2 {
                // Compare relative file paths, including file names, by adding them to the hash
                let rel_path = get_path_relative_to_dir(dir_path, path);
                hasher.write(rel_path.as_os_str().as_bytes());
            }
            // if thoroughness >= 3 {
            //     // Compare by file size
            //     let file_size = entry.metadata().expect("Error reading a file's size").len();
            //     hasher.write(&file_size.to_ne_bytes());
            // }
            if thoroughness == 4 {
                // Hash all file contents
                if entry.metadata().unwrap().is_file() {
                    let file = fs::File::open(path).expect("Error opening a file for hashing");
                    if let Some(mmap) = maybe_memmap_file(&file) {
                        // let _n = io::copy(&mut io::Cursor::new(mmap), &mut hasher)
                        //     .expect("Error hashing a file");
                        let cursor = &mut io::Cursor::new(mmap);
                        hasher.write(cursor.get_ref());
                    } else {
                        // Not sure how to do the following with rayon/in parallel
                        // See: https://github.com/BLAKE3-team/BLAKE3/blob/master/b3sum/src/main.rs#L224-L235
                        // let _n = io::copy(&mut file, &mut hasher).expect("Error hashing a file");
                        hash_file(path, &mut hasher).unwrap();
                    }
                }
            }
            Some(hasher.finish())
        })
        .collect();
    let mut all_hasher = ahash::AHasher::default();

    // Another idea: Rather than sorting entries or paths aboves,
    // sort the hashes here
    // hashes.par_sort_by(|a, b| a.partial_cmp(b).unwrap());
    for hash in hashes {
        all_hasher.write_u64(hash);
    }
    all_hasher.finish()
}

// https://github.com/BLAKE3-team/BLAKE3/blob/master/b3sum/src/main.rs#L276-L306
// Mmap a file, if it looks like a good idea. Return None in cases where we
// know mmap will fail, or if the file is short enough that mmapping isn't
// worth it. However, if we do try to mmap and it fails, return the error.
fn maybe_memmap_file(file: &File) -> Option<memmap::Mmap> {
    let metadata = file.metadata().unwrap();
    let file_size = metadata.len();
    if !metadata.is_file() {
        // Not a real file.
        None
    } else if file_size > isize::max_value() as u64 {
        // Too long to safely map.
        // https://github.com/danburkert/memmap-rs/issues/69
        None
    } else if file_size == 0 {
        // Mapping an empty file currently fails.
        // https://github.com/danburkert/memmap-rs/issues/72
        None
    } else if file_size < 16 * 1024 {
        // Mapping small files is not worth it.
        None
    } else {
        // Explicitly set the length of the memory map, so that filesystem
        // changes can't race to violate the invariants we just checked.
        let map = unsafe {
            memmap::MmapOptions::new()
                .len(file_size as usize)
                // .map(&file)
                .map(file)
                .unwrap()
        };
        Some(map)
    }
}

pub fn hash_file(path: &Path, hasher: &mut impl Hasher) -> Result<(), io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    loop {
        // Read some data.
        let buffer: &[u8] = reader.fill_buf()?;
        if buffer.is_empty() {
            // End of file.
            return Ok(());
        }
        // Hash it!
        hasher.write(buffer);
        // Tell the reader we consumed all the data it gave us.
        let size = buffer.len();
        reader.consume(size);
    }
}

fn is_all_same<T: PartialEq>(arr: &[T]) -> bool {
    arr.windows(2).all(|w| w[0] == w[1])
}

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn can_determine_same_directory_is_same() {
        let test_path = Path::new("./test-files/bar");
        assert_eq!(hash_dir(&test_path, 1), hash_dir(&test_path, 1));
        assert_eq!(hash_dir(&test_path, 2), hash_dir(&test_path, 2));
        assert_eq!(hash_dir(&test_path, 4), hash_dir(&test_path, 4));
    }

    #[test]
    fn can_determine_different_directories_are_different() {
        let path1 = Path::new("/home/sschlinkert/code/tic-tac-go");
        let path2 = Path::new("/home/sschlinkert/code/tidy");
        assert_ne!(hash_dir(&path1, 1), hash_dir(&path2, 1));
        assert_ne!(hash_dir(&path1, 2), hash_dir(&path2, 2));
        assert_ne!(hash_dir(&path1, 4), hash_dir(&path2, 4));
    }

    #[test]
    fn can_determine_copied_directory_is_same_from_paths_even_have_have_different_paths_and_path_lengths(
    ) {
        let test_path1 = Path::new("./test-files/bar");
        let test_path2 = Path::new("./test-files/back-ups/bar");
        assert_eq!(hash_dir(&test_path1, 2), hash_dir(&test_path2, 2));
        assert_eq!(hash_dir(&test_path1, 4), hash_dir(&test_path2, 4));
    }

    #[test]
    fn can_detect_files_differing_solely_based_on_file_content() {
        let path1 = Path::new("./test-files/bar");
        let path2 = Path::new("./test-files/corrupted_back_up/bar");
        assert_ne!(hash_dir(&path1, 4), hash_dir(&path2, 4));
    }

    #[test]
    fn can_determine_copied_directory_is_same_from_paths_even_have_have_different_dir_name() {
        let test_path1 = Path::new("./test-files/bar");
        let test_path2 = Path::new("./test-files/baz");
        assert_eq!(hash_dir(&test_path1, 2), hash_dir(&test_path2, 2));
        assert_eq!(hash_dir(&test_path1, 4), hash_dir(&test_path2, 4));
    }

    #[test]
    fn can_get_relative_path() {
        let dir_path = Path::new("home/username/Pictures");
        let this_sub_folder = Path::new("home/username/Pictures/holidays/maine");
        assert_eq!(
            get_path_relative_to_dir(&dir_path, &this_sub_folder),
            Path::new("holidays/maine"),
        );
    }

    #[test]
    fn can_get_relative_path_with_filename() {
        let dir_path = Path::new("home/username/Pictures");
        let absolute_path = Path::new("home/username/Pictures/holidays/maine/sunset.jpg");
        assert_eq!(
            get_path_relative_to_dir(&dir_path, &absolute_path),
            Path::new("holidays/maine/sunset.jpg"),
        );
    }
}
