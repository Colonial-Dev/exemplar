use std::path::{Path, PathBuf};

use criterion::*;
use exemplar::*;

use rusqlite::Connection;
use rusqlite::types::ValueRef;

#[derive(Debug, PartialEq, Eq, Model)]
#[table("users")]
#[check("schema.sql")]
struct User {
    username: String,
    #[bind(bind_path)]
    #[extr(extr_path)]
    home_dir: PathBuf,
    #[column("pwd")]
    password: Vec<u8>,
}

pub fn bind_path(value: &Path) -> BindResult {
    use rusqlite::types::Value;
    use rusqlite::types::ToSqlOutput;

    let str = value.to_string_lossy().into_owned();

    Ok(ToSqlOutput::Owned(
        Value::Text(str)
    ))
}

pub fn extr_path(value: &ValueRef) -> ExtrResult<PathBuf> {
    let path = value.as_str()?;
    let path = PathBuf::from(path);

    Ok(path)
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut conn = Connection::open("benches/bench.db")
        .unwrap();

    conn.execute_batch(
        include_str!("schema.sql")
    ).unwrap();

    let alice = User {
        username: "Alice".to_owned(),
        home_dir: "/var/home/alice".into(),
        password: b"hunter2".as_slice().into(),
    };

    let txn = conn.transaction().unwrap();

    c.bench_function("insert", |b| b.iter(|| {
        alice.insert(&txn).unwrap();
    }));

    txn.commit().unwrap();

    let mut stmt = conn.prepare("SELECT * FROM users LIMIT 1")
        .unwrap();

    c.bench_function("retrieve", |b| b.iter(|| {
        stmt
            .query_and_then([], User::from_row)
            .unwrap()
            .map(Result::unwrap)
            .for_each(|u| {
                black_box(u);
            })
    }));

    drop(stmt);
    drop(conn);

    std::fs::remove_file("benches/bench.db")
        .unwrap();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);