use std::os::raw::c_char;
use std::slice;
use std::str;

use crate::ffi;
use crate::opaque::OpaquePtr;
use crate::result::FFIResult;

pub use postgres::{Connection, TlsMode};

#[no_mangle]
pub unsafe extern "C" fn connect(dsn: *const c_char, len: usize) -> FFIResult<ffi::Connection> {
    let dsn_str = str::from_utf8_unchecked(slice::from_raw_parts(dsn as *const _, len));
    FFIResult::from_result(Connection::connect(dsn_str, TlsMode::None))
}

#[no_mangle]
pub unsafe extern "C" fn close(conn: *mut ffi::Connection) {
    let conn = OpaquePtr::from_opaque(conn);
    conn.free();
}
