use std::fs::File;
use std::path::Path;
use std::{fs, io};
use walkdir::WalkDir;

fn main() {
    let test_path = "/home/sschlinkert";
    println!("Making first hash");
    let hash1 = hash_dir(test_path);
    println!("Making second hash");
    let hash2 = hash_dir("/home/sschlinkert/Documents");

    if hash1 == hash2 {
        println!("Matched!");
    } else {
        println!("Does not match");
    }
}

fn hash_dir(dir_path: &str) -> blake3::Hash {
    if !Path::new(dir_path).is_dir() {
        panic!("Not a directory! Quitting");
    }
    let mut hasher = blake3::Hasher::new();

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.metadata().unwrap().is_file()
            && !entry
                .path()
                .starts_with("/home/sschlinkert/.steam/steam.pipe")
        {
            let mut file = fs::File::open(&entry.path()).expect("Error opening a file for hashing");
            if let Some(mmap) = maybe_memmap_file(&file) {
                // println!("mmapping {}, baby!", entry.path().display());
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

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn can_determine_ccopiedd_directory_is_same() {
        assert_eq!(hash_dir("./test-files/bar"), hash_dir("./test-files/baz"));
    }

    #[test]
    fn can_determine_same_directory_is_same() {
        let path = "/home/sschlinkert/code/tic-tac-go";
        assert_eq!(hash_dir(path), hash_dir(path));
    }

    #[test]
    fn can_determine_different_directories_are_different() {
        let path1 = "/home/sschlinkert/code/tic-tac-go";
        let path2 = "/home/sschlinkert/code/tidy";
        assert_ne!(hash_dir(path1), hash_dir(path2));
        assert_ne!(
            hash_dir("./test-files/bar"),
            hash_dir("./test-files/lasagna")
        );
    }
}
