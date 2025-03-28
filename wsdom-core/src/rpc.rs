use core::error::Error;
use core::marker::PhantomData;
use core::task::Poll;

use futures_core::Stream;

use crate::js_types::JsValue;
use crate::{link::RpcCellAM, serialize::UseInJsCodeWriter, Browser, UseInJsCode};

use crate::protocol::REPLY;
use core::fmt::Write;
pin_project_lite::pin_project! {

pub struct RpcHandle<C> {
    #[pin]
    pub(crate) recv: RpcCellAM,
    pub(crate) browser: Browser,
    pub(crate) data: C,
}
}
pub struct Reply<C> {
    pub(crate) phantom: PhantomData<C>,
    pub(crate) id: u64,
}
impl<C: UseInJsCode> RpcHandle<Reply<C>> {
    pub fn reply(self, c: C) {
        let mut link = self.browser.0.lock();
        let id = self.data.id;
        writeln!(
            link.raw_commands_buf(),
            "{REPLY}({id},{})",
            UseInJsCodeWriter(&c)
        )
        .unwrap();
        link.wake_outgoing_lazy();
    }
}
pub struct Request<T, C> {
    pub(crate) reply: Reply<C>,
    pub(crate) data: T,
}
impl<C: UseInJsCode, T> RpcHandle<Request<T, C>> {
    pub fn decaps(self) -> (T, RpcHandle<Reply<C>>) {
        (
            self.data.data,
            RpcHandle {
                recv: self.recv,
                browser: self.browser,
                data: self.data.reply,
            },
        )
    }
}
pub struct Endpoint<T, C> {
    pub(crate) phantom: PhantomData<(T, C)>,
}
pub struct Lock {
    nope: (),
}
pub trait RpcDeserialize: Sized {
    fn deser<'a>(
        a: &'a str,
        browser: &Browser,
        recv: &RpcCellAM,
        lock: &Lock,
    ) -> Result<(Self, &'a str), ()>;
}
impl RpcDeserialize for () {
    fn deser<'a>(
        a: &'a str,
        browser: &Browser,
        recv: &RpcCellAM,
        lock: &Lock,
    ) -> Result<(Self, &'a str), ()> {
        Ok(((), a))
    }
}
impl<A: RpcDeserialize, B: RpcDeserialize> RpcDeserialize for (A, B) {
    fn deser<'a>(
        a: &'a str,
        browser: &Browser,
        recv: &RpcCellAM,
        lock: &Lock,
    ) -> Result<(Self, &'a str), ()> {
        let (v0, a) = A::deser(a, browser, recv, lock)?;
        let (v1, a) = B::deser(a, browser, recv, lock)?;
        Ok(((v0, v1), a))
    }
}
impl RpcDeserialize for u64 {
    fn deser<'a>(
        a: &'a str,
        browser: &Browser,
        recv: &RpcCellAM,
        lock: &Lock,
    ) -> Result<(Self, &'a str), ()> {
        let Some((s, a)) = a.split_once(";") else {
            return Err(());
        };
        let Ok(v) = u64::from_str_radix(s, 10) else {
            return Err(());
        };
        return Ok((v, a));
    }
}
impl RpcDeserialize for JsValue {
    fn deser<'a>(
        a: &'a str,
        browser: &Browser,
        recv: &RpcCellAM,
        lock: &Lock,
    ) -> Result<(Self, &'a str), ()> {
        let (v, a) = u64::deser(a, browser, recv, lock)?;
        return Ok((
            JsValue {
                id: v,
                browser: browser.clone(),
            },
            a,
        ));
    }
}
impl<C: UseInJsCode, T: RpcDeserialize> Stream for RpcHandle<Endpoint<T, C>> {
    type Item = RpcHandle<Request<T, C>>;

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        let (browser, recv) = (self.browser.clone(), self.recv.clone());
        let mut project = self.project();
        loop {
            let Poll::Ready(a) = project.recv.as_mut().poll_next(cx) else {
                return Poll::Pending;
            };
            let Some(a) = a else {
                return Poll::Ready(None);
            };
            let Ok(((r, a), _)) =
                RpcDeserialize::deser(a.as_str(), &browser, &recv, &Lock { nope: () })
            else {
                continue;
            };
            return Poll::Ready(Some(RpcHandle {
                recv: recv,
                browser: browser,
                data: Request {
                    reply: Reply {
                        phantom: PhantomData,
                        id: r,
                    },
                    data: a,
                },
            }));
        }
    }
}
#[macro_export]
macro_rules! wrap {
    ([$($g:tt)*] as $e:expr => $ty:ty) => {
        impl $($g)* $crate::RpcDeserialize for $ty{
            fn deser<'a>(
                a: &'a str,
                browser: &$crate::Browser,
                recv: &$crate::RpcCellAM,
                lock: &$crate::Lock,
            ) -> Result<(Self, &'a str), ()> {
                let (v,a) = $crate::RpcDeserialize::deser(a,browser,recv,lock)?;
                Ok(($e(v),a))
            }
        }
    };
}
