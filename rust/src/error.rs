use buffer::Buffer;
use ffi;
use opaque::OpaquePtr;

pub struct Error {
    pub code: u8,
    pub msg: String,
}

#[no_mangle]
pub unsafe extern "C" fn error_msg(error: *mut ffi::Error) -> Buffer {
    let error = OpaquePtr::from_opaque(error);
    Buffer::from_str(&error.msg)
}

#[no_mangle]
pub unsafe extern "C" fn error_free(error: *mut ffi::Error) {
    let error = OpaquePtr::from_opaque(error);
    error.free();
}
