/// Derive macro for the [`Model`] trait.
/// 
/// # Requirements
/// 
/// [`Model`](crate::Model) can *only* be derived for non-generic `struct`s with suitable named fields.
/// 
/// | Type | Example | Supported? |
/// | ---- | ------- | ---------- |
/// | Standard `struct` | `struct Person { name: String }` | ✔ |
/// | Generic `struct`  | `struct Person<'a, T, const N: usize> { ... }` | ✘ |
/// | Tuple struct | `struct Point(i64, i64)` | ✘ | 
/// | Unit/ZST struct | `struct Unit;` or `struct Unit {}` | ✘ |
/// | `enum`s | `enum Direction { Up, Down }` | ✘ |
/// | `union`s | `union Number { i: i32, f: f32 }` | ✘ |
/// 
/// (Note, however, that any non-supported type can be *used* in a [`Model`](crate::Model) assuming it meets the below requirements.)
/// 
/// All fields in a [`Model`](crate::Model) derivee must either:
/// - Implement [`ToSql`](https://docs.rs/rusqlite/latest/rusqlite/types/trait.ToSql.html) and [`FromSql`](https://docs.rs/rusqlite/latest/rusqlite/types/trait.FromSql.html). Most common types will meet this requirement.
/// - Have `#[bind]` and `#[extr]` [attributes](Model#attributes) on fields that do not meet the first requirement. 
///   - This escape hatch is designed to enable compatibility with certain `std` types like [`PathBuf`](std::path::PathBuf) and third-party crate types.
/// 
/// # Usage
/// Most of the time, deriving [`Model`](crate::Model) is easy. The only thing you need to specify is the table name:
/// ```rust
/// # use exemplar::Model;
/// #[derive(Model)]
/// #[table("people")] // <-- Required
/// pub struct Person {
///     pub name: String,
///     pub age: u16,
/// }
/// ```
///
/// For more complicated types and schemas, you may need to make use of some of the [attributes](Model#attributes) recognized by the macro:
/// ```rust
/// # use std::path::{PathBuf, Path};
/// # use exemplar::Model;
/// #[derive(Model)]
/// #[table("users")]
/// #[check("schema.sql")]
/// struct User {
///    username: String,
///    #[bind(bind_path)]
///    #[extr(extr_path)]
///    home_dir: PathBuf,
///    #[column("pwd")]
///    password: Vec<u8>,
/// }
/// # fn bind_path(value: &Path) -> exemplar::BindResult { panic!() }
/// # fn extr_path(value: &rusqlite::types::ValueRef) -> exemplar::ExtrResult<PathBuf> { panic!() }
/// ```
/// 
/// # Attributes
/// The [`Model`](crate::Model) derive macro recognizes several attributes.
/// 
/// ### `#[check]`
/// Usage:
/// ```ignore
/// #[check("path_to_schema")]
/// pub struct T { ... }
/// ```
/// 
/// The `check` attribute automatically generates a test that checks the derived [`Model`](crate::Model) implementation against a provided schema.
/// 
/// More specifically, the generated test verifies that:
/// - The specified table exists.
/// - The columns in the schema match up with those specified by the model type. If the model has fields not present in the schema (or vice versa) the test will fail.
/// 
/// It does *not* verify the validity of column types, nor does it test actual insertion/retrieval.
/// 
/// ### `#[bind]`/`#[extr]`
/// Usage:
/// ```ignore
/// #[bind(path::to::fn)]
/// #[extr(path::to::fn)]
/// field: T,
/// ```
/// 
/// The `bind` and `extr` attributes specify functions used to convert the annotated field to and from an SQL-friendly representation.
/// This is primarily intended as an escape hatch for when you can't implement [`ToSql`](https://docs.rs/rusqlite/latest/rusqlite/types/trait.ToSql.html) and [`FromSql`](https://docs.rs/rusqlite/latest/rusqlite/types/trait.FromSql.html)
/// yourself.
/// 
/// Both attributes take as their argument a path to a free function used to do the conversion.
/// - For `bind`, the signature should be [`fn(&T) -> BindResult`](crate::BindResult).
/// - For `extr`, the signature should be [`fn(&ValueRef) -> ExtrResult<T>`](crate::ExtrResult). 
/// 
/// In both cases `T` is the type of the field being annotated. For some types (e.g. `PathBuf`) you may also be able to use a type it derefs to, like `Path`.
/// 
/// Example implementations for `PathBuf`:
/// ```rust
/// # use exemplar::*;
/// # use std::path::{Path, PathBuf};
/// # use rusqlite::types::ValueRef;
/// pub fn bind_path(value: &Path) -> BindResult {
///     use rusqlite::types::Value;
///     use rusqlite::types::ToSqlOutput;
///     
///     // Depending on your program, it may make more sense
///     // to error if a lossless conversion isn't possible.
///     let str = value.to_string_lossy().into_owned();
/// 
///     Ok(ToSqlOutput::Owned(
///         Value::Text(str)
///     ))
/// }
/// 
/// pub fn extr_path(value: &ValueRef) -> ExtrResult<PathBuf> {
///     let path = value.as_str()?;
///     let path = PathBuf::from(path);
/// 
///     Ok(path)
/// }
/// ```
/// 
/// ### `#[column]`
/// Usage:
/// ```ignore
/// #[column("name")]
/// field: T,
/// ```
/// 
/// The `column` attribute overrides the column name Exemplar maps the annotated field to. By default, the field name is assumed to directly map to the underlying schema - `#[column]` is how you alter this behavior.
/// 
/// # Notes
/// Any type that derives [`Model`](crate::Model) also has an implementation of [`TryFrom<Row>`] derived, making models usable in some generic contexts.
pub use exemplar_proc_macro::Model;

/// Generate an "anonymous" record `struct` that implements `from_row`.
/// 
/// This is best used for deserializing rows from an ad-hoc query in a strongly typed manner. 
/// 
/// Note that the generated struct does *not* implement [`Model`](crate::Model), as it can't be associated with any specific table.
/// This means that tools like `#[bind]`/`#[extr]` and the like are not available for records.
/// 
/// However, `TryFrom<Row>` is still implemented, making records usable in some generic contexts.
/// 
/// # Example
/// The example assumes this database schema:
/// ```sql
/// CREATE TABLE people (name, age, alive); 
/// ```
/// 
/// ```ignore
/// record! {
///     // The provided field name is assumed to map directly to a column in a query's output.
///     name => String,
///     age  => u16,
/// }
/// 
/// record! {
///     // By default, the generated struct is called `Record.`
///     // This can be overridden with the `Name` parameter, should the need arise.
///     Name => Age,
///     age  => u16,
/// }
/// 
/// let mut get_people = conn.prepare("SELECT name, age FROM people")?;
/// let mut get_ages = conn.prepare("SELECT age FROM people")?;
/// 
/// get_people
///     .query_and_then([], Record::from_row)?
///     .map(|record| ...);
/// 
/// get_ages
///     .query_and_then([], Age::from_row)?
///     .map(|age| ...);
/// ```
/// 
/// # Notes
/// 
/// Doc comments (and other attributes) are supported:
/// ```rust
/// # use exemplar::*;
/// record! {
///     /// A person's name.
///     name => String,
///     /// A person's age.
///     age  => u16,
/// }
/// ```
/// Additionally, you can apply type-level attributes like derives on the `Name` argument.
/// ```rust
/// # use exemplar::*;
/// record! {
///     #[derive(Debug, Clone)]
///     Name => Age,
///     age  => u16,
/// }
/// ```
/// (`record!` does not apply any derives automatically.)
/// 
/// This does *not* work without the `Name` argument, due to macro limitations - Rust can't
/// disambiguate between "attributes for the struct" and "attributes for the field."
/// 
/// ```compile_fail
/// # use exemplar::*;
/// record! {
///     #[derive(Debug, Clone)]
///     /// A person's name.
///     name => String,
///     /// A person's age.
///     age  => u16,
/// }
/// ```
#[macro_export]
macro_rules! record {
    ($(#[$struct_doc:meta])* Name => $name:ident, $($(#[$field_doc:meta])* $fname:ident => $ftype:ty),* $(,)?) => {
        $(#[$struct_doc])*
        /// 
        /// Automatically generated record type for storing query results.
        pub struct $name {
            $($(#[$field_doc])* pub $fname : $ftype),*
        }
        
        impl $name {
            fn from_row(row: &::rusqlite::Row) -> ::rusqlite::Result<Self> {
                Ok(Self {
                    $($fname : row.get(stringify!($fname))?),*
                })
            }
        }

        impl<'a> ::std::convert::TryFrom<&'a ::rusqlite::Row<'_>> for $name {
            type Error = ::rusqlite::Error;

            fn try_from(value: &'a ::rusqlite::Row) -> Result<Self, Self::Error> {
                Self::from_row(value)
            }
        }
    };
    ($($(#[$field_doc:meta])* $fname:ident => $ftype:ty),* $(,)?) => {
        record!(Name => Record, $($(#[$field_doc])* $fname => $ftype),*);
    };
}

/// Generate an SQL-compatible field-less `enum`.
/// 
/// SQL compatible means:
/// - `#[repr(i*/u*)]`
/// - Implements [`TryFrom<i64>`].
/// - Implements [`ToSql`](https://docs.rs/rusqlite/latest/rusqlite/types/trait.ToSql.html) and [`FromSql`](https://docs.rs/rusqlite/latest/rusqlite/types/trait.FromSql.html).
/// 
/// (Additionally, the standard constellation of `Debug`/`Clone`/`Copy`/`Eq`/`Hash` are derived.)
/// 
/// An enum generated by this macro can be used in any [`Model`](crate::Model) implementor.
/// 
/// # Usage
/// ```rust
/// # use exemplar::sql_enum;
/// sql_enum! {
///     Name => Color,
///     Red,
///     Green,
///     Blue,
/// };
/// ```
/// 
/// By default, `#[repr(i64)]` is used. The optional `Type` parameter can override this:
/// 
/// ```rust
/// # use exemplar::sql_enum;
/// sql_enum! {
///     Name => Color,
///     Type => u8,
///     Red,
///     Green,
///     Blue,
/// };
/// ```
/// 
/// # Notes
/// Explicit discriminants are *not* supported. Variants will always be implicitly numbered, in order of definition, from zero. 
/// 
/// Concretely, this means that:
/// ```compile_fail
/// # use exemplar::*;
/// sql_enum! {
///     Name => Color,
///     Red = 1,
///     Green = 2,
///     Blue = 3
/// }
/// ```
/// ...will not compile.
/// 
/// <hr>
/// 
/// Doc comments (and other attributes, like derives) *are* supported:
/// ```rust
/// # use exemplar::sql_enum;
/// sql_enum! {
///     #[derive(Default)]
///     /// An RGB color tag.
///     Name => Color,
///     /// Red
///     #[default]
///     Red,
///     /// Green
///     Green,
///     /// Blue
///     Blue,
/// }
/// ```
#[macro_export]
macro_rules! sql_enum {
    ($(#[$enum_doc:meta])* Name => $name:ident, Type => $disc:ty, $($(#[$variant_doc:meta])* $vname:ident),* $(,)?) => {
        $(#[$enum_doc])*
        #[repr($disc)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($(#[$variant_doc])* $vname),*
        }

        #[automatically_derived]
        impl ::rusqlite::ToSql for $name {
            fn to_sql(&self) -> ::rusqlite::Result<::rusqlite::types::ToSqlOutput<'_>> {
                let value = ::rusqlite::types::Value::Integer(*self as i64);
                let value = ::rusqlite::types::ToSqlOutput::Owned(value);
                Ok(value)
            }
        }

        #[automatically_derived]
        impl ::rusqlite::types::FromSql for $name {
            fn column_result(value: ::rusqlite::types::ValueRef<'_>) -> ::rusqlite::types::FromSqlResult<Self> {
                value.as_i64()
                    .map(<$name>::try_from)?
                    .map_err(|err| {
                        ::rusqlite::types::FromSqlError::Other(Box::new(err))
                    })
            }
        }

        #[automatically_derived]
        impl ::std::convert::TryFrom<i64> for $name {
            type Error = ::rusqlite::types::FromSqlError;

            fn try_from(value: i64) -> Result<Self, Self::Error> {
                match value {
                    $(x if x == Self::$vname as i64 => Ok(Self::$vname),)*
                    _ => {
                        let msg = format!(
                            "No discriminant in enum `{}` matches the value `{value}`",
                            stringify!($name)
                        );
    
                        Err(::rusqlite::types::FromSqlError::Other(
                            msg.into()
                        ))
                    }
                }
            }
        }
    };
    ($(#[$enum_doc:meta])* Name => $name:ident, $($(#[$variant_doc:meta])* $vname:ident),* $(,)?) => {
        sql_enum!($(#[$enum_doc])* Name => $name, Type => i64, $($(#[$variant_doc])* $vname),*);
    }
}