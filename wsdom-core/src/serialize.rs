use serde::Serialize;

use crate::js::value::JsValue;
use crate::protocol::GET;

/// For values that can be serialized to JS code:
/// - Rust values that implement `serde::Serialize`
/// - WRMI stubs ([JsValue]s)
///
/// This trait is used by [ToJs].
pub trait UseInJsCode {
    fn serialize_to(&self, buf: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;
}

impl UseInJsCode for JsValue {
    fn serialize_to(&self, buf: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let self_id = self.id;
        write!(buf, "{GET}({self_id})").unwrap();
        Ok(())
    }
}

pub struct SerdeToJs<'a, T: ?Sized>(pub &'a T);

impl<'a, T: Serialize + ?Sized> UseInJsCode for SerdeToJs<'a, T> {
    fn serialize_to(&self, buf: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        buf.write_str(&serde_json::to_string(&self.0).map_err(|_|core::fmt::Error)?)
    }
}

pub(crate) struct UseInJsCodeWriter<'a, T: UseInJsCode + ?Sized>(pub &'a T);

impl<'a, T: UseInJsCode + ?Sized> core::fmt::Display for UseInJsCodeWriter<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.serialize_to(f)
    }
}

pub struct RawCodeImmediate<'a>(pub &'a str);
impl<'a> UseInJsCode for RawCodeImmediate<'a> {
    fn serialize_to(&self, buf: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        buf.write_str(self.0)
    }
}

/// Values that can be serialized to JS code satisfying certain types.
///
/// For example, `ToJs<JsNumber>` means serializable to the same type that
/// `JsNumber` serializes to.
pub trait ToJs<JsType>
where
    Self: UseInJsCode,
    JsType: ?Sized,
{
}

impl<T> ToJs<T> for T where T: UseInJsCode {}
