use criterion::{Criterion, criterion_group, criterion_main};
use polarity_lang_driver::{Database, FileSource, InMemorySource};
use url::Url;

const EXAMPLE_STLC: &str = include_str!("../../examples/stlc.pol");
const EXAMPLE_STRONG_EX: &str = include_str!("../../examples/strong_existentials.pol");

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
    let mut inmemory_source = InMemorySource::new();
    let uri: Url = "inmemory:///bench.pol".parse().expect("Failed to parse URI");
    inmemory_source.write_string(&uri, example).await.expect("Failed to write inmemory source");
    let mut db = Database::from_source(inmemory_source);
    let _ = db.ast(&uri).await.map_err(|err| db.pretty_error(&uri, err));
    Ok(())
}

criterion_group!(benches, benchmark_stlc, benchmark_strong_existentials);
criterion_main!(benches);
