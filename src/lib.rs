use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rayon::prelude::ParallelSliceMut;

// use crossbeam_channel::{bounded, select};
// use crossbeam_utils::thread;
use crossbeam_channel::unbounded;
// use ignore::overrides::Glob;
use ignore::overrides::Override;
use ignore::DirEntry;
use std::fs::File;
use std::hash::Hasher;
use std::io::BufRead;
use std::io::BufReader;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::{fs, io};
// use walkdir::WalkDir;
// use ignore::DirEntry;
use ignore::WalkBuilder;

pub fn get_path_relative_to_dir<'a>(dir_path: &Path, full_path: &'a Path) -> &'a Path {
    full_path.strip_prefix(dir_path).unwrap()
}

fn build_override(dir_path: &Path, exclude_globs: &Option<Vec<String>>) -> Override {
    // https://docs.rs/ignore/0.4.18/ignore/overrides/struct.OverrideBuilder.html
    let mut my_override_builder = ignore::overrides::OverrideBuilder::new(dir_path);
    match exclude_globs {
        Some(exclude_globs) => {
            for this_glob in exclude_globs {
                my_override_builder
                    .add(this_glob)
                    .expect("Error adding an exclusion glob to override builder");
            }
        }
        // if no excludes found from end user, don't call `add` so
        // return override is empty (no exclusions)
        None => (),
    };
    my_override_builder.build().unwrap()
}

fn find_entries(dir_path: &Path, ignore_hidden: bool, my_override: Override) -> Vec<DirEntry> {
    // Following is based off of this example: https://github.com/BurntSushi/ripgrep/blob/master/crates/ignore/examples/walk.rs
    let (tx, rx) = unbounded(); // shound probably try to re-work this to use bounded
    let walker = WalkBuilder::new(dir_path)
        .hidden(ignore_hidden) // bool of whether we want to IGNORE hidden
        .overrides(my_override) // End user's exclude choices
        .threads(num_cpus::get()) // Get number of available CPUs of the current system
        .build_parallel();
    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            use ignore::WalkState::*;
            let entry = result.unwrap();
            tx.send(entry).unwrap();
            Continue
        })
    });
    drop(tx);
    // Collect all messages from the channel.
    // Note that the call to `collect` blocks until the sender is dropped.
    rx.iter().collect()
}

fn make_hashes_from_entries(
    entries: Vec<DirEntry>,
    dir_path: &Path,
    thoroughness: usize,
) -> Vec<u64> {
    entries
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
            if thoroughness >= 3 {
                // Compare by file size
                let file_size = entry.metadata().expect("Error reading a file's size").len();
                hasher.write(&file_size.to_ne_bytes());
            }
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
                        hash_file(&file, &mut hasher).unwrap();
                    }
                }
            }
            Some(hasher.finish())
        })
        .collect()
}

pub fn hash_dir(
    dir_path: &Path,
    thoroughness: usize,
    verbose: bool,
    ignore_hidden: bool,
    exclude_globs: &Option<Vec<String>>,
) -> u64 {
    if !dir_path.is_dir() {
        panic!("Not a directory! Quitting");
    }
    if verbose {
        println!("Checking directory: {:?}", dir_path);
    }

    let my_override = build_override(dir_path, exclude_globs);

    let mut entries: Vec<DirEntry> = find_entries(dir_path, ignore_hidden, my_override);
    // Our choice here is whether to sort and iterate through ENTRIES or PATHS
    // Using entries gives us access to more data about each file, including metadat,
    // Using paths seems to be approximately 4% faster in a casual test.
    // let sorted_paths: Vec<&Path> = entries.iter().map(|entry| entry.path()).collect();
    entries.par_sort_by(|a, b| a.path().partial_cmp(b.path()).unwrap());

    let hashes: Vec<u64> = make_hashes_from_entries(entries, dir_path, thoroughness);

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

pub fn hash_file(file: &File, hasher: &mut impl Hasher) -> Result<(), io::Error> {
    // let file = File::open(path)?;
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

pub fn is_all_same<T: PartialEq>(arr: &[T]) -> bool {
    arr.windows(2).all(|w| w[0] == w[1])
}
