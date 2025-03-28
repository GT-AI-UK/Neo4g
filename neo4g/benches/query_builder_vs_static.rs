use neo4rs::Graph;
use heck::ToShoutySnakeCase;
use dotenv::dotenv;
use std::env;
use criterion::{BenchmarkId, criterion_group, criterion_main, Criterion};
use neo4g::benches::query_builder_string::query_builder_string_bench;
use neo4g::benches::static_string::static_string_bench;
use neo4g::benches::query_builder_query::query_builder_query_bench;
use neo4g::benches::static_query::static_query_bench;

pub fn criterion_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("ss", |b| b.iter(|| static_string_bench()));
    c.bench_function("qbs", |b| b.iter(|| query_builder_string_bench()));
    c.bench_function("sq", |b| {
        b.to_async(&rt).iter(|| async {
            static_query_bench().await;
        });
    });
    c.bench_function("qbq", |b| {
        b.to_async(&rt).iter(|| async {
            query_builder_query_bench().await;
        });
    });
    c.bench_function("sq", |b| {
        b.to_async(&rt).iter(|| async {
            static_query_bench().await;
        });
    });
    c.bench_function("qbq", |b| {
        b.to_async(&rt).iter(|| async {
            query_builder_query_bench().await;
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);