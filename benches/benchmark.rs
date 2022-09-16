use criterion::{black_box, criterion_group, criterion_main, Criterion};

const URI: &str = "sqlite://tests/db/sqlite/complex_sample.gnucash?mode=ro";

fn benchmark_sql_query(c: &mut Criterion) {
    let book = rucash::SqliteBook::new(URI).unwrap();
    c.bench_function("sql query", |b| {
        b.iter(|| book.accounts_contains_name(black_box("aS")));
    });
}

fn benchmark_vec_filter(c: &mut Criterion) {
    let book = rucash::SqliteBook::new(URI).unwrap();
    c.bench_function("vec filter", |b| {
        b.iter(|| {
            let vec = book.accounts().unwrap();
            let _: Vec<_> = vec
                .into_iter()
                .filter(|x| x.name.to_lowercase().contains(black_box("aS")))
                .collect();
        })
    });
}

fn benchmark_XMLBook(c: &mut Criterion) {
    let book = rucash::XMLBook::new("tests/db/xml/complex_sample.gnucash").unwrap();
    c.bench_function("XMLBook", |b| b.iter(|| book.accounts()));
}

fn benchmark_SqliteBook(c: &mut Criterion) {
    let book = rucash::SqliteBook::new(URI).unwrap();
    c.bench_function("SqliteBook", |b| b.iter(|| book.accounts()));
}

criterion_group!(
    benches,
    benchmark_sql_query,
    benchmark_vec_filter,
    benchmark_XMLBook,
    benchmark_SqliteBook,
);
criterion_main!(benches);
