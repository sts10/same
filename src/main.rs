use std::fs::File;
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
    /// How thorough to be when comparing directories
    #[structopt(short = "t", long = "thoroughness")]
    thoroughness: usize,

    /// Directories to hash and compare
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
    let length_of_dir_path = dir_path.components().count();
    let mut rel_path_components = full_path.components();
    for _n in 0..length_of_dir_path {
        rel_path_components.next();
    }
    rel_path_components.as_path()
}

fn hash_dir(dir_path: &Path, thoroughness: usize) -> blake3::Hash {
    if !dir_path.is_dir() {
        panic!("Not a directory! Quitting");
    }
    println!("New directory: {:?}", dir_path);
    let mut hasher = blake3::Hasher::new();

    // We have to sort entries because WalkDir doesn't walk the same way
    // each run
    let mut sorted_entries = vec![];
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        sorted_entries.push(entry)
    }
    sorted_entries.sort_by(|a, b| a.path().partial_cmp(b.path()).unwrap());
    for entry in sorted_entries {
        if thoroughness == 1 {
            let file_name = entry.path().file_name().unwrap();
            hasher.update(file_name.as_bytes());
        }
        if thoroughness >= 2 {
            let rel_path = get_path_relative_to_dir(dir_path, entry.path());
            hasher.update(rel_path.as_os_str().as_bytes());
        }
        if thoroughness == 4 {
            if !entry.metadata().unwrap().is_file() {
                continue;
            }
            let mut file = fs::File::open(&entry.path()).expect("Error opening a file for hashing");
            if let Some(mmap) = maybe_memmap_file(&file) {
                let _n = io::copy(&mut io::Cursor::new(mmap), &mut hasher)
                    .expect("Error hashing a file");
            } else {
                let _n = io::copy(&mut file, &mut hasher).expect("Error hashing a file");
            }
        }
    }
    hasher.finalize()
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
    fn can_determine_ccopied_directory_is_same() {
        assert_eq!(
            hash_dir_content(&PathBuf::from("./test-files/bar")),
            hash_dir_content(&PathBuf::from("./test-files/baz"))
        );
    }

    #[test]
    fn can_determine_copied_directory_is_same_from_paths() {
        assert_eq!(
            hash_dir_paths(&PathBuf::from("./test-files/bar")),
            hash_dir_paths(&PathBuf::from("./test-files/bar"))
        );
    }

    #[test]
    fn can_determine_same_directory_is_same() {
        let path = PathBuf::from("/home/sschlinkert/code/tic-tac-go");
        assert_eq!(hash_dir_content(&path), hash_dir_content(&path));
    }

    #[test]
    fn can_determine_different_directories_are_different() {
        let path1 = PathBuf::from("/home/sschlinkert/code/tic-tac-go");
        let path2 = PathBuf::from("/home/sschlinkert/code/tidy");
        assert_ne!(hash_dir_content(&path1), hash_dir_content(&path2));
        assert_ne!(
            hash_dir_content(&PathBuf::from("./test-files/bar")),
            hash_dir_content(&PathBuf::from("./test-files/lasagna"))
        );
    }
}

// fn hash_dir_paths(dir_path: &Path) -> blake3::Hash {
//     if !dir_path.is_dir() {
//         panic!("Not a directory! Quitting");
//     }
//     let mut hasher = blake3::Hasher::new();

//     for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
//         if entry.metadata().unwrap().is_file()
//             && !entry
//                 .path()
//                 .starts_with("/home/sschlinkert/.steam/steam.pipe")
//         {
//             // println!("Metadata: {:?}", entry.metadata().unwrap());
//             // println!("Path: {:?}", entry.path());
//             // for component in entry.path() {
//             //     println!("{:?}", component)
//             // }
//             println!("Path: {:?}", entry.path().components().into_iter()[0..1]);
//             hasher.update(entry.path().to_str().unwrap().as_bytes());
//         }
//     }
//     hasher.finalize()
// }
