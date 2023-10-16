use super::*;

pub fn from_row(derivee: &Derivee) -> QuoteStream {
    let field_idents = derivee.field_idents();
    let col_names    = derivee.col_names().map(|s| Literal::string(&s));

    let getters = derivee
        .fields
        .iter()
        .zip(col_names)
        // Handle #[extr]/no #[extr]
        .map(|(field, name)| {
            let ty = &field.ty;

            if let Some(extr) = util::get_extr_path(field) {
                quote! { #extr(&row.get_ref(#name)?)? }
            }
            else {
                quote! { row.get::<_, #ty>(#name)? }
            }
        });

    quote! {
        #[inline]
        fn from_row(row: &::rusqlite::Row) -> ::rusqlite::Result<Self> 
        where
            Self: ::std::marker::Sized,
        {
            Ok(Self {
                #(#field_idents : #getters),*
            })
        }
    }
}

pub fn inserts(derivee: &Derivee) -> QuoteStream {
    let col_names: Vec<_> = derivee
        .col_names()
        .map(|mut str| {
            str.insert(0, ':');
            Literal::string(&str)
        })
        .collect();
    
    let field_idents: Vec<_> = derivee
        .field_idents()
        .zip(&derivee.fields)
        // Handle #[bind]/no #[bind]
        .map(|(ident, field)| {
            if let Some(bind) = util::get_bind_path(field) {
                quote! { &#bind(&self.#ident)? }
            }
            else {
                quote! { &self.#ident }
            }
        })
        .collect();

    let abort_sql    = derivee.gen_query(None);
    let fail_sql     = derivee.gen_query(Some("FAIL"));
    let ignore_sql   = derivee.gen_query(Some("IGNORE"));
    let replace_sql  = derivee.gen_query(Some("REPLACE"));
    let rollback_sql = derivee.gen_query(Some("ROLLBACK"));

    quote! {
        #[inline]
        fn insert(&self, conn: &::rusqlite::Connection) -> ::rusqlite::Result<()> {
            self.insert_or(conn, ::exemplar::OnConflict::Abort)
        }

        #[inline]
        fn insert_or(&self, conn: &::rusqlite::Connection, strategy: ::exemplar::OnConflict) -> ::rusqlite::Result<()> {
            use ::exemplar::OnConflict::*;
            
            let exec = |sql: &str| -> ::rusqlite::Result<()> {
                let mut stmt = conn.prepare_cached(sql)?;

                stmt.execute(rusqlite::named_params! {
                    #(#col_names: #field_idents),*
                })?;

                Ok(())
            };
            
            match strategy {
                Abort => exec(#abort_sql),
                Fail => exec(#fail_sql),
                Ignore => exec(#ignore_sql),
                Replace => exec(#replace_sql),
                Rollback => exec(#rollback_sql),
            }
        }

        #[inline]
        fn bind_to(&self, stmt: &mut::rusqlite::Statement) -> ::rusqlite::Result<()> {
            stmt.execute(rusqlite::named_params! {
                #(#col_names: #field_idents),*
            })?;

            Ok(())
        }
    }
}

pub fn to_params(derivee: &Derivee) -> QuoteStream {
    let col_names = derivee
        .col_names()
        .map(|mut str| {
            str.insert(0, ':');
            Literal::string(&str)
        });

    let field_idents = derivee
        .field_idents()
        .zip(&derivee.fields)
        .map(|(ident, field)| {
            if let Some(bind) = util::get_bind_path(field) {
                // If the field has a #[bind] attribute, then we execute it now and box the result.
                quote! { Boxed(Box::new(#bind(&self.#ident)?) as Box<dyn ::rusqlite::ToSql>) }
            }
            else {
                // Otherwise, we're good to just borrow directly from self and cast to a dyn ToSql.
                quote! { Borrowed(&self.#ident as &dyn ::rusqlite::ToSql) }
            }
        });
    
    quote! {
        #[inline]
        fn to_params(&self) -> ::rusqlite::Result<::exemplar::Parameters> {
            use ::std::boxed::Box;
            use ::exemplar::Parameter::*;

            let params = [
                #((#col_names, #field_idents)),*
            ];

            Ok(
                Box::new(params)
            )
        }
    }
}

pub fn metadata(derivee: &Derivee) -> QuoteStream {
    let model = &derivee.name;
    let table = &derivee.table;
    
    let field_names = derivee
        .fields
        .iter()
        .map(|field| {
            field
                .ident
                .as_ref()
                .expect("All fields should have an identifier.")
                .to_string()
        });
    
    let columns = derivee.col_names();
    
    quote! {
        #[inline]
        fn metadata_dyn(&self) -> ::exemplar::ModelMeta {
            Self::metadata()
        }
        
        #[inline]
        fn metadata() -> ::exemplar::ModelMeta
        where
            Self: ::std::marker::Sized
        {
            use ::exemplar::ModelMeta;

            static FIELDS: &'static [&'static str] = &[
                #(#field_names),*
            ];

            static COLUMNS: &'static [&'static str] = &[
                #(#columns),*
            ];

            ModelMeta {
                model: stringify!(#model),
                table: #table,
                fields: FIELDS,
                columns: COLUMNS,
            }
        }
    }
}

pub fn check_test(derivee: &Derivee) -> QuoteStream {
    let Some(path) = &derivee.schema else {
        return QuoteStream::new()
    };

    let func = derivee.name.to_string().to_lowercase();
    let func = format_ident!("{}_exemplar_check", func);

    let table = &derivee.table;
    let columns = derivee.col_names();
    
    quote! {
        // Hack to prevent clippy::items_after_test_module from firing
        #[cfg(not(not(test)))]
        #[automatically_derived]
        #[test]
        fn #func() {
            use ::rusqlite::Connection;

            let schema = include_str!(#path);

            let conn = Connection::open_in_memory()
                .expect("In-memory DB connection should open successfully.");

            conn.execute_batch(schema)
                .expect("Failed to apply provided schema to check DB.");

            let mut names = String::new();

            conn.pragma(None, "table_info", #table, |row| {
                let name = row.get::<_, String>("name")
                    .expect("Failed to get name for table row.");

                names += &name;
                names += "\n";

                Ok(())
            }).expect("Failed to query table_info pragma.");

            #(assert!(names.contains(#columns)));*
        }
    }
}