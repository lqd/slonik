use postgres::rows::Rows;
use postgres::types as pgtypes;
use std::error::Error as StdError;
use std::fmt;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::slice;
use std::str;

use crate::buffer::Buffer;
use crate::connection::Connection;
use crate::ffi::{self, RowMajor2DArray};
use crate::opaque::OpaquePtr;
use crate::result::FFIResult;
use crate::row::RowItem;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct QueryParam {
    pub type_name: Buffer,
    pub value: Buffer,
}

pub trait ParamType {
    const NAME: &'static str;
}

macro_rules! get_typed_param {
    ($typename: expr, $value: expr) => {{
        #[derive(Copy, Clone, Debug)]
        struct _ParamType {}
        impl ParamType for _ParamType {
            const NAME: &'static str = $typename;
        }
        Box::new(TypedQueryParam::<_ParamType>::new($value))
    }};
}

impl QueryParam {
    pub unsafe fn typed_param(&self) -> Box<dyn pgtypes::ToSql> {
        match self.type_name.to_str() {
            "text" => get_typed_param!("text", self.value),
            "int4" => get_typed_param!("int4", self.value),
            "float8" => get_typed_param!("float8", self.value),
            type_name => {
                println!("unknown type: {:?}", type_name);
                get_typed_param!("", self.value)
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TypedQueryParam<T: ParamType + fmt::Debug> {
    pub value: Buffer,
    _marker: PhantomData<T>,
}

impl<T: ParamType + fmt::Debug> TypedQueryParam<T> {
    pub fn new(value: Buffer) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }
}

impl<T: ParamType + fmt::Debug> pgtypes::ToSql for TypedQueryParam<T> {
    fn to_sql(
        &self,
        _ty: &pgtypes::Type,
        out: &mut Vec<u8>,
    ) -> Result<pgtypes::IsNull, Box<dyn StdError + Send + Sync>> {
        for i in 0..self.value.size {
            out.push(unsafe { *self.value.bytes.offset(i as isize) });
        }
        Ok(pgtypes::IsNull::No)
    }

    fn accepts(ty: &pgtypes::Type) -> bool {
        ty.name() == T::NAME
    }

    postgres::to_sql_checked!();
}

pub struct Query<'a> {
    pub conn: &'a Connection,
    pub query: String,
    pub params: Vec<Box<dyn pgtypes::ToSql>>,
}

impl<'a> Query<'a> {
    pub fn sql_params(&self) -> Vec<&dyn pgtypes::ToSql> {
        self.params.iter().map(|p| p.as_ref()).collect()
    }

    pub fn execute(&self) -> Result<u64, postgres::Error> {
        let params = self.sql_params();
        self.conn.execute(&self.query, params.as_slice())
    }

    pub fn execute_with_result<QR: QueryResult>(&self) -> Result<QR, postgres::Error> {
        let params = self.sql_params();
        let result = self.conn.query(&self.query, params.as_slice());
        result.map(|rows| QR::from_rows(rows))
    }
}

pub trait QueryResult {
    fn from_rows(rows: Rows) -> Self;
}

pub struct IteratedQueryResult {
    rows: *mut ffi::Rows,
    pub iter: *mut ffi::RowsIterator,
}

impl QueryResult for IteratedQueryResult {
    fn from_rows(rows: Rows) -> Self {
        let iter = OpaquePtr::new(rows.iter()).opaque();
        let rows = OpaquePtr::new(rows).opaque();
        Self { rows, iter }
    }
}

impl Drop for IteratedQueryResult {
    fn drop(&mut self) {
        let rows = OpaquePtr::from_opaque(self.rows);
        let iter = OpaquePtr::from_opaque(self.iter);
        unsafe {
            iter.free();
            rows.free();
        }
    }
}

pub struct EagerQueryResult {
    /// Handle to the pg rows, so that the items are valid until this query result
    /// is dropped.
    _rows: Rows,

    /// The number of rows contained in this QueryResult
    len: usize,

    /// The number of columns contained in each row
    stride: usize,

    /// The row storage container: items are stored inline in row-major format,
    /// and traversed:
    /// - row by row: the row_idx is >= 0 and < len
    /// - column by column: the col_idx is <= 0 and < stride
    items: Vec<RowItem>,
}

impl QueryResult for EagerQueryResult {
    fn from_rows(rows: Rows) -> Self {
        let len = rows.len();

        let columns = rows.columns();
        let stride = columns.len();

        // store all items inline
        let mut items = Vec::with_capacity(len * stride);
        for row in rows.iter() {
            for col_idx in 0..stride {
                let typename = columns[col_idx].type_().name();
                let item = match row.get_bytes(col_idx) {
                    Some(data) => RowItem {
                        type_name: Buffer::from_str(typename),
                        value: Buffer::from_bytes(data),
                    },
                    None => RowItem::empty(),
                };
                items.push(item);
            }
        }

        Self {
            _rows: rows,
            len,
            items,
            stride,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn new_query(
    conn: *mut ffi::Connection,
    query: *const c_char,
    len: usize,
) -> *mut ffi::Query {
    let conn = OpaquePtr::from_opaque(conn);
    let query_str = str::from_utf8_unchecked(slice::from_raw_parts(query as *const _, len));
    let q = Query {
        conn: &conn,
        query: query_str.to_string(),
        params: vec![],
    };
    OpaquePtr::new(q).opaque()
}

#[no_mangle]
pub unsafe extern "C" fn query_param(query: *mut ffi::Query, param: QueryParam) {
    let query = &mut *(query as *mut Query);
    query.params.push(param.typed_param());
}

#[no_mangle]
pub unsafe extern "C" fn query_exec(query: *mut ffi::Query) -> FFIResult<u64> {
    let query = OpaquePtr::from_opaque(query);
    let result = query.execute();
    query.free();
    FFIResult::from_result(result)
}

#[no_mangle]
pub unsafe extern "C" fn query_exec_result(
    query: *mut ffi::Query,
) -> FFIResult<ffi::IteratedQueryResult> {
    let query = OpaquePtr::from_opaque(query);
    let result: Result<IteratedQueryResult, _> = query.execute_with_result();
    query.free();
    FFIResult::from_result(result)
}

#[no_mangle]
pub unsafe extern "C" fn query_exec_result_eager(
    query: *mut ffi::Query,
) -> FFIResult<ffi::EagerQueryResult> {
    let query = OpaquePtr::from_opaque(query);
    let result: Result<EagerQueryResult, _> = query.execute_with_result();
    query.free();
    FFIResult::from_result(result)
}

/// Returns a view into the `RowItem`s of the query results.
/// Does not give ownership to the FFI of this data, which will be
/// dropped at the same time as the `QueryResult` itself, by `result_close`.
#[no_mangle]
pub extern "C" fn eager_result_get_items(
    result: *mut ffi::EagerQueryResult,
) -> RowMajor2DArray<RowItem> {
    let result = OpaquePtr::from_opaque(result);

    let ptr = result.items.as_ptr();
    let len = result.len;
    let stride = result.stride;

    RowMajor2DArray { ptr, len, stride }
}

#[no_mangle]
pub unsafe extern "C" fn result_close(result: *mut ffi::IteratedQueryResult) {
    let result = OpaquePtr::from_opaque(result);
    result.free();
}

#[no_mangle]
pub unsafe extern "C" fn eager_result_close(result: *mut ffi::EagerQueryResult) {
    let result = OpaquePtr::from_opaque(result);
    result.free();
}
