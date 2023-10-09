use super::*;

pub fn from_row(derivee: &Derivee) -> QuoteStream {
    let field_idents = derivee.field_idents();
    let col_names    = derivee.col_names().map(|s| Literal::string(&s));

    let getters = derivee
        .fields
        .iter()
        .zip(col_names)
        .map(|(field, name)| {
            let ty = &field.ty;

            if let Some(extr) = util::get_extr_path(field) {
                quote! { #extr(row.get_ref(#name)?)? }
            }
            else {
                quote! { row.get::<_, #ty>(#name)? }
            }
        });

    quote! {
        #[inline]
        fn from_row(row: &::rusqlite::Row) -> ::rusqlite::Result<Self> {
            Ok(Self {
                #(#field_idents : #getters),*
            })
        }
    }
}

pub fn inserts(derivee: &Derivee) -> (QuoteStream, QuoteStream) {
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
                quote! { &#bind(&self.#ident)? }
            }
            else {
                quote! { &self.#ident }
            }
        });

    let abort_sql    = derivee.gen_query(None);
    let fail_sql     = derivee.gen_query(Some("FAIL"));
    let ignore_sql   = derivee.gen_query(Some("IGNORE"));
    let replace_sql  = derivee.gen_query(Some("REPLACE"));
    let rollback_sql = derivee.gen_query(Some("ROLLBACK"));
    
    let insert = quote! {
        #[inline]
        fn insert<C>(&self, conn: C) -> ::rusqlite::Result<()>
        where
            C: ::std::ops::Deref<Target = ::rusqlite::Connection>
        {
            self.insert_or(conn, ::exemplar::OnConflict::Abort)
        }
    };

    let insert_or = quote! {
        #[inline]
        fn insert_or<C>(&self, conn: C, strategy: ::exemplar::OnConflict) -> ::rusqlite::Result<()>
        where
            C: ::std::ops::Deref<Target = ::rusqlite::Connection>
        {
            use ::exemplar::OnConflict::*;
            
            let exec = |sql: &str| -> ::rusqlite::Result<()> {
                let mut stmt = conn.prepare_cached(sql)?;
                
                let params = [
                    #((#col_names, #field_idents as &dyn ::rusqlite::ToSql)),*
                ];

                stmt.execute(&params)?;

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
    };

    (insert, insert_or)
}

pub fn as_params(derivee: &Derivee) -> QuoteStream {
    let col_names = derivee
        .col_names()
        .map(|mut str| {
            str.insert(0, ':');
            Literal::string(&str)
        });

    let field_idents = derivee.field_idents();

    quote! {
        #[inline]
        fn as_params(&self) -> ::std::boxed::Box<[(&str, &dyn ::rusqlite::ToSql)]> {
            let params = [
                #((#col_names, &self.#field_idents as &dyn ::rusqlite::ToSql)),*
            ];

            ::std::boxed::Box::new(params)
        }
    }
}

pub fn check_test(derivee: &Derivee) -> QuoteStream {
    let Some(path) = &derivee.schema else {
        return QuoteStream::new()
    };

    let module = derivee.name.to_string().to_lowercase();
    let module = format_ident!("{}_exemplar_check", module);

    let table = &derivee.table;
    let columns = derivee.col_names();
    
    quote! {
        // Hack to prevent clippy::items_after_test_module from firing
        #[cfg(not(not(test)))]
        #[allow(clippy::items_after_test_module)]
        mod #module {
            use ::rusqlite::Connection;

            #[test]
            fn schema_matches() {
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
}