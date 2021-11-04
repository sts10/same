use criterion::{criterion_group, criterion_main, Criterion};
use same::*;
use std::path::Path;
use std::time::Duration;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hash a directory");
    // https://docs.rs/criterion/0.3.5/criterion/struct.BenchmarkGroup.html
    group.measurement_time(Duration::new(20, 0));
    let path = Path::new("/home/sschlinkert/Pictures");

    group.bench_function("t=4", |b| {
        b.iter(|| hash_dir(path, 4, false, false, &None));
    });
    group.bench_function("t=2", |b| {
        b.iter(|| hash_dir(path, 2, false, false, &None));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
