//! Criterion bench harness. Keep benches fast so CI can enforce a
//! >10% regression gate (see Constitution V).

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use speclens_lib::models::{AppConfig, RecentProject};
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

criterion_group!(benches, bench_touch_recent);
criterion_main!(benches);
