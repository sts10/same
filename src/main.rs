// use std::fs::File;
// use blake2::{Blake2b, Digest};
use std::{fs, io};

fn main() {
    // let hash1 = blake3::hash(b"test");
    // println!("Found hash: {}", hash1);

    let path = "foo.txt";
    let hasher = blake3::Hasher::new();
    let hash1 = hash(&path, hasher.clone());
    // let mut hasher = blake3::Hasher::new();
    let hash2 = hash(&path, hasher.clone());
    println!("{}\nvs\n{}", hash1, hash2);
}

fn hash(path: &str, mut hasher: blake3::Hasher) -> String {
    let mut file = fs::File::open(&path).unwrap();
    // let mut hasher = Blake2b::new();
    let _n = io::copy(&mut file, &mut hasher).unwrap();
    let hash = hasher.finalize();
    // println!("Path: {}", path);
    // println!("Bytes processed: {}", n);
    // println!("Hash value: {:?}", hash);
    return hash.to_string();
}
