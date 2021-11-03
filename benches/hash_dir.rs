use criterion::{criterion_group, criterion_main, Criterion};
use same::*;
use std::path::Path;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hash a directory");
    // group.significance_level(0.1).sample_size(100);
    group.sample_size(25);
    let path = Path::new("/home/sschlinkert/Pictures");

    group.bench_function("t=4", |b| {
        b.iter(|| hash_dir(path, 4, false));
    });
    group.bench_function("t=2", |b| {
        b.iter(|| hash_dir(path, 2, false));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
