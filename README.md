<h1 align="center">Exemplar</h1>
<h3 align="center">A boilerplate eliminator for <code>rusqlite</code>.</h3>

<p align="center">
<img src="https://img.shields.io/crates/v/exemplar">
<img src="https://img.shields.io/github/actions/workflow/status/Colonial-Dev/exemplar/rust.yml">
<img src="https://img.shields.io/docsrs/exemplar" href="https://docs.rs/exemplar">
<img src="https://img.shields.io/crates/l/exemplar">
</p>

## Features
- Works with raw SQL, not against it. Exemplar is *not* an ORM.
- Thin, zero-cost API.
  - Most of Exemplar revolves around the `Model` trait, which gets inlined and monomorphized away before runtime. The resulting code is roughly what you'd write by hand when using pure `rusqlite`.
  - Designed to be drop-in; reuses `rusqlite`'s existing types where possible, including its `Result` type alias.
  - Supports any type that `Deref`'s to `rusqlite::Connection`, such as transactions or pooled connections.
- Optional test derivation for guarding against drift between your database schema and Rust model types.
- Macros for working with SQL-compatible `enum`s and "anonymous" record types that map to ad-hoc queries.
- Some ability to reflect on/work with `dyn Model`s at runtime.

If you just need to CRUD some Rust data with `sqlite` and don't want a whole ORM or enterprise-grade DBMS, then Exemplar is for you!

## Cargo Features
- (Default) `sql_enum` - enables the `sql_enum` macro. Depends on `num_enum`.

## Won't Support
- Schema generation and management. It's very difficult to represent concepts like foreign keys and migrations without falling into ORM territory.
  - If this is a "must" for you, check out `diesel` or `sqlx`/`seaorm`.
- Database interfaces besides `rusqlite`. 
  - (I may extend support to `sqlite` if there is demand, but it's likely to be less performant as it lacks statement caching.)

## Acknowledgements
- `rusqlite`, for providing the foundation on which this library is built.
- David Tolnay, for his various proc macro ~~incantations~~ crates.