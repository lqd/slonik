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

pub struct IteratedQueryResult;
impl OpaqueTarget<'_> for IteratedQueryResult {
    type Target = query::IteratedQueryResult;
}

pub struct EagerQueryResult;
impl OpaqueTarget<'_> for EagerQueryResult {
    type Target = query::EagerQueryResult;
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

/// An immutable 2D array stored in a row-major 1D array:
/// `len` rows, each of containing `stride` elements.
#[repr(C)]
pub struct RowMajor2DArray<T> {
    /// Pointer to the first element of the array
    pub ptr: *const T,

    /// Number of rows in the array
    pub len: usize,

    /// Number of elements per row
    pub stride: usize,
}
