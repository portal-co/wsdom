use alloc::string::{String, ToString};
use core::fmt;

use portal_jit_host_names::{HostMethodNames, PropertyAccess};

/// WSDOM-owned semantic keys for the private `_w` protocol object.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WsdomMethod {
    Get,
    Delete,
    Set,
    Reply,
    Error,
    Catch,
    Import,
    RpcReply,
    Allocate,
}

impl fmt::Display for WsdomMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Get => "g",
            Self::Delete => "d",
            Self::Set => "s",
            Self::Reply => "r",
            Self::Error => "e",
            Self::Catch => "c",
            Self::Import => "x",
            Self::RpcReply => "rp",
            Self::Allocate => "a",
        })
    }
}

/// Resolve WSDOM's complete protocol ABI before a custom host mapping is used.
pub fn validate_host_method_names<N: HostMethodNames<WsdomMethod>>(
    names: &N,
) -> Result<(), String> {
    for method in [
        WsdomMethod::Get,
        WsdomMethod::Delete,
        WsdomMethod::Set,
        WsdomMethod::Reply,
        WsdomMethod::Error,
        WsdomMethod::Catch,
        WsdomMethod::Import,
        WsdomMethod::RpcReply,
        WsdomMethod::Allocate,
    ] {
        names.property(method).map_err(|err| err.to_string())?;
    }
    Ok(())
}

/// Render a resolved protocol member on the fixed `_w` receiver.
pub fn member<N: HostMethodNames<WsdomMethod>>(names: &N, method: WsdomMethod) -> String {
    names
        .property(method)
        .expect("WSDOM host names must be validated")
        .on("_w")
}

/// Render a resolved protocol call on the fixed `_w` receiver.
pub fn call<N: HostMethodNames<WsdomMethod>>(names: &N, method: WsdomMethod, args: &str) -> String {
    names
        .property(method)
        .expect("WSDOM host names must be validated")
        .call("_w", args)
}

pub fn protocol_call_with_names<N: HostMethodNames<WsdomMethod>>(
    names: &N,
    method: WsdomMethod,
    args: &str,
) -> String {
    validate_host_method_names(names)
        .expect("WSDOM host names must be complete before source generation");
    call(names, method, args)
}

// Canonical protocol spellings retained for existing browser APIs. Custom command
// emitters should use `member`/`call` with their selected resolver.
pub const GET: &str = "_w.g";
pub const DEL: &str = "_w.d";
pub const SET: &str = "_w.s";
pub const REP: &str = "_w.r";
pub const ERR: &str = "_w.e";
pub const CATCH: &str = "_w.c";
pub const IMPORT: &str = "_w.x";
pub const REPLY: &str = "_w.rp";
pub const ALLOC: &str = "_w.a";

#[cfg(test)]
mod tests {
    use super::*;
    use portal_jit_host_names::{CanonicalHostMethodNames, MappedHostMethodNames};

    #[test]
    fn canonical_protocol_access_is_unchanged() {
        assert_eq!(
            call(&CanonicalHostMethodNames, WsdomMethod::Get, "1"),
            "_w.g(1)"
        );
    }

    #[test]
    fn mapped_protocol_access_is_safe() {
        let names = MappedHostMethodNames::new([
            ("g".into(), "not-a-name".into()),
            ("d".into(), "d".into()),
            ("s".into(), "s".into()),
            ("r".into(), "r".into()),
            ("e".into(), "e".into()),
            ("c".into(), "c".into()),
            ("x".into(), "x".into()),
            ("rp".into(), "rp".into()),
            ("a".into(), "a".into()),
        ]);
        validate_host_method_names(&names).unwrap();
        assert_eq!(call(&names, WsdomMethod::Get, "1"), "_w[\"not-a-name\"](1)");
    }
}
