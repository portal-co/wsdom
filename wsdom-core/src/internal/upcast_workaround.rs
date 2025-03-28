use crate::serialize::{ToJs, UseInJsCode};

pub struct UpcastWorkaround<'a, T: ?Sized, JsType> {
    ty: &'a T,
    _phantom: core::marker::PhantomData<JsType>,
}

impl<'a, T: ToJs<JsType> + ?Sized, JsType> UpcastWorkaround<'a, T, JsType> {
    pub fn new(ty: &'a T) -> Self {
        Self {
            ty,
            _phantom: core::marker::PhantomData,
        }
    }
    pub fn cast(&self) -> &dyn UseInJsCode {
        self
    }
}

impl<'a, T: ToJs<JsType> + ?Sized, JsType> UseInJsCode for UpcastWorkaround<'a, T, JsType> {
    fn serialize_to(&self, buf: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.ty.serialize_to(buf)
    }
}
