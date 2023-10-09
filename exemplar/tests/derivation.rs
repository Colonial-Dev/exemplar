use exemplar::Model;

#[derive(Model)]
#[table("foo")]
#[check("schema.sql")]
struct Foo {
    bar: String,
    #[column("qux")]
    #[bind(bind_str)]
    #[extr(extr_str)]
    baz: String,
}

pub fn bind_str(value: &String) -> exemplar::BindResult {
    use rusqlite::ToSql;

    value.to_sql()
}

pub fn extr_str(value: rusqlite::types::ValueRef<'_>) -> exemplar::ExtrResult<String> {
    value.as_str().map(str::to_owned)
}