extern crate chrono;
use chrono::offset::Utc;
use chrono::DateTime;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
// use std::time::SystemTime;
use std::{fs, io};
use structopt::StructOpt;
use walkdir::WalkDir;

/// same: Compare directories
#[derive(StructOpt, Debug)]
#[structopt(name = "same")]
struct Opt {
    /// How thorough to be when comparing directories. 1 checks file names; 2 checks paths relative
    /// to inputted directory; 4 checks the actual files
    #[structopt(short = "t", long = "thoroughness")]
    thoroughness: usize,

    /// Directories to compare for sameness
    #[structopt(name = "Inputted Directories", parse(from_os_str))]
    inputted_directories: Vec<PathBuf>,
}

fn main() {
    let current_sys_time = SystemTime::now();
    let opt = Opt::from_args();
    let mut hashes = vec![];
    for directory in &opt.inputted_directories {
        hashes.push(hash_dir(directory, opt.thoroughness, current_sys_time))
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

fn hash_dir(dir_path: &Path, thoroughness: usize, current_sys_time: SystemTime) -> blake3::Hash {
    if !dir_path.is_dir() {
        panic!("Not a directory! Quitting");
    }
    println!("New directory: {:?}", dir_path);
    let mut hasher = blake3::Hasher::new();

    // for entry in sort_dir_par(dir_path) {
    for entry in WalkDir::new(dir_path)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if thoroughness == 1 {
            let file_name = entry.path().file_name().unwrap();
            hasher.update_rayon(file_name.as_bytes());
        }
        if thoroughness >= 2 {
            let rel_path = get_path_relative_to_dir(dir_path, entry.path());
            hasher.update_rayon(rel_path.as_os_str().as_bytes());
        }
        if thoroughness >= 3 {
            let time_modified = entry
                .metadata()
                .expect("Unable to access file metadata")
                .modified()
                .unwrap();
            let rel_path = get_path_relative_to_dir(dir_path, entry.path());
            // println!(
            //     "time modified is {:?}; and rel path is {:?}",
            //     time_modified, rel_path
            // );
            // let basic_hash_of_time_modified = hash_system_time(time_modified);
            let last_modified = get_system_time_difference(time_modified, current_sys_time);
            dbg!(last_modified);

            hasher.update_rayon(&last_modified.as_nanos().to_ne_bytes());
            // let modified_time_as_date_time: DateTime<Utc> = time_modified.into();
            // hasher.update_rayon(modified_time_as_date_time.to_rfc3339().as_bytes());
            // hasher.update_rayon(basic_hash_of_time_modified.into());
            // hasher.update_rayon(basic_hash_of_time_modified.as_bytes());
        }
        if thoroughness == 4 {
            if !entry.metadata().unwrap().is_file() {
                continue;
            }
            let mut file = fs::File::open(&entry.path()).expect("Error opening a file for hashing");
            if let Some(mmap) = maybe_memmap_file(&file) {
                // let _n = io::copy(&mut io::Cursor::new(mmap), &mut hasher)
                //     .expect("Error hashing a file");
                let cursor = &mut io::Cursor::new(mmap);
                hasher.update_rayon(cursor.get_ref());
            } else {
                // Not sure how to do the following with rayon/in parallel
                // See: https://github.com/BLAKE3-team/BLAKE3/blob/master/b3sum/src/main.rs#L224-L235
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

fn _sort_dir_par(dir_path: &Path) -> Vec<walkdir::DirEntry> {
    // We have to sort entries because WalkDir doesn't walk the same way
    // each run
    let mut sorted_entries = vec![];
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        sorted_entries.push(entry)
    }
    use rayon::slice::ParallelSliceMut;
    sorted_entries.par_sort_unstable_by(|a, b| a.path().partial_cmp(b.path()).unwrap());
    sorted_entries
}

fn get_system_time_difference(
    sys_time: SystemTime,
    current_sys_time: SystemTime,
) -> std::time::Duration {
    let difference = current_sys_time
        .duration_since(sys_time)
        .expect("SystemTime::duration_since failed");
    println!("Difference is {:?}", difference);
    difference
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
    fn can_determine_copied_directory_is_same_from_paths_even_have_have_different_dir_name() {
        let test_path1 = Path::new("./test-files/bar");
        let test_path2 = Path::new("./test-files/baz");
        assert_eq!(hash_dir(&test_path1, 2), hash_dir(&test_path2, 2),);
    }

    #[test]
    fn can_determine_copied_directory_is_same_from_paths_even_have_have_different_paths_and_path_lengths(
    ) {
        let test_path1 = Path::new("./test-files/bar");
        // let test_path2 = Path::new("./test-files/back-ups/bar");
        let test_path2 = Path::new("./test-files/back_ups2/bar");
        assert_eq!(hash_dir(&test_path1, 4), hash_dir(&test_path2, 4),);
    }
}
