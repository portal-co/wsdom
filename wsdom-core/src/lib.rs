#![no_std]
pub extern crate alloc;
mod interaction;
mod internal;
mod js;
mod js_cast;
mod link;
mod operations;
mod protocol;
mod retrieve;
mod rpc;
mod serialize;

pub use link::{Browser, Error, RpcCellAM};
/// Protocol-member name resolution for hosts that property-mangle WSDOM's
/// private `_w` runtime object.
pub mod protocol_names {
    pub use super::protocol::{
        WsdomMethod, call, member, protocol_call_with_names, validate_host_method_names,
    };
}
pub use rpc::{Endpoint, Lock, Reply, Request, RpcDeserialize, RpcHandle};

pub mod js_types {
    //! Stubs for primitive JS types including number, string, null, undefined, object.
    pub use super::js::{
        nullable::{JsNullable, JsNullish},
        object::JsObject,
        primitives::*,
        value::JsValue,
    };
}
pub use interaction::r#await;
pub use interaction::callback;
pub use js_cast::{Cast, JsCast};
pub use serialize::{ToJs, UseInJsCode};
pub mod immediates {
    pub use super::js::immediates::{null, undefined};
}

#[doc(hidden)]
pub mod for_macro {
    pub use super::internal::upcast_workaround::UpcastWorkaround;
    pub use super::link::BrowserInternal;
    pub use super::serialize::RawCodeImmediate;
}
