use core::{future::Future, marker::PhantomData};

use crate::js::value::JsValue;

/// For converting between JavaScript types.
///
/// Note that most class types generated by WSDOM also comes with `Into` and `AsRef` impls for converting to their ancestors in the inheritance chain.
pub trait JsCast
where
    Self: AsRef<JsValue> + Into<JsValue>,
{
    // fn instanceof(val: &JsValue) -> bool;
    fn unchecked_from_js(val: JsValue) -> Self;
    fn unchecked_from_js_ref(val: &JsValue) -> &Self;

    // fn has_type<T>(&self) -> bool
    //    where T: JsCast { ... }
    // fn dyn_into<T>(self) -> Result<T, Self>
    //    where T: JsCast { ... }
    // fn dyn_ref<T>(&self) -> Option<&T>
    //    where T: JsCast { ... }
    fn unchecked_into<T>(self) -> T
    where
        T: JsCast,
    {
        T::unchecked_from_js(self.into())
    }
    fn unchecked_ref<T>(&self) -> &T
    where
        T: JsCast,
    {
        T::unchecked_from_js_ref(self.as_ref())
    }
    // fn is_instance_of<T>(&self) -> bool
    //    where T: JsCast { ... }
    // fn is_type_of(val: &JsValue) -> bool { ... }
}
pin_project_lite::pin_project! {
    #[derive(Clone, Copy)]
    pub struct Cast<T,U>{
        #[pin]
        pub value: T,
        pub phantom: PhantomData<U>
    }
}
impl<T: Future<Output: JsCast>, U: JsCast> Future for Cast<T, U> {
    type Output = U;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        self.project().value.poll(cx).map(|a| a.unchecked_into())
    }
}
