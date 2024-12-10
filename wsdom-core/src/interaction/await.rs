use std::{
    future::{Future, IntoFuture},
    task::Poll,
    fmt::Write,
};

use crate::{
    js_types::JsValue,
    link::RetrievalState,
    protocol::{DEL, GET, REP, SET},
    Browser,
};

pub struct Await {
    browser: Browser,
    ret_id: u64,
    cell_id: u64,
}

impl Future for Await {
    type Output = JsValue;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        let mut link = this.browser.0.lock().unwrap();
        let ret_id = this.ret_id;
        match link.retrievals.entry(ret_id) {
            std::collections::hash_map::Entry::Occupied(mut occ) => {
                let state = occ.get_mut();

                let new_waker = cx.waker();
                if !state.waker.will_wake(new_waker) {
                    state.waker = new_waker.to_owned();
                }

                if state.times != 0 {
                    let val_id = link.get_new_id();
                    let cell_id = this.cell_id;
                    writeln!(
                        link.raw_commands_buf(),
                        "{SET}({val_id}, {GET}({cell_id}).$), {DEL}({cell_id});"
                    )
                    .unwrap();
                    link.wake_outgoing_lazy();
                    Poll::Ready(JsValue {
                        browser: this.browser.clone(),
                        id:val_id,
                    })
                } else {
                    Poll::Pending
                }
            }
            std::collections::hash_map::Entry::Vacant(vac) => {
                vac.insert(RetrievalState {
                    waker: cx.waker().to_owned(),
                    last_value: String::new(),
                    times: 0,
                });
                Poll::Pending
            }
        }
    }
}
impl IntoFuture for JsValue {
    type Output = JsValue;

    type IntoFuture = Await;

    fn into_future(self) -> Self::IntoFuture {
        let mut link = self.browser.0.lock().unwrap();
        let ret_id = link.get_new_id();
        let cell_id = link.get_new_id();
        let id = self.id;
        writeln!(
            link.raw_commands_buf(),
            "{SET}({cell_id},{{}}), Promise.prototype.then.call({GET}({id}),function(e) {{{GET}({cell_id}).$ = e; {REP}({ret_id}, 0) }}));"
        )
        .unwrap();
        link.wake_outgoing_lazy();
        return Await {
            browser: self.browser.clone(),
            ret_id,
            cell_id,
        };
    }
}
