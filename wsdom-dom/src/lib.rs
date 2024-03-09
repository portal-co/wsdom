#![allow(non_snake_case)]
mod __wsdom_load_ts_macro {
    pub use ref_cast::RefCast;
    pub use wsdom_core::{
        for_macro::{RawCodeImmediate, UpcastWorkaround},
        js_types::*,
        Browser, JsCast, ToJs, UseInJsCode,
    };
    pub use wsdom_macros_decl::*;
}
use wsdom_javascript::*;

wsdom_macros::load_ts!("../typescript-defs/used/dom.generated.d.ts");