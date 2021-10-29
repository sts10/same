use std::{fs, io};
use walkdir::WalkDir;

fn main() {
    let test_path = "/home/sschlinkert/code";
    println!("Making first hash");
    let hash1 = hash_dir(test_path);
    println!("Making second hash");
    let hash2 = hash_dir("/home/sschlinkert/code");

    if hash1 == hash2 {
        println!("Matched!");
    } else {
        println!("Does not match");
    }
}

fn hash_dir(dir_path: &str) -> blake3::Hash {
    let mut hasher = blake3::Hasher::new();

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.metadata().unwrap().is_file() {
            let mut file = fs::File::open(&entry.path()).expect("Error opening a file for hashing");
            let _n = io::copy(&mut file, &mut hasher).expect("Error hashing a file");
        }
    }
    hasher.finalize()
}
