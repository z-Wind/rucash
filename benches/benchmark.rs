use criterion::{black_box, criterion_group, criterion_main, Criterion};

const URI: &str = "sqlite://tests/db/sqlite/complex_sample.gnucash?mode=ro";
fn uri() -> String {
    format!(
        "sqlite://{}/tests/db/sqlite/complex_sample.gnucash?mode=ro",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn benchmark_sql_query(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    //panic!("{}", uri());

    c.bench_function("sql query", |b| {
        b.to_async(&rt).iter(|| async {
            let book = rucash::SqliteBook::new(&uri()).await.unwrap();
            book.accounts_contains_name(black_box("aS")).await
        });
    });
}

fn benchmark_vec_filter(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("vec filter", |b| {
        b.to_async(&rt).iter(|| async {
            let book = rucash::SqliteBook::new(&uri()).await.unwrap();
            let vec = book.accounts().await.unwrap();
            let _: Vec<_> = vec
                .into_iter()
                .filter(|x| x.name.to_lowercase().contains(black_box("aS")))
                .collect();
        })
    });
}

fn benchmark_xml_book(c: &mut Criterion) {
    let book = rucash::XMLBook::new("tests/db/xml/complex_sample.gnucash").unwrap();
    c.bench_function("XMLBook", |b| b.iter(|| book.accounts()));
}

fn benchmark_sqlite_book(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("SqliteBook", |b| {
        b.to_async(&rt).iter(|| async {
            let book = rucash::SqliteBook::new(&uri()).await.unwrap();
            book.accounts().await
        })
    });
}

criterion_group!(
    benches,
    benchmark_sql_query,
    benchmark_vec_filter,
    benchmark_xml_book,
    benchmark_sqlite_book,
);
criterion_main!(benches);
