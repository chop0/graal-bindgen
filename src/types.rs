pub mod ctypes {
    use crate::ruesti::Value;

    pub type c_void = Value;
    pub type c_char = i8;
    pub type c_schar = i8;
    pub type c_uchar = u8;
    pub type c_short = i16;
    pub type c_ushort = u16;
    pub type c_int = i32;
    pub type c_uint = u32;
    pub type c_long = i64;
    pub type c_ulong = u64;
    pub type c_longlong = i64;
    pub type c_ulonglong = u64;
    pub type c_float = f32;
    pub type c_double = f64;
}

pub mod jtypes {
    pub type byte = i8;
    pub type short = i16;
    pub type int = i32;
    pub type long = i64;
    pub type float = f32;
    pub type double = f64;
    pub type boolean = bool;
}
