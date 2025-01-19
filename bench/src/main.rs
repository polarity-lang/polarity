use criterion::{criterion_group, criterion_main, Criterion};
use driver::Database;

const EXAMPLE_STLC: &str = "examples/stlc.pol";
const EXAMPLE_STRONG_EX: &str = "examples/strong_existentials.pol";

fn benchmark_stlc(c: &mut Criterion) {
    c.bench_function("stlc.pol", |b| b.iter(|| run(EXAMPLE_STLC)));
}

fn benchmark_strong_existentials(c: &mut Criterion) {
    c.bench_function("strong_existentials.pol", |b| b.iter(|| run(EXAMPLE_STRONG_EX)));
}

fn run(example: &str) -> miette::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run_async(example))
}

async fn run_async(example: &str) -> miette::Result<()> {
    let mut db = Database::from_path(example);
    let uri = db.resolve_path(example)?;
    let _ = db.ast(&uri).await.map_err(|err| db.pretty_error(&uri, err));
    Ok(())
}

criterion_group!(benches, benchmark_stlc, benchmark_strong_existentials);
criterion_main!(benches);
