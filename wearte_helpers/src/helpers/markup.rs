use v_htmlescape::escape;

use std::fmt::{self, Display, Formatter};

pub enum MarkupAsStr<'a> {
    UnSafe(&'a str),
    Safe(SafeTypes<'a>),
}

pub enum SafeTypes<'a> {
    Usize(&'a usize),
    U8(&'a u8),
    U16(&'a u16),
    U32(&'a u32),
    U64(&'a u64),
    Isize(&'a isize),
    I8(&'a i8),
    I16(&'a i16),
    I32(&'a i32),
    I64(&'a i64),
    Bool(&'a bool),
}

impl<'a> Display for MarkupAsStr<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::MarkupAsStr::*;
        match self {
            UnSafe(s) => escape(s).fmt(f),
            Safe(s) => match s {
                SafeTypes::Usize(n) => n.fmt(f),
                SafeTypes::U8(n) => n.fmt(f),
                SafeTypes::U16(n) => n.fmt(f),
                SafeTypes::U32(n) => n.fmt(f),
                SafeTypes::U64(n) => n.fmt(f),
                SafeTypes::Isize(n) => n.fmt(f),
                SafeTypes::I8(n) => n.fmt(f),
                SafeTypes::I16(n) => n.fmt(f),
                SafeTypes::I32(n) => n.fmt(f),
                SafeTypes::I64(n) => n.fmt(f),
                SafeTypes::Bool(n) => n.fmt(f),
            },
        }
    }
}
macro_rules! impl_from_string {
    ($($t:ty)+) => ($(
        impl<'a> From<&'a $t> for MarkupAsStr<'a> {
            #[inline]
            fn from(t: &'a $t) -> MarkupAsStr<'a> {
                MarkupAsStr::UnSafe(t.as_ref())
            }
        }
    )+)
}

#[rustfmt::skip]
impl_from_string!(String &String &&String);

macro_rules! impl_from_for {
    (Str for $($t:ty)+) => ($(
        impl<'a> From<&'a $t> for MarkupAsStr<'a> {
            #[inline]
            fn from(t: &'a $t) -> MarkupAsStr<'a> {
                MarkupAsStr::UnSafe(t)
            }
        }
    )+);
    ($p:ident for $($t:ty)+) => ($(
        impl<'a> From<&'a $t> for MarkupAsStr<'a> {
            #[inline]
            fn from(t: &'a $t) -> MarkupAsStr<'a> {
                MarkupAsStr::Safe(SafeTypes::$p(t))
            }
        }
    )+)
}

impl_from_for!(Str for str &str &&str);
impl_from_for!(Bool for bool &bool &&bool);
impl_from_for!(Usize for usize &usize &&usize);
impl_from_for!(U8 for u8 &u8 &&u8);
impl_from_for!(U16 for u16 &u16 &&u16);
impl_from_for!(U32 for u32 &u32 &&u32);
impl_from_for!(U64 for u64 &u64 &&u64);
impl_from_for!(Isize for isize &isize &&isize);
impl_from_for!(I8 for i8 &i8 &&i8);
impl_from_for!(I16 for i16 &i16 &&i16);
impl_from_for!(I32 for i32 &i32 &&i32);
impl_from_for!(I64 for i64 &i64 &&i64);
