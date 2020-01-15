use postgres::types as pgtypes;
use std::error::Error as StdError;
use std::fmt;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::slice;
use std::str;

use buffer::Buffer;
use connection::Connection;
use ffi;
use opaque::OpaquePtr;
use result::FFIResult;

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

    pub fn execute_with_result(&self) -> Result<QueryResult, postgres::Error> {
        let params = self.sql_params();
        let result = self.conn.query(&self.query, params.as_slice());
        result.map(|rows| QueryResult::from_rows(rows))
    }
}

pub struct QueryResult {
    pub rows: *mut ffi::Rows,
    pub iter: *mut ffi::RowsIterator,
}

impl QueryResult {
    pub fn from_rows(rows: postgres::rows::Rows) -> Self {
        let iter = OpaquePtr::new(rows.iter()).opaque();
        let rows = OpaquePtr::new(rows).opaque();
        Self { rows, iter }
    }
}
impl Drop for QueryResult {
    fn drop(&mut self) {
        let rows = OpaquePtr::from_opaque(self.rows);
        let iter = OpaquePtr::from_opaque(self.iter);
        unsafe {
            iter.free();
            rows.free();
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
pub unsafe extern "C" fn query_exec(query: *mut ffi::Query) -> FFIResult<u8> {
    let query = OpaquePtr::from_opaque(query);
    let result = query.execute();
    query.free();
    FFIResult::from_result(result)
}

#[no_mangle]
pub unsafe extern "C" fn query_exec_result(query: *mut ffi::Query) -> FFIResult<ffi::QueryResult> {
    let query = OpaquePtr::from_opaque(query);
    let result = query.execute_with_result();
    query.free();
    FFIResult::from_result(result)
}

#[no_mangle]
pub unsafe extern "C" fn result_close(result: *mut ffi::QueryResult) {
    let result = OpaquePtr::from_opaque(result);
    result.free();
}
