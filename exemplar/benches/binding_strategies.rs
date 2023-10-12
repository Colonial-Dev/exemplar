//!  Benchmarks comparing the `dyn ToSql` and raw API.
//! 
//! These show that they're about the same, so we use the simpler one in Exemplar.

use criterion::*;

use anyhow::Result;
use rusqlite::Connection;

fn binding_dyn(conn: &Connection, data: i64) -> Result<()> {
    let mut stmt = conn.prepare("
        INSERT INTO bench VALUES(:a, :b, :c, :d, :e);
    ")?;

    stmt.execute(rusqlite::named_params! {
        ":a": data,
        ":b": data,
        ":c": data,
        ":d": data,
        ":e": data
    })?;
    
    Ok(())
}

fn binding_raw(conn: &Connection, data: i64) -> Result<()> {
    let mut stmt = conn.prepare("
        INSERT INTO bench VALUES(:a, :b, :c, :d, :e);
    ")?;

    let a = stmt.parameter_index(":a")?.unwrap();
    let b = stmt.parameter_index(":b")?.unwrap();
    let c = stmt.parameter_index(":c")?.unwrap();
    let d = stmt.parameter_index(":d")?.unwrap();
    let e = stmt.parameter_index(":e")?.unwrap();
    
    stmt.raw_bind_parameter(a, data)?;
    stmt.raw_bind_parameter(b, data)?;
    stmt.raw_bind_parameter(c, data)?;
    stmt.raw_bind_parameter(d, data)?;
    stmt.raw_bind_parameter(e, data)?;

    stmt.raw_execute()?;

    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut data = 0_i64;

    let conn = Connection::open_in_memory()
        .unwrap();

    conn.execute("CREATE TABLE bench (a, b, c, d, e)", [])
        .unwrap();

    c.bench_function("dyn binding", |b| b.iter(|| {
        binding_dyn(
            &conn,
            black_box(data)
        )
        .unwrap();

        data += 1;
    }));

    data = 0;

    c.bench_function("raw binding", |b| b.iter(|| {
        binding_raw(
            &conn,
            black_box(data)
        )
        .unwrap();

        data += 1;
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);