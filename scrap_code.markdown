One way to get relative path

```rust
fn get_path_relative_to_dir<'a>(dir_path: &Path, full_path: &'a Path) -> &'a Path {
    let length_of_dir_path = dir_path.components().count();
    let mut rel_path_components = full_path.components();
    for _n in 0..length_of_dir_path {
        rel_path_components.next();
    }
    rel_path_components.as_path()
}
```

Sorting a vector with Rayon

```rust
// We have to sort entries because WalkDir doesn't walk the same way each run
// Here's one way to do that, in parallel.
fn sort_dir_par(dir_path: &Path) -> Vec<walkdir::DirEntry> {
    let mut sorted_entries = vec![];
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        sorted_entries.push(entry)
    }
    use rayon::slice::ParallelSliceMut;
    sorted_entries.par_sort_unstable_by(|a, b| a.path().partial_cmp(b.path()).unwrap());
    sorted_entries
}
```

An old directory hash function

```rust
fn hash_dir_old(dir_path: &str) -> blake3::Hash {
    let mut hasher = blake3::Hasher::new();

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.metadata().unwrap().is_file() {
            let mut file = fs::File::open(&entry.path()).expect("Error opening a file for hashing");
            let _n = io::copy(&mut file, &mut hasher).expect("Error hashing a file");
        }
    }
    hasher.finalize()
}
```

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


More old hash_dir

```rust
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
```
