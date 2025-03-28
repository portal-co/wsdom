use crate::{
    js::nullable::{JsNullable, JsNullish},
    js_cast::JsCast,
    serialize::{RawCodeImmediate, ToJs, UseInJsCode},
};

pub struct NullImmediate;

impl UseInJsCode for NullImmediate {
    fn serialize_to(&self, buf: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        RawCodeImmediate("null").serialize_to(buf)
    }
}

impl ToJs<JsNullish> for NullImmediate {}
impl<T> ToJs<JsNullable<T>> for NullImmediate {}

pub struct UndefinedImmediate;
impl UseInJsCode for UndefinedImmediate {
    fn serialize_to(&self, buf: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        RawCodeImmediate("undefined").serialize_to(buf)
    }
}

impl ToJs<JsNullish> for UndefinedImmediate {}
impl<T> ToJs<JsNullable<T>> for UndefinedImmediate {}

/// The return value implements `ToJs<JsNullish>` and `ToJs<JsNullable<T>>`.
pub const fn null() -> NullImmediate {
    NullImmediate
}

/// The return value implements `ToJs<JsNullish>` and `ToJs<JsNullable<T>>`.
pub const fn undefined() -> UndefinedImmediate {
    UndefinedImmediate
}

impl<'a, T: UseInJsCode> UseInJsCode for Option<&'a T> {
    fn serialize_to(&self, buf: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Some(t) => t.serialize_to(buf),
            None => NullImmediate.serialize_to(buf),
        }
    }
}

impl<'a, T, U> ToJs<JsNullable<T>> for Option<&'a U>
where
    T: JsCast,
    U: UseInJsCode + ToJs<T>,
{
}
