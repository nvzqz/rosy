use super::{
    prelude::*,
    RBasic,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RData {
    pub basic: RBasic,
    pub dmark: Option<unsafe extern "C" fn(*mut c_void)>,
    pub dfree: Option<unsafe extern "C" fn(*mut c_void)>,
    pub data: *mut c_void,
}
