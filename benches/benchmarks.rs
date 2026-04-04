//! Benchmarks for hcscoder
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_api_key_validation(c: &mut Criterion) {
    c.bench_function("validate_api_key_valid", |b| {
        b.iter(|| {
            hcscoder::hcscoder_openrouter::auth::validate_api_key(black_box("sk-or-a1b2c3d4e5f6g7h8"))
        })
    });

    c.bench_function("validate_api_key_invalid", |b| {
        b.iter(|| {
            hcscoder::hcscoder_openrouter::auth::validate_api_key(black_box("invalid-key"))
        })
    });
}

fn benchmark_path_validation(c: &mut Criterion) {
    c.bench_function("validate_safe_path", |b| {
        b.iter(|| {
            // Placeholder - actual benchmark needs implementation
            black_box("/tmp/test.txt")
        })
    });
}

criterion_group!(benches, benchmark_api_key_validation, benchmark_path_validation);
criterion_main!(benches);
