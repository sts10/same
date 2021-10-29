// use std::fs::File;
// use blake2::{Blake2b, Digest};

use std::path::Path;
use std::{fs, io};
use walkdir::WalkDir;

fn main() {
    // let hash1 = blake3::hash(b"test");
    // println!("Found hash: {}", hash1);

    // let path = "foo.txt";
    // let hasher = blake3::Hasher::new();
    // let hash1 = hash(&path, hasher.clone());
    // // let mut hasher = blake3::Hasher::new();
    // let hash2 = hash(&path, hasher.clone());
    // println!("{}\nvs\n{}", hash1, hash2);

    // for entry in WalkDir::new("bar").into_iter().filter_map(|e| e.ok()) {
    //     println!("{:?}", entry.metadata());
    //     println!("{}", entry.path().display());
    // }
    // for entry in WalkDir::new("baz").into_iter().filter_map(|e| e.ok()) {
    //     println!("{:?}", entry.metadata().unwrap().is_file());
    //     println!("{}", entry.path().display());
    // }
    let hash_vec1 = hash_dir("/home/sschlinkert/code");
    let hash_vec2 = hash_dir("/home/sschlinkert/code");
    // let hash_vec2 = hash_dir("baz");

    println!("{:?}\nvs\n{:?}", hash_vec1, hash_vec2);
    if hash_vec1 == hash_vec2 {
        println!("Matched!");
    } else {
        println!("Do not match");
    }
}

fn hash(path: &Path, mut hasher: blake3::Hasher) -> String {
    let mut file = fs::File::open(&path).expect("Error opening a file for hashing");
    // let mut hasher = Blake2b::new();
    let _n = io::copy(&mut file, &mut hasher).expect("Error hashing a file");
    let hash = hasher.finalize();
    // println!("Path: {}", path);
    // println!("Bytes processed: {}", n);
    // println!("Hash value: {:?}", hash);
    return hash.to_string();
}

fn hash_dir(dir_path: &str) -> Vec<String> {
    let mut hash_dir = vec![];

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.metadata().unwrap().is_file() == true {
            println!("{}", entry.path().display());
            let hasher = blake3::Hasher::new();
            let this_file_hash = hash(entry.path(), hasher);
            hash_dir.push(this_file_hash.to_string());
        }
    }
    hash_dir
}
