use std::path::{Path, PathBuf};

use anyhow::Result;

use exemplar::Model;
use exemplar::{
    BindResult,
    ExtrResult
};

use rusqlite::types::ValueRef;

// Simple case
#[derive(Debug, PartialEq, Eq, Model)]
#[table("people")]
struct Person {
    name: String,
    age: u16,
    alive: bool, 
}

#[test]
fn test_person() -> Result<()> {
    use rusqlite::Connection;

    let conn = Connection::open_in_memory()
        .unwrap();

    conn.execute_batch("
        CREATE TABLE people (name, age, alive);
    ")?;

    let alice = Person {
        name: "Alice".to_owned(),
        age: 21,
        alive: true
    };
    
    let bob = Person {
        name: "Bob".to_owned(),
        age: 90,
        alive: false
    };

    alice.insert(&conn)?;
    bob.insert(&conn)?;

    let mut stmt = conn.prepare("SELECT * FROM people ORDER BY name ASC")?;

    let mut iter = stmt.query_and_then([], Person::from_row)?;

    assert_eq!(alice, iter.next().unwrap()?);
    assert_eq!(bob, iter.next().unwrap()?);

    Ok(())
}

// Complicated case
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

#[test]
fn test_user() -> Result<()> {
    use rusqlite::Connection;

    let conn = Connection::open_in_memory()
        .unwrap();

    conn.execute_batch(
        include_str!("schema.sql")
    )?;

    let alice = User {
        username: "Alice".to_owned(),
        home_dir: "/var/home/alice".into(),
        password: b"hunter2".as_slice().into(),
    };
    
    let bob = User {
        username: "Bob".to_owned(),
        home_dir: "/var/home/robert".into(),
        password: b"password".as_slice().into(),
    };

    alice.insert(&conn)?;
    bob.insert(&conn)?;

    let mut stmt = conn.prepare("SELECT * FROM users ORDER BY username ASC")?;

    let mut iter = stmt.query_and_then([], User::from_row)?;

    assert_eq!(alice, iter.next().unwrap()?);
    assert_eq!(bob, iter.next().unwrap()?);

    Ok(())
}