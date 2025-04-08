use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn uri_sqlite() -> String {
    format!(
        "file:/{}/tests/db/sqlite/complex_sample.gnucash",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn uri_xml() -> String {
    format!(
        "{}/tests/db/xml/complex_sample.gnucash",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn benchmark_sql_query(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let book = rt.block_on(async {
        let query = rucash::SQLiteQuery::new(&uri_sqlite()).unwrap();
        rucash::Book::new(query).await.unwrap()
    });

    c.bench_function("sql query", |b| {
        b.to_async(&rt).iter(|| async {
            book.accounts_contains_name_ignore_case(black_box("aS"))
                .await
        });
    });
}

fn benchmark_vec_filter(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let book = rt.block_on(async {
        let query = rucash::SQLiteQuery::new(&uri_sqlite()).unwrap();
        rucash::Book::new(query).await.unwrap()
    });

    c.bench_function("vec filter", |b| {
        b.to_async(&rt).iter(|| async {
            let vec = book.accounts().await.unwrap();
            let _: Vec<_> = vec
                .into_iter()
                .filter(|x| x.name.to_lowercase().contains(black_box("aS")))
                .collect();
        })
    });
}

fn benchmark_xml_book(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let book = rt.block_on(async {
        let query = rucash::XMLQuery::new(&uri_xml()).unwrap();
        rucash::Book::new(query).await.unwrap()
    });

    c.bench_function("XMLBook", |b| {
        b.to_async(&rt).iter(|| async { book.accounts().await })
    });
}

fn benchmark_sqlite_book(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let book = rt.block_on(async {
        let query = rucash::SQLiteQuery::new(&uri_sqlite()).unwrap();
        rucash::Book::new(query).await.unwrap()
    });

    c.bench_function("SqliteBook", |b| {
        b.to_async(&rt).iter(|| async { book.accounts().await })
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
