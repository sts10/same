use std::{fs, io};
use walkdir::WalkDir;

fn main() {
    let test_path = "/media/sschlinkert/My Book/back-ups-rsync-snapshot-feb-2021/Documents/macbook-air-code/buzzfeed";
    println!("path is {}", test_path);

    println!("Making first hash");
    let hash_vec1 = hash_dir(test_path);
    println!("Making second hash");
    let hash_vec2 = hash_dir(test_path);

    println!("{:?}\nvs\n{:?}", hash_vec1, hash_vec2);
    if hash_vec1 == hash_vec2 {
        println!("Matched!");
    } else {
        println!("Does not match");
    }
}

fn hash_dir(dir_path: &str) -> blake3::Hash {
    let mut hasher = blake3::Hasher::new();

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        // println!("{:?}", entry.path());
        if entry.metadata().unwrap().is_file()
            && entry.path().starts_with("/home/sschlinkert/.") == false
        {
            let mut file = fs::File::open(&entry.path()).expect("Error opening a file for hashing");
            let _n = io::copy(&mut file, &mut hasher).expect("Error hashing a file");
        }
    }
    hasher.finalize()
}
