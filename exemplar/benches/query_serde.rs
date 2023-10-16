use criterion::*;

use serde::{Serialize, Deserialize};
use serde_rusqlite::*;

use rusqlite::Connection;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct User {
    username: String,
    home_dir: String,
    #[serde(rename = "pwd")]
    password: Vec<u8>,
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

    let mut stmt = txn.prepare("
        INSERT INTO users (username, home_dir, pwd) VALUES(:username, :home_dir, :pwd);
    ").unwrap();

    c.bench_function("insert (serde_rusqlite)", |b| b.iter(|| {
        stmt.execute(to_params_named(black_box(&alice)).unwrap().to_slice().as_slice()).unwrap();
    }));

    drop(stmt);
    txn.commit().unwrap();

    let mut stmt = conn.prepare("SELECT * FROM users LIMIT 1")
        .unwrap();

    let columns = columns_from_statement(&stmt);

    c.bench_function("retrieve (serde_rusqlite)", |b| b.iter(|| {
        stmt
            .query_and_then([], |row| from_row_with_columns::<User>(row, &columns))
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