//! Exemplar is a boilerplate eliminator for `rusqlite.`
//! 
//! # Getting Started
//! A taste of what you can do:
//! ```rust
//! use std::path::{
//!     PathBuf,
//!     Path
//! };
//! 
//! use exemplar::Model;
//! use rusqlite::Result;
//! use rusqlite::Connection;
//! 
//! #[derive(Debug, PartialEq, Model)]
//! #[table("users")]
//! #[check("../tests/schema.sql")]
//! struct User {
//!    username: String,
//!    #[bind(bind_path)]
//!    #[extr(extr_path)]
//!    home_dir: PathBuf,
//!    #[column("pwd")]
//!    password: Vec<u8>,
//! }
//! 
//! fn main() -> Result<()> {
//!     let conn = Connection::open_in_memory()?;
//! 
//!     conn.execute_batch(
//!         include_str!("../tests/schema.sql")
//!     )?;
//! 
//!     let alice = User {
//!         username: "Alice".to_owned(),
//!         home_dir: "/var/home/alice".into(),
//!         password: b"hunter2".to_vec()
//!     };
//! 
//!     let bob = User {
//!         username: "Bob".to_owned(),
//!         home_dir: "/var/home/robert".into(),
//!         password: b"password".to_vec()
//!     };
//! 
//!     alice.insert(&conn)?;
//!     bob.insert(&conn)?;
//! 
//!     let mut stmt = conn.prepare("SELECT * FROM users ORDER BY username ASC")?;
//!     let mut iter = stmt.query_and_then([], User::from_row)?;
//! 
//!     assert_eq!(alice, iter.next().unwrap()?);
//!     assert_eq!(bob, iter.next().unwrap()?);
//! 
//!     Ok(())
//! }
//! # 
//! # fn bind_path(value: &Path) -> exemplar::BindResult { 
//! #    use rusqlite::types::Value;
//! #    use rusqlite::types::ToSqlOutput;
//! #    let str = value.to_string_lossy().into_owned();
//! #    
//! #    Ok(ToSqlOutput::Owned(
//! #        Value::Text(str)
//! #    )) 
//! # }
//! # 
//! # fn extr_path(value: &rusqlite::types::ValueRef) -> exemplar::ExtrResult<PathBuf> { 
//! #   let path = value.as_str()?;
//! #   let path = PathBuf::from(path);
//! # 
//! #   Ok(path)
//! # }
//! ```
//! 
//! Exemplar is based around the [`Model`] trait, which has its own [derive macro](crate::macros::Model).
//! 
//! - See the aformentioned [macro](crate::macros::Model)'s documentation to get started.
//! - For handling `enum`s in models, check out the [`sql_enum`] macro.
//! - For working with "anonymous" record types, look at the [`record`] macro.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod macros;

use std::ops::Deref;

use rusqlite::Connection;
use rusqlite::Result;
use rusqlite::Row;
use rusqlite::Statement;
use rusqlite::ToSql;

use rusqlite::types::{
    ToSqlOutput,
    FromSqlResult
};

pub use crate::macros::*;

/// Type alias for the outcome of converting a value to an SQL-friendly representation.
/// 
/// Note that the lifetime parameter is explicitly fixed as `'static` for the benefit of [`Model::to_params`].
/// 
/// # Example
/// ```rust
/// # use std::path::Path;
/// # use exemplar::BindResult;
/// # use rusqlite::types::Value;
/// # use rusqlite::types::ToSqlOutput;
/// /// `#[bind]` function used to convert a PathBuf to an SQL-friendly representation.
/// pub fn bind_path(value: &Path) -> BindResult {   
///     let str = value.to_string_lossy().into_owned();
///     
///     Ok(ToSqlOutput::Owned(
///         Value::Text(str)
///     ))
/// }
/// ```
pub type BindResult = Result<ToSqlOutput<'static>>;

/// Type alias for the outcome of extracting a value from a [`Row`].
/// 
/// # Example
/// ```rust
/// # use std::path::PathBuf;
/// # use exemplar::ExtrResult;
/// # use rusqlite::types::ValueRef;
/// # use rusqlite::types::ToSqlOutput;
/// /// `#[extr]` function used to convert an SQL string to a PathBuf.
/// pub fn extr_path(value: &ValueRef) -> ExtrResult<PathBuf> {
///     let path = value.as_str()?;
///     let path = PathBuf::from(path);
///
///     Ok(path)
/// }
/// ```
pub type ExtrResult<T> = FromSqlResult<T>;

/// Type alias for a boxed slice of named query parameters.
pub type Parameters<'a> = Box<[(&'a str, Parameter<'a>)]>;

/// An interface for types that model SQLite database tables.
/// 
/// You should use the associated [derive macro](crate::macros::Model) to implement this trait.
/// 
/// # Object Safety
/// `Model` is mostly object safe, so you *can* have a [`dyn Model`](Model). The only caveat is that 
/// the [`from_row`](Model::from_row) method is bounded to `Self: Sized` - you can't get a concrete `Self` from a trait object.
pub trait Model {
    /// Attempt to extract an instance of `Self` from the provided [`Row`].
    /// 
    /// Best used with the [`query_and_then`](https://docs.rs/rusqlite/latest/rusqlite/struct.Statement.html#method.query_and_then) method on [`Statement`]:
    /// 
    /// ```ignore
    /// #[derive(Model)]
    /// #[table("people")]
    /// pub struct Person {
    ///     pub name: String,
    ///     pub age: u16,
    /// }
    /// 
    /// stmt.query_and_then([], Person::from_row)?
    ///     .map(|_| ...)
    /// ```
    /// 
    /// Note that this method is *not* object safe - you can't get a concrete `Self` from a [`dyn Model`](Model).
    fn from_row(row: &Row) -> Result<Self>
    where
        Self: Sized;
    
    /// Attempt to insert `self` into the database behind the provided connection.
    /// 
    /// This method is a convenience shorthand for [`Model::insert_or`] with the [`Abort`](OnConflict::Abort) conflict resolution strategy.
    /// 
    /// # Performance
    /// This method uses [`prepare_cached`](rusqlite::Connection::prepare_cached) to create the insertion SQL statement,
    /// so any calls after the first with the same connection and `self` type should be significantly faster.
    fn insert(&self, conn: &Connection) -> Result<()>;
    
    /// Attempt to insert `self` into the database behind the provided connection, using the provided [conflict resolution strategy](OnConflict).
    /// 
    /// # Performance
    /// This method uses [`prepare_cached`](rusqlite::Connection::prepare_cached) to create the insertion SQL statement,
    /// so any calls after the first with the same connection and `self` type should be significantly faster.
    fn insert_or(&self, conn: &Connection, strategy: OnConflict) -> Result<()>;
    
    /// Attempt to bind `self` to the provided statement and execute it.
    /// 
    /// This method serves two purposes:
    /// - Enabling insertions into secondary tables (such as in-memory caches.)
    /// - Squeezing out a few hundred extra nanoseconds of performance on insert operations. [`insert`](Model::insert)
    /// and [`insert_or`](Model::insert_or) use [`prepare_cached`](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.prepare_cached)
    /// to make the API convenient, but this incurs a map lookup on every call. [`bind_to`](Model::bind_to) can therefore
    /// help you squeeze out a bit more speed if your program is extremely write-heavy.
    /// 
    /// # Usage
    /// Exemplar binds fields to statements as *named parameters.* Take this example model type:
    /// 
    /// ```rust
    /// # use exemplar::*;
    /// #[derive(Model)]
    /// #[table("foos")]
    /// pub struct Foo {
    ///     pub bar: String,
    ///     pub baz: String,
    /// }
    /// ```
    /// 
    /// Exemplar will bind `bar` to `:bar` and `baz` to `:baz`, which can be used in a query like this:
    /// 
    /// ```sql
    /// INSERT INTO foos (bar, baz) 
    /// VALUES(:bar, :baz);
    /// ```
    /// 
    /// Or, in Rust:
    /// 
    /// ```rust
    /// # use exemplar::*;
    /// # use rusqlite::*;
    /// # #[derive(Model)]
    /// # #[table("foos")]
    /// # struct Foo { bar: String, baz: String }
    /// # fn main() -> Result<()> {
    /// let conn = Connection::open_in_memory()?;
    /// 
    /// conn.execute("CREATE TABLE foos (bar, baz);", [])?;
    /// 
    /// let mut stmt = conn.prepare("
    ///     INSERT INTO foos (bar, baz) 
    ///     VALUES(:bar, :baz);
    /// ")?;
    /// 
    /// let foo = Foo {
    ///     bar: "my_bar".to_string(),
    ///     baz: "my_baz".to_string(),
    /// };
    /// 
    /// foo.bind_to(&mut stmt)?;
    /// # Ok(())
    /// # }
    /// ```
    fn bind_to(&self, stmt: &mut Statement) -> Result<()>;
    
    /// Generate a slice of named [`Parameters`] from an instance of the implementing type.
    ///  
    /// # Performance
    /// This method allocates at least once, in order to [`Box`] the returned slice.
    /// 
    /// If the implementing type has any fields annotated with `#[bind]`, an additional boxing will be incurred for each annotated field.
    fn to_params(&self) -> Result<Parameters>;

    /// Static dispatch version of [`Model::metadata_dyn`].
    fn metadata() -> ModelMeta
    where
        Self: Sized;

    /// Retrieve [`ModelMeta`] (model metadata) associated with the implementing type.
    /// 
    /// This is the dynamic dispatch version of [`Model::metadata`].
    /// If (for whatever reason) you find yourself needing to dynamically reflect on [`Model`] properties, then this is for you.
    /// 
    /// # Performance
    /// Despite the name of this method, [`ModelMeta`] consists solely of `'static` data generated at compile time, making it trivially copyable.
    /// 
    /// The only overhead on this call is therefore dynamic dispatch and several shallow copies.
    fn metadata_dyn(&self) -> ModelMeta;
}

/// Possible conflict resolution strategies when using [`Model::insert_or`].
/// 
/// The default setting (used by [`Model::insert`]) is [`Abort`](OnConflict::Abort).
/// 
/// Sourced from the [SQLite docs](https://www.sqlite.org/lang_conflict.html).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OnConflict {
    /// When an applicable constraint violation occurs, error and revert any changes made by the current SQL statement.
    /// Prior SQL statements in the same transaction are unaffected, and the transaction remains active.
    /// 
    /// This is the default behavior.
    Abort,
    /// When an applicable constraint violation occurs, error but do *not* revert any changes made by the current SQL statement.
    /// Prior SQL statements in the same transaction are unaffected, and the transaction remains active.
    Fail,
    /// When an applicable constraint violation occurs, skip the row that contains the constraint violation and continue processing
    /// the current SQL statement. Other rows, prior statements and the current transaction are unaffected.
    /// 
    /// When encountering a foreign key constraint error, this behaves like [`Abort`](Self::Abort).
    Ignore,
    /// - When a uniqueness or primary key constraint violation occurs, delete the offending rows and continue.
    /// - If a `NOT NULL` constraint violation occurs, the offending column will either be replaced with the default value (if one exists) or [`Abort`](Self::Abort) behavior will be used.
    /// - If a `CHECK` or foreign key constraint violation occurs, [`Abort`](Self::Abort) behavior will be used.
    Replace,
    /// When an applicable constraint violation occurs, abort the current SQL statement and roll back the current transaction.
    /// If no transaction is active, [`Abort`](Self::Abort) behavior will be used.
    Rollback,
}

impl Default for OnConflict {
    fn default() -> Self {
        Self::Abort
    }
}

/// [`Cow`](std::borrow::Cow)-like type for query parameters.
/// 
/// Necessary to efficiently implement [`Model::to_params`] - while most fields can be directly referenced as
/// a [`dyn ToSql`](ToSql), those with `#[bind]` and `#[extr]` parameters require a non-trivial conversion step.
/// 
/// [`Self::Borrowed`] is used for the former case, while [`Self::Boxed`] is used for the latter case. 
/// 
/// This type implements [`ToSql`] and dereferences to `dyn ToSql`, so it can be used just like any other query parameter.
pub enum Parameter<'a> {
    /// A borrowed [`ToSql`] trait object.
    Borrowed(&'a dyn ToSql),
    /// A boxed [`ToSql`] trait object.
    Boxed(Box<dyn ToSql + 'a>)
}

impl<'a> ToSql for Parameter<'a> {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        match self {
            Self::Borrowed(param) => param.to_sql(),
            Self::Boxed(param) => param.to_sql(),
        }
    }
}

impl<'a> Deref for Parameter<'a> {
    type Target = dyn ToSql + 'a;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(param) => *param,
            Self::Boxed(param) => param,
        }
    }
}

/// Metadata about a [`Model`] implementor.
/// 
/// Can be retrieved via the [`Model::metadata`] and [`Model::metadata_dyn`] methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelMeta {
    /// The name of the model type.
    /// 
    /// # Example
    /// ```rust
    /// # use exemplar::*;
    /// #[derive(Model)]
    /// #[table("foos")]
    /// pub struct Foo { 
    ///     pub field: String 
    /// }
    /// 
    /// assert_eq!("Foo", Foo::metadata().model)
    /// ```
    pub model: &'static str,
    /// The name of the model table.
    /// 
    /// # Example
    /// ```rust
    /// # use exemplar::*;
    /// #[derive(Model)]
    /// #[table("foos")]
    /// pub struct Foo { 
    ///     pub field: String 
    /// }
    /// 
    /// assert_eq!("foos", Foo::metadata().table)
    /// ```
    pub table: &'static str,
    /// The field names of the model type, in order of their definition.
    /// 
    /// # Example
    /// ```rust
    /// # use exemplar::*;
    /// #[derive(Model)]
    /// #[table("foos")]
    /// pub struct Foo { 
    ///     pub bar: String,
    ///     pub qux: String,
    /// }
    /// 
    /// assert_eq!(&["bar", "qux"], Foo::metadata().fields)
    /// ```
    pub fields: &'static [&'static str],
    /// The columns of the model table.
    /// 
    /// # Example
    /// ```rust
    /// # use exemplar::*;
    /// #[derive(Model)]
    /// #[table("foos")]
    /// pub struct Foo { 
    ///     pub bar: String,
    ///     #[column("baz")]
    ///     pub qux: String,
    /// }
    /// 
    /// assert_eq!(&["bar", "baz"], Foo::metadata().columns)
    /// ```
    pub columns: &'static [&'static str],
}