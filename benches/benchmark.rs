use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rucash;

const URI:&str = "sqlite://tests/sqlite/sample/complex_sample.gnucash";

fn benchmark_sql_query(c: &mut Criterion) {
    let book = rucash::Book::new(URI).unwrap();
    c.bench_function("sql query", |b| {
        b.iter(|| book.accounts_contains_name(black_box("as")));
    });
}

fn benchmark_vec_filter(c: &mut Criterion) {
    let book = rucash::Book::new(URI).unwrap();
    c.bench_function("vec filter", |b| {
        b.iter(|| {
            let vec = book.accounts().unwrap();
            let accounts: Vec<rucash::sqlite::Account> = vec
                .into_iter()
                .filter(|x| x.name.to_lowercase().contains(black_box("as")))
                .collect();
        })
    });
}

criterion_group!(benches, benchmark_sql_query, benchmark_vec_filter,);
criterion_main!(benches);
