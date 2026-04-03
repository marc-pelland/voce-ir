//! Validation benchmarks.
//!
//! Run: `cargo bench -p voce-validator`

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn load_landing_page() -> String {
    fs::read_to_string(
        workspace_root().join("examples/landing-page/landing-page.voce.json"),
    )
    .expect("Failed to load landing page fixture")
}

fn bench_validate_landing_page(c: &mut Criterion) {
    let json = load_landing_page();

    c.bench_function("validate_landing_page", |b| {
        b.iter(|| {
            voce_validator::validate(black_box(&json)).unwrap()
        })
    });
}

fn bench_validate_and_compile(c: &mut Criterion) {
    let json = load_landing_page();
    let options = voce_compiler_dom::CompileOptions {
        skip_fonts: true,
        ..Default::default()
    };

    c.bench_function("validate_and_compile_landing_page", |b| {
        b.iter(|| {
            let _val = voce_validator::validate(black_box(&json)).unwrap();
            voce_compiler_dom::compile(black_box(&json), black_box(&options)).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_validate_landing_page,
    bench_validate_and_compile,
);
criterion_main!(benches);
