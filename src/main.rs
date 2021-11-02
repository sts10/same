use rayon::iter::IntoParallelRefIterator;

use rayon::iter::ParallelIterator;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;
use std::{fs, io};
use structopt::StructOpt;
use walkdir::WalkDir;

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
            "blake3sum for {:?} is\n{}",
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
    println!("New directory: {:?}", dir_path);
    // We have to sort entries because WalkDir doesn't walk the same way
    // each run

    let mut sorted_entries: Vec<walkdir::DirEntry> = vec![];
    for entry in WalkDir::new(dir_path)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.metadata().unwrap().is_file() {
            sorted_entries.push(entry)
        }
    }
    // sorted_entries.par_sort_by(|a, b| a.path().partial_cmp(b.path()).unwrap());

    let sorted_paths: Vec<&Path> = sorted_entries.iter().map(|entry| entry.path()).collect();
    // for entry in sort_dir_par(dir_path) {
    let hashes: Vec<u64> = sorted_paths
        .par_iter() // maybe the other one
        .filter_map(|path| {
            let mut hasher = ahash::AHasher::default();
            if thoroughness == 1 {
                // Compare file names by adding them to the hash
                let file_name = path.file_name().unwrap();
                hasher.write(file_name.as_bytes());
            }
            if thoroughness >= 2 {
                // Compare realtive file paths, including file names, by adding them to the hash
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
                let mut file = fs::File::open(&path).expect("Error opening a file for hashing");
                if let Some(mmap) = maybe_memmap_file(&file) {
                    // let _n = io::copy(&mut io::Cursor::new(mmap), &mut hasher)
                    //     .expect("Error hashing a file");
                    let cursor = &mut io::Cursor::new(mmap);
                    hasher.write(cursor.get_ref());
                } else {
                    // Not sure how to do the following with rayon/in parallel
                    // See: https://github.com/BLAKE3-team/BLAKE3/blob/master/b3sum/src/main.rs#L224-L235
                    // let _n = io::copy(&mut file, &mut hasher).expect("Error hashing a file");
                    // let bytes = file.bytes().map(|byte| byte.unwrap()).collect();
                    // let mut bytes = vec![];
                    // for byte in file.bytes() {
                    //     bytes.push(byte.unwrap());
                    // }
                    // hasher.write(&bytes);
                    let mut buffer = Vec::new();
                    hasher.write_u64(file.read(&mut buffer).unwrap().try_into().unwrap());
                }
            }
            Some(hasher.finish())
        })
        .collect();

    let mut all_hasher = ahash::AHasher::default();
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

fn is_all_same<T: PartialEq>(arr: &[T]) -> bool {
    arr.windows(2).all(|w| w[0] == w[1])
}

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn can_determine_copied_directory_is_same_from_paths() {
        let test_path = Path::new("./test-files/bar");
        assert_eq!(hash_dir(&test_path, 4), hash_dir(&test_path, 4),);
    }

    #[test]
    fn can_determine_different_directories_are_different() {
        let path1 = Path::new("/home/sschlinkert/code/tic-tac-go");
        let path2 = Path::new("/home/sschlinkert/code/tidy");
        assert_ne!(hash_dir(&path1, 4), hash_dir(&path2, 4));
    }

    #[test]
    fn can_detect_files_differing_solely_based_on_file_size() {
        let path1 = Path::new("./test-files/bar");
        let path2 = Path::new("./test-files/corrupted_back_up/bar");
        assert_ne!(hash_dir(&path1, 3), hash_dir(&path2, 3));
    }

    #[test]
    fn can_determine_copied_directory_is_same_from_paths_even_have_have_different_dir_name() {
        let test_path1 = Path::new("./test-files/bar");
        let test_path2 = Path::new("./test-files/baz");
        assert_eq!(hash_dir(&test_path1, 2), hash_dir(&test_path2, 2),);
    }

    #[test]
    fn can_determine_copied_directory_is_same_from_paths_even_have_have_different_paths_and_path_lengths(
    ) {
        let test_path1 = Path::new("./test-files/bar");
        let test_path2 = Path::new("./test-files/back-ups/bar");
        assert_eq!(hash_dir(&test_path1, 4), hash_dir(&test_path2, 4),);
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
