use exemplar::*;

sql_enum! {
    Name => Color,
    Type => u8,
    Red,
    Green,
    Blue
}

#[test]
fn conversion() {
    assert_eq!(0, Color::Red as i64);
    assert_eq!(1, Color::Green as i64);
    assert_eq!(2, Color::Blue as i64);

    assert_eq!(
        Color::Red,
        Color::try_from(0).unwrap()
    );

    assert_eq!(
        Color::Green,
        Color::try_from(1).unwrap()
    );

    assert_eq!(
        Color::Blue,
        Color::try_from(2).unwrap()
    );
}

#[test]
fn safety() {
    assert!(
        Color::try_from(-1).is_err()
    );

    assert!(
        Color::try_from(i64::MIN).is_err()
    );

    assert!(
        Color::try_from(i64::MAX).is_err()
    );

    assert!(
        Color::try_from(3).is_err()
    );
}