
```rust
    entry
        .path()
        .starts_with("/home/sschlinkert/.steam/steam.pipe")
        == false
```
```rust
fn _hash(path: &Path, mut hasher: blake3::Hasher) -> String {
    let mut file = fs::File::open(&path).expect("Error opening a file for hashing");
    // let mut hasher = Blake2b::new();
    let _n = io::copy(&mut file, &mut hasher).expect("Error hashing a file");
    let hash = hasher.finalize();
    // println!("Path: {}", path);
    // println!("Bytes processed: {}", n);
    // println!("Hash value: {:?}", hash);
    return hash.to_string();
}

fn _add_to_hash(path: &Path, mut hasher: blake3::Hasher) {
    let mut file = fs::File::open(&path).expect("Error opening a file for hashing");
    // let mut hasher = Blake2b::new();
    let _n = io::copy(&mut file, &mut hasher).expect("Error hashing a file");
}
```

```rust

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
    // let test_path = "/home/sschlinkert";
```
