//! Criterion bench harness. Keep benches fast so CI can enforce a
//! >10% regression gate (see Constitution V).
//!
//! Covered budgets:
//!   * `app_config::touch_recent 50x` — Welcome-screen recent list churn.
//!   * `highlight_rules::first_match` — FR-045 terminal highlight, ≤ 10 µs
//!     per line on the reference corpus.
//!   * `doc_hash_store::compute_sha256_hex` — FR-085 document hash recompute,
//!     ≤ 5 ms on a 1 MiB file.

use std::io::Write as _;

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use speclens_lib::models::{AppConfig, RecentProject};
use speclens_lib::services::doc_hash_store::compute_sha256_hex;
use speclens_lib::services::highlight_rules::{default_compiled, first_match};
use tempfile::tempdir;
use uuid::Uuid;

fn bench_touch_recent(c: &mut Criterion) {
    c.bench_function("app_config::touch_recent 50x", |b| {
        b.iter(|| {
            let mut cfg = AppConfig::default();
            for i in 0..50 {
                let entry = RecentProject {
                    id: Uuid::new_v4(),
                    name: format!("p{i}"),
                    path: format!("/tmp/p{i}"),
                    last_opened_at: Utc::now(),
                    pinned: false,
                };
                cfg.touch_recent(entry);
            }
            black_box(cfg.recent_projects.len());
        });
    });
}

/// Mixed severity lines mirroring what the Spec-Kit terminal emits during a
/// typical `/speckit.implement` run. Keeps the bench deterministic while
/// exercising every branch of the default rule set.
const HIGHLIGHT_CORPUS: &[&str] = &[
    "ERROR: build failed at src/lib.rs:42:3",
    "WARNING: deprecated API used in src/main.rs",
    ">>> cargo test --all-features",
    "  PASS   src/services/highlight_rules.rs",
    r#"  { "level": "info", "msg": "step done" }  "#,
    "src/services/doc_hash_store.rs:14:1 — warning about unused import",
    "running 8 tests",
    "DONE in 1.23s",
    "note: run with `RUST_BACKTRACE=1` for a backtrace",
    "plain terminal output without any rule matches",
];

fn bench_highlight_rules(c: &mut Criterion) {
    let rules = default_compiled();
    let mut group = c.benchmark_group("highlight_rules");
    group.throughput(Throughput::Elements(HIGHLIGHT_CORPUS.len() as u64));
    group.bench_function("first_match/corpus", |b| {
        b.iter(|| {
            for line in HIGHLIGHT_CORPUS {
                black_box(first_match(black_box(line), rules));
            }
        });
    });
    group.finish();
}

fn bench_doc_hash_recompute(c: &mut Criterion) {
    // Seed a 1 MiB deterministic file once; the bench measures the hot
    // recompute path (read + SHA-256 over the whole blob).
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("doc.md");
    {
        let chunk: Vec<u8> = (0u8..=255).cycle().take(1024).collect();
        let mut f = std::fs::File::create(&path).expect("create doc");
        for _ in 0..1024 {
            f.write_all(&chunk).expect("write chunk");
        }
        f.flush().expect("flush");
    }

    let mut group = c.benchmark_group("doc_hash_store");
    group.throughput(Throughput::Bytes(1024 * 1024));
    group.bench_function("compute_sha256_hex/1MiB", |b| {
        b.iter(|| {
            let hex = compute_sha256_hex(black_box(&path)).expect("hash ok");
            black_box(hex);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_touch_recent,
    bench_highlight_rules,
    bench_doc_hash_recompute
);
criterion_main!(benches);
