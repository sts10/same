extern crate fxhash;

use criterion::{criterion_group, criterion_main, Criterion};
// use fasthash::{xx, XXHasher};
use fasthash::FastHasher;
// use fxhash::FxHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
// use std::{fs, io};
use std::io;

// Pasted this from src/main.rs for now
fn hash_file(path: &Path, hasher: &mut impl Hasher) -> Result<(), io::Error> {
    let file = File::open(path)?;
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

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hash a file");
    group.significance_level(0.1).sample_size(300);
    let path = Path::new("../README.markdown");

    group.bench_function("AHash", |b| {
        let mut hasher = ahash::AHasher::default();
        b.iter(|| hash_file(path, &mut hasher));
    });
    group.bench_function("SeaHash", |b| {
        let mut hasher = seahash::SeaHasher::new();
        b.iter(|| hash_file(path, &mut hasher));
    });
    group.bench_function("FxHash", |b| {
        let mut hasher = fxhash::FxHasher::default();
        b.iter(|| hash_file(path, &mut hasher));
    });
    group.bench_function("xx", |b| {
        let mut hasher = fasthash::xx::Hasher64::new();
        b.iter(|| hash_file(path, &mut hasher));
    });
    group.bench_function("farm", |b| {
        let mut hasher = fasthash::farm::Hasher64::new();
        b.iter(|| hash_file(path, &mut hasher));
    });
    group.bench_function("city", |b| {
        let mut hasher = fasthash::city::Hasher64::new();
        b.iter(|| hash_file(path, &mut hasher));
    });
    group.bench_function("mum", |b| {
        let mut hasher = fasthash::mum::Hasher64::new();
        b.iter(|| hash_file(path, &mut hasher));
    });
    group.bench_function("spooky", |b| {
        let mut hasher = fasthash::spooky::Hasher64::new();
        b.iter(|| hash_file(path, &mut hasher));
    });
    group.bench_function("murmur2", |b| {
        let mut hasher = fasthash::murmur2::Hasher64_x64::new();
        b.iter(|| hash_file(path, &mut hasher));
    });
    group.bench_function("metro", |b| {
        let mut hasher = fasthash::metro::crc::Hasher64_1::new();
        b.iter(|| hash_file(path, &mut hasher));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
