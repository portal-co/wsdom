use core::marker::PhantomData;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use spin::Mutex;
use core::{fmt::Write, future::Future, pin::Pin, task::Poll};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::js_types::JsValue;
use crate::link::{BrowserInternal, Error, RetrievalState};
use crate::protocol::{GET, REP, CATCH};
use crate::Browser;

/// A [Future] for retrieving value from the JS side to the Rust side.
///
/// If something goes wrong (for example if the network disconnects), this Future will simply pend forever.
pub struct RetrieveFuture<'a, T: DeserializeOwned> {
    pub(crate) id: u64,
    pub(crate) ret_id: u64,
    // pub(crate) error_slot: u64,
    pub(crate) link: &'a Browser,
    _phantom: PhantomData<Pin<Box<T>>>,
}

impl<'a, T: DeserializeOwned> RetrieveFuture<'a, T> {
    pub(crate) fn new(id: u64, link: &'a Browser) -> Self {
        Self {
            id,
            ret_id: 0,
            // error_slot: 0,
            link,
            _phantom: PhantomData,
        }
    }
}
#[derive(Serialize,Deserialize)]
#[serde(untagged)]
enum ResI<T>{
    Value{
        value: T
    },
    Error{
        error: u64
    }
}
impl<'a, T: DeserializeOwned> Future for RetrieveFuture<'a, T> {
    type Output = Result<T,JsValue>;
    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let mut link = this.link.0.lock();
        match this.ret_id {
            0 => {
                let ret_id = link.get_new_id();
                this.ret_id = ret_id;
                let this_id = this.id;
                // this.error_slot = link.get_new_id();
                // let error_slot = this.error_slot;
                writeln!(link.raw_commands_buf(), "{REP}({ret_id},{CATCH}({this_id}));").unwrap();
                link.wake_outgoing();
                link.retrievals.insert(
                    ret_id,
                    RetrievalState {
                        waker: cx.waker().to_owned(),
                        last_value: String::new(),
                        times: 0,
                    },
                );
                Poll::Pending
            }
            ret_id => match link.retrievals.entry(ret_id) {
                hashbrown::hash_map::Entry::Occupied(mut occ) => match occ.get_mut() {
                    RetrievalState {
                        waker, times: 0, ..
                    } => {
                        let new_waker = cx.waker();
                        if !waker.will_wake(new_waker) {
                            *waker = new_waker.to_owned();
                        }
                        Poll::Pending
                    }
                    RetrievalState { last_value, .. } => {
                        let v = core::mem::take(last_value);
                        occ.remove();
                        let v = v.split_once(':').unwrap().1;
                        match serde_json::from_str(v) {
                            Ok(v) => {
                                let v = match v{
                                    ResI::Value { value } => Ok(value),
                                    ResI::Error { error } => Err(JsValue { id: error, browser: this.link.clone() }),
                                };
                                this.ret_id = 0;
                                Poll::Ready(v)
                            }
                            Err(e) => {
                                link.kill(Error::DataDeserialize(e));
                                Poll::Pending
                            }
                        }
                    }
                },
                hashbrown::hash_map::Entry::Vacant(_) => Poll::Pending,
            },
        }
    }
}

impl<'a, T: DeserializeOwned> Drop for RetrieveFuture<'a, T> {
    fn drop(&mut self) {
        match self.ret_id {
            0 => {
                // NO-OP
            }
            ret_id => {
                let mut link = self.link.0.lock();
                link.retrievals.remove(&ret_id);
            }
        }
    }
}
