use anyhow::Result;

use exemplar::sql_enum;
use exemplar::Model;

sql_enum!(
    Name => Gender,
    Male,
    Female
);

#[derive(Model, Debug, PartialEq, Eq)]
#[table("people")]
pub struct Person {
    pub name: String,
    pub gender: Gender,
    pub age: u16,
}

#[test]
fn test_person() -> Result<()> {
    use rusqlite::Connection;

    let conn = Connection::open_in_memory()
        .unwrap();

    conn.execute_batch("
        CREATE TABLE people (name, gender, age);
    ")?;

    let alice = Person {
        name: "Alice".to_owned(),
        gender: Gender::Female,
        age: 21,
    };
    
    let bob = Person {
        name: "Bob".to_owned(),
        gender: Gender::Male,
        age: 90,
    };

    alice.insert(&conn)?;
    bob.insert(&conn)?;

    let mut stmt = conn.prepare("SELECT * FROM people ORDER BY name ASC")?;

    let mut iter = stmt.query_and_then([], Person::from_row)?;

    assert_eq!(alice, iter.next().unwrap()?);
    assert_eq!(bob, iter.next().unwrap()?);

    Ok(())
}