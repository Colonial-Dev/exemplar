use std::path::{Path, PathBuf};

use criterion::*;
use exemplar::*;

use rusqlite::{Connection, Row};
use rusqlite::types::ValueRef;

#[derive(Debug, PartialEq, Eq)]
struct User {
    username: String,
    home_dir: PathBuf,
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

    c.bench_function("insert (manual)", |b| b.iter(|| {
        let mut stmt = txn.prepare_cached("
            INSERT INTO users (username, home_dir, pwd) 
            VALUES(:username, :home_dir, :pwd);
        ").unwrap();
        
        stmt.execute(rusqlite::named_params! {
            ":username": alice.username,
            ":home_dir": bind_path(&alice.home_dir).unwrap(),
            ":pwd": alice.password
        }).unwrap();
    }));

    txn.commit().unwrap();

    let mut stmt = conn.prepare_cached("SELECT * FROM users LIMIT 1")
        .unwrap();

    let to_user = |row: &Row| -> rusqlite::Result<_> {
        Ok(User {
            username: row.get("username")?,
            home_dir: extr_path(&row.get_ref("home_dir")?)?,
            password: row.get("pwd")?
        })
    };

    c.bench_function("retrieve (manual)", |b| b.iter(|| {
        stmt
            .query_and_then([], to_user)
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