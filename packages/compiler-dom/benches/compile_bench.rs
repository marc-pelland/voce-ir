//! Compilation benchmarks for the DOM compiler.
//!
//! Run: `cargo bench -p voce-compiler-dom`

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

fn load_production_page() -> String {
    fs::read_to_string(
        workspace_root().join("examples/production/landing.voce.json"),
    )
    .expect("Failed to load production landing page")
}

fn minimal_ir() -> String {
    r#"{
        "schema_version_major": 1,
        "schema_version_minor": 0,
        "root": {
            "node_id": "root",
            "viewport_width": { "value": 1024, "unit": "Px" },
            "metadata": { "title": "Hello", "description": "Test" },
            "children": [
                {
                    "value_type": "TextNode",
                    "value": {
                        "node_id": "hello",
                        "content": "Hello, World!",
                        "font_size": { "value": 16, "unit": "Px" }
                    }
                }
            ]
        }
    }"#
    .to_string()
}

fn bench_compile_landing_page(c: &mut Criterion) {
    let json = load_landing_page();
    let options = voce_compiler_dom::CompileOptions {
        skip_fonts: true,
        ..Default::default()
    };

    c.bench_function("compile_landing_page", |b| {
        b.iter(|| {
            voce_compiler_dom::compile(black_box(&json), black_box(&options)).unwrap()
        })
    });
}

fn bench_compile_production_page(c: &mut Criterion) {
    let json = load_production_page();
    let options = voce_compiler_dom::CompileOptions {
        skip_fonts: true,
        ..Default::default()
    };

    c.bench_function("compile_production_page", |b| {
        b.iter(|| {
            voce_compiler_dom::compile(black_box(&json), black_box(&options)).unwrap()
        })
    });
}

fn bench_compile_minimal(c: &mut Criterion) {
    let json = minimal_ir();
    let options = voce_compiler_dom::CompileOptions::default();

    c.bench_function("compile_minimal_textnode", |b| {
        b.iter(|| {
            voce_compiler_dom::compile(black_box(&json), black_box(&options)).unwrap()
        })
    });
}

fn bench_output_size(c: &mut Criterion) {
    let json = minimal_ir();
    let options = voce_compiler_dom::CompileOptions::default();

    c.bench_function("output_size_minimal", |b| {
        b.iter(|| {
            let result = voce_compiler_dom::compile(black_box(&json), black_box(&options)).unwrap();
            assert!(result.size_bytes < 5000, "Minimal page should be under 5KB, got {}", result.size_bytes);
            result.size_bytes
        })
    });
}

criterion_group!(
    benches,
    bench_compile_landing_page,
    bench_compile_production_page,
    bench_compile_minimal,
    bench_output_size,
);
criterion_main!(benches);
