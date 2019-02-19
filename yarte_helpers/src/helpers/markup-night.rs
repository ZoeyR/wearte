// https://github.com/rust-iendo/v_htmlescape/issues/26
use v_htmlescape::fallback::escape;

use std::fmt::{self, Display, Formatter};

pub struct MarkupDisplay<T>(T) where T: AsStr;

pub trait AsStr: Display {
    fn as_str(&self) -> &str;
}

pub trait Safe {}

impl<T> Display for MarkupDisplay<T> where T: AsStr {
    default fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        escape(self.0.as_str()).fmt(f)
    }
}

impl<T> Display for MarkupDisplay<T> where T: AsStr + Safe {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for MarkupDisplay<T> where T: AsStr {
    fn from(t: T) -> MarkupDisplay<T> {
        MarkupDisplay(t)
    }
}

macro_rules! impl_as_str_string {
    ($($t:ty)+) => ($(
        impl AsStr for $t {
            #[inline]
            fn as_str(&self) -> &str {
                &self
            }
        }
    )+)
}

#[rustfmt::skip]
impl_as_str_string!(String &String &&String);

macro_rules! impl_as_str_str {
    ($($t:ty)+) => ($(
        impl AsStr for $t {
            #[inline]
            fn as_str(&self) -> &str {
                self
            }
        }
    )+)
}

#[rustfmt::skip]
impl_as_str_str!(str &str &&str &&&str);

static E: &str = "";

macro_rules! impl_as_str_safe {
    ($($t:ty)+) => ($(
        impl AsStr for $t {
            fn as_str(&self) -> &str {
                E
            }
        }
        impl Safe for $t {}
    )+)
}

#[rustfmt::skip]
impl_as_str_safe!(
    bool &bool &&bool &&&bool
    usize &usize &&usize &&&usize
    u8 &u8 &&u8 &&&u8
    u16 &u16 &&u16 &&&u16
    u32 &u32 &&u32 &&&u32
    u64 &u64 &&u64 &&&u64
    isize &isize &&isize &&&isize
    i8 &i8 &&i8 &&&i8
    i16 &i16 &&i16 &&&i16
    i32 &i32 &&i32 &&&i32
    i64 &i64 &&i64 &&&i64
);
