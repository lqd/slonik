//! FFI-related opaque pointer types

use connection;
use error;
use opaque::OpaqueTarget;
use query;

pub struct Connection;
impl OpaqueTarget<'_> for Connection {
    type Target = connection::Connection;
}

pub struct Error;
impl OpaqueTarget<'_> for Error {
    type Target = error::Error;
}

pub struct Query;
impl<'a> OpaqueTarget<'a> for Query {
    type Target = query::Query<'a>;
}

pub struct QueryResult;
impl OpaqueTarget<'_> for QueryResult {
    type Target = query::QueryResult;
}

pub struct Rows;
impl OpaqueTarget<'_> for Rows {
    type Target = postgres::rows::Rows;
}

pub struct RowsIterator;
impl<'a> OpaqueTarget<'a> for RowsIterator {
    type Target = postgres::rows::Iter<'a>;
}

pub struct Row;
impl<'a> OpaqueTarget<'a> for Row {
    type Target = postgres::rows::Row<'a>;
}
