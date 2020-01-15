use buffer::Buffer;
use opaque::{OpaquePtr, OpaqueTarget};

pub struct Error {
    pub code: u8,
    pub msg: String,
}

pub struct _Error;
impl OpaqueTarget<'_> for _Error {
    type Target = Error;
}

#[no_mangle]
pub unsafe extern "C" fn error_msg(error: *mut _Error) -> Buffer {
    let error = OpaquePtr::from_opaque(error);
    Buffer::from_str(&error.msg)
}

#[no_mangle]
pub unsafe extern "C" fn error_free(error: *mut _Error) {
    let error = OpaquePtr::from_opaque(error);
    error.free();
}
