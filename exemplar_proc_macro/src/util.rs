use super::*;

pub struct Derivee<'a> {
    pub name: Ident,
    pub table: String,
    pub fields: Vec<&'a Field>,
    pub schema: Option<String>,
}

impl Derivee<'_> {
    pub fn field_idents(&self) -> impl Iterator<Item = &Ident> {
        self
            .fields
            .iter()
            .map(|field| {
                field
                    .ident
                    .as_ref()
                    .expect("All fields should have an indentifier.")
            })
    }

    pub fn col_names(&self) -> impl Iterator<Item = String> + '_ {
        self
            .fields
            .iter()
            .copied()
            .map(get_col_name)
    }

    pub fn gen_query(&self, clause: Option<&str>) -> Literal {
        let mut buf = String::from("INSERT ");

        if let Some(clause) = clause {
            buf += &format!("OR {} ", clause);
        }

        buf += &format!("INTO {} ", self.table);

        let mut iter = self.col_names().peekable();

        let mut cols = String::from("(");
        let mut values = String::from("VALUES(");

        while let Some(col) = iter.next() {
            if iter.peek().is_some() {
                cols += &format!("{}, ", col);
                values += &format!(":{}, ", col);
            }
            else {
                cols += &format!("{}) ", col);
                values += &format!(":{});", col);
            }
        }

        buf += &cols;
        buf += &values;

        Literal::string(&buf)
    }
}

pub fn get_table_name(ast: &DeriveInput) -> String {
    let table = ast
        .attrs
        .iter()
        .find(|attr| {
            attr.path().is_ident("table")
        });

    let Some(table) = table else {
        abort_call_site!(
            "Expected a #[table(...)] attribute.";
            note = "Exemplar does not infer the name of the SQL table your type maps to.";
            hint = r#"Specify the table like this: #[table("table_name")]."#
        )
    };

    let Ok(Lit::Str(str)) = table.parse_args::<Lit>() else {
        abort!(
            table.span(),
            "The #[table] attribute expects a single string literal as its argument.";
            hint = r#"Specify the table like this: #[table("table_name")]."#
        )
    };

    str.value()
}

pub fn get_check_path(ast: &DeriveInput) -> Option<String> {
    let table = ast
        .attrs
        .iter()
        .find(|attr| {
            attr.path().is_ident("check")
        });

    let table = table?;

    let Ok(Lit::Str(str)) = table.parse_args::<Lit>() else {
        abort!(
            table.span(),
            "The #[check] attribute expects a single string literal as its argument.";
            hint = r#"Specify the schema path like this: #[check("path/to/schema")]."#;
            hint = "The path should be specified relative to the current file."
        )
    };

    Some(
        str.value()
    )
}

pub fn get_col_name(field: &Field) -> String {
    let column = field
        .attrs
        .iter()
        .find(|attr| {
            attr.path().is_ident("column")
        });
    
    if let Some(column) = column {
        let Ok(Lit::Str(str)) = column.parse_args::<Lit>() else {
            abort!(
                column.span(),
                "The #[column] attribute expects a single string literal as its argument.";
                hint = r#"Specify the column like this: #[column("column_name")]."#
            )
        };

        return str.value();
    }

    field
        .ident
        .as_ref()
        .expect("All fields should have an identifier.")
        .to_string()
}

pub fn get_bind_path(field: &Field) -> Option<Path> {
    let bind = field
        .attrs
        .iter()
        .find(|attr| {
            attr.path().is_ident("bind")
        });

    let bind = bind?;

    let Ok(path) = bind.parse_args::<Path>() else {
        abort!(
            bind.span(),
            "The #[bind] attribute expects a single path for its argument.";
            hint = r#"Specify the bind function like this: #[bind(path::to::fn)]."#;
            hint = "Your bind function should have the signature fn (&T) -> Result<ToSqlOutput>, where T is the type of the annotated field."
        )
    };
    
    Some(path)
}

pub fn get_extr_path(field: &Field) -> Option<ExprPath> {
    let extr = field
        .attrs
        .iter()
        .find(|attr| {
            attr.path().is_ident("extr")
        });

    let extr = extr?;

    let Ok(path) = extr.parse_args::<ExprPath>() else {
        abort!(
            extr.span(),
            "The #[extr] attribute expects a single path for its argument.";
            hint = r#"Specify the extraction function like this: #[extr(path::to::fn)]."#;
            hint = "Your extraction function should have the signature fn (ValueRef) -> FromSqlResult<T>, where T is the type of the annotated field."
        )
    };
    
    Some(path)
}