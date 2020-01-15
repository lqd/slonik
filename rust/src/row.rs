use buffer::Buffer;
use ffi;
use opaque::OpaquePtr;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RowItem {
    pub type_name: Buffer,
    pub value: Buffer,
}

impl RowItem {
    pub fn empty() -> Self {
        let buff = Buffer::null();
        Self {
            type_name: buff,
            value: buff,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn next_row(result: *mut ffi::QueryResult) -> *const ffi::Row {
    let result = OpaquePtr::from_opaque(result);
    let mut iter = OpaquePtr::from_opaque(result.iter);
    match iter.next() {
        Some(x) => OpaquePtr::new(x).opaque(),
        None => std::ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn row_len(row: *mut ffi::Row) -> usize {
    let row = OpaquePtr::from_opaque(row);
    row.len()
}

#[no_mangle]
pub unsafe extern "C" fn row_close(row: *mut ffi::Row) {
    let row = OpaquePtr::from_opaque(row);
    row.free();
}

#[no_mangle]
pub unsafe extern "C" fn row_item(row: *mut ffi::Row, i: usize) -> RowItem {
    let row = OpaquePtr::from_opaque(row);
    let type_name = row.columns()[i].type_().name();

    match row.get_bytes(i) {
        Some(data) => RowItem {
            type_name: Buffer::from_str(type_name),
            value: Buffer::from_bytes(data),
        },
        None => RowItem::empty(),
    }
}
