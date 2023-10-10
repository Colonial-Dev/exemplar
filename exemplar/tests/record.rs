use anyhow::Result;

use exemplar::Model;
use exemplar::record;

record! {
    name => String,
    age  => u16
}

record! {
    Name => Age,
    age  => u16
}

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

    let mut get_people = conn.prepare("SELECT name, age FROM people ORDER BY name ASC")?;
    let mut get_ages = conn.prepare("SELECT age FROM people ORDER BY age ASC")?;

    let mut iter_people = get_people.query_and_then([], Record::from_row)?;

    let alice = iter_people.next().unwrap()?;
    let bob = iter_people.next().unwrap()?; 

    assert_eq!(alice.name, "Alice");
    assert_eq!(alice.age, 21);
    assert_eq!(bob.name, "Bob");
    assert_eq!(bob.age, 90);

    let mut iter_ages = get_ages.query_and_then([], Age::from_row)?;

    let alice = iter_ages.next().unwrap()?;
    let bob = iter_ages.next().unwrap()?; 

    assert_eq!(alice.age, 21);
    assert_eq!(bob.age, 90);

    Ok(())
}