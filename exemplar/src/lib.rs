use std::ops::Deref;

use rusqlite::Connection;
use rusqlite::Result;
use rusqlite::Row;
use rusqlite::ToSql;

use rusqlite::types::{
    ToSqlOutput,
    FromSqlResult
};

/// Derive macro for the [`Model`] trait.
pub use exemplar_proc_macro::Model;

pub type BindResult = Result<ToSqlOutput<'static>>;
pub type ExtrResult<T> = FromSqlResult<T>;
pub type Parameters<'a> = Box<[(&'a str, Parameter<'a>)]>;

pub trait Model : Sized {
    fn from_row(row: &Row) -> Result<Self>;
    
    fn insert<C>(&self, conn: C) -> Result<()>
    where
        C: Deref<Target = Connection>;

    fn insert_or<C>(&self, conn: C, strategy: OnConflict) -> Result<()>
    where
        C: Deref<Target = Connection>;
    
    fn as_params(&self) -> Result<Parameters>;
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OnConflict {
    Abort,
    Fail,
    Ignore,
    Replace,
    Rollback,
}

pub enum Parameter<'a> {
    Borrowed(&'a dyn ToSql),
    Boxed(Box<dyn ToSql>)
}

impl<'a> ToSql for Parameter<'a> {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        match self {
            Self::Borrowed(param) => param.to_sql(),
            Self::Boxed(param) => param.to_sql(),
        }
    }
}