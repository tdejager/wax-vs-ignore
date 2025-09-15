use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wax_vs_ignore::{
    collect_with_bfs_fastglob, collect_with_globwalk, collect_with_ignore, collect_with_wax, PATTERNS,
};

fn bench_wax(c: &mut Criterion) {
    let root = Path::new(".");
    c.bench_function("wax_total", |b| {
        b.iter(|| {
            let v = collect_with_wax(root, PATTERNS).expect("wax collection failed");
            black_box(v.len())
        })
    });
}

fn bench_ignore(c: &mut Criterion) {
    let root = Path::new(".");
    c.bench_function("ignore_total", |b| {
        b.iter(|| {
            let v = collect_with_ignore(root, PATTERNS).expect("ignore collection failed");
            black_box(v.len())
        })
    });
}

fn bench_globwalk(c: &mut Criterion) {
    let root = Path::new(".");
    c.bench_function("globwalk_total", |b| {
        b.iter(|| {
            let v = collect_with_globwalk(root, PATTERNS).expect("globwalk collection failed");
            black_box(v.len())
        })
    });
}

fn bench_bfs_fastglob(c: &mut Criterion) {
    let root = Path::new(".");
    c.bench_function("bfs_fastglob_total", |b| {
        b.iter(|| {
            let v =
                collect_with_bfs_fastglob(root, PATTERNS, &["**/.*/**"]).expect("bfs collection failed");
            black_box(v.len())
        })
    });
}

criterion_group!(benches, bench_wax, bench_ignore, bench_globwalk, bench_bfs_fastglob);
criterion_main!(benches);
