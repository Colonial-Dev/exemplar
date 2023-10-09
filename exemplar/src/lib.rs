use std::ops::Deref;

use rusqlite::Connection;
use rusqlite::Result;
use rusqlite::Row;
use rusqlite::ToSql;

use rusqlite::types::{
    ToSqlOutput,
    FromSqlResult
};

/// Derive macro for implementing the [`Model`] trait.
pub use exemplar_derive::Model;

pub type BindResult<'a> = Result<ToSqlOutput<'a>>;
pub type ExtrResult<T> = FromSqlResult<T>;

pub trait Model : Sized {
    fn from_row(row: &Row) -> Result<Self>;
    
    fn insert<C>(&self, conn: C) -> Result<()>
    where
        C: Deref<Target = Connection>;

    fn insert_or<C>(&self, conn: C, strategy: OnConflict) -> Result<()>
    where
        C: Deref<Target = Connection>;
    
    fn as_params(&self) -> Box<[(&str, &dyn ToSql)]>;
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