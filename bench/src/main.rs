use criterion::{criterion_group, criterion_main, Criterion};
use driver::Database;

const EXAMPLE: &str = "examples/stlc.pol";

fn benchmark(c: &mut Criterion) {
    c.bench_function("stlc.pol", |b| b.iter(|| run()));
}

fn run() -> miette::Result<()> {
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(run_async())
}

async fn run_async() -> miette::Result<()> {
    let mut db = Database::from_path(EXAMPLE);
    let uri = db.resolve_path(EXAMPLE)?;
    let _ = db.ast(&uri).await.map_err(|err| db.pretty_error(&uri, err));
    Ok(())
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
