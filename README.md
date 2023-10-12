<h1 align="center">Exemplar</h1>
<h3 align="center">A boilerplate eliminator for <code>rusqlite</code>.</h3>

<p align="center">
<img src="https://img.shields.io/crates/v/exemplar">
<img src="https://img.shields.io/github/actions/workflow/status/Colonial-Dev/exemplar/rust.yml">
<img src="https://img.shields.io/docsrs/exemplar">
<img src="https://img.shields.io/crates/l/exemplar">
</p>

## Getting Started
A taste of what you can do:
```rs
use std::path::{
    PathBuf,
    Path
};
 
use exemplar::Model;
use rusqlite::Result;
use rusqlite::Connection;
 
#[derive(Debug, PartialEq, Model)]
#[table("users")]
#[check("../tests/schema.sql")]
struct User {
   username: String,
   #[bind(bind_path)]
   #[extr(extr_path)]
   home_dir: PathBuf,
   #[column("pwd")]
   password: Vec<u8>,
}
 
fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
 
    conn.execute_batch(
        include_str!("../tests/schema.sql")
    )?;
 
    let alice = User {
        username: "Alice".to_owned(),
        home_dir: "/var/home/alice".into(),
        password: b"hunter2".to_vec()
    };
 
    let bob = User {
        username: "Bob".to_owned(),
        home_dir: "/var/home/robert".into(),
        password: b"password".to_vec()
    };
 
    alice.insert(&conn)?;
    bob.insert(&conn)?;
 
    let mut stmt = conn.prepare("SELECT * FROM users ORDER BY username ASC")?;
    let mut iter = stmt.query_and_then([], User::from_row)?;
 
    assert_eq!(alice, iter.next().unwrap()?);
    assert_eq!(bob, iter.next().unwrap()?);
 
    Ok(())
}
```

Exemplar is based around the [`Model`](https://docs.rs/exemplar/latest/exemplar/trait.Model.html) trait, which has its own [derive macro](https://docs.rs/exemplar/latest/exemplar/derive.Model.html).

- See the aformentioned [macro](https://docs.rs/exemplar/latest/exemplar/derive.Model.html)'s documentation to get started.
- For handling `enum`s in models, check out the [`sql_enum`](https://docs.rs/exemplar/latest/exemplar/macro.sql_enum.html) macro.
- For working with "anonymous" record types, look at the [`record`](https://docs.rs/exemplar/latest/exemplar/macro.record.html) macro.

## Features
- Works with raw SQL, not against it.
- Thin, zero-cost API.
  - Most of Exemplar revolves around the `Model` trait, which gets inlined and monomorphized away before runtime. The resulting code is roughly what you'd write by hand when using pure `rusqlite`.
  - Designed to be drop-in; reuses `rusqlite`'s existing types where possible, including its `Result` type alias.
  - Supports any type that `Deref`'s to `rusqlite::Connection`, such as transactions or pooled connections.
- Optional test derivation for guarding against drift between your database schema and Rust model types.
- Macros for working with SQL-compatible `enum`s and "anonymous" record types that map to ad-hoc queries.
- Some ability to reflect on/work with `dyn Model`s at runtime.

If you just need to CRUD some Rust data with `sqlite` and don't want a whole ORM or enterprise-grade DBMS, then Exemplar is for you!

## FAQ
### *"What does Exemplar not do?"*
A few key things:

- Schema generation and management. Exemplar is explicitly not an ORM, and it's difficult to represent concepts like foreign keys and migrations
without falling into ORM territory.
  - If this is a "must" for you, check out `diesel` or `sqlx`/`seaorm`, which both support SQLite.
- Query generation (excluding `INSERT`.)
- Interface portability. Only `rusqlite` is supported.

### *"Is it blazing fast?"*
Yes. On my machine (according to [this](https://github.com/Colonial-Dev/exemplar/blob/master/exemplar/benches/insert.rs) benchmark) Exemplar can insert a non-trivial model type in ~600 nanoseconds - which works out to about eight million rows in five seconds. 

Obviously the credit for this speed goes to the SQLite and `rusqlite` developers, but I can confidently say that I didn't slow things down!

### *"How does this compare to `serde-rusqlite`?"*
`serde_rusqlite` is a clever hack, but it still involved too much contorting and boilerplate for my taste - that's why I created Exemplar.

The pain points I tried to fix were:
- Needing to allocate and juggle a slice of `String` column names to efficiently deserialize rows. Exemplar statically knows what columns to expect, so `from_row` can extract each field in a single line.
- Odd design choices for field-less `enum`s - they are inefficiently serialized as `TEXT` instead of `INTEGER`. This was nice for debugging, but I figured the faster option should be Exemplar's default.
- General `serde` overhead popping up, both at compile and runtime.

## Acknowledgements
- `rusqlite`, for providing the foundation on which this library is built.
- David Tolnay, for his various proc macro ~~incantations~~ crates.