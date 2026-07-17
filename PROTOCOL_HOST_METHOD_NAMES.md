# Configurable protocol member names

WSDOM's browser protocol emits JavaScript against its private `_w` runtime
object. `wsdom-core` exposes safe protocol-member rendering so an embedding host
can use a consistently property-mangled `_w` object without constructing raw
JavaScript property names.

## Protocol ABI

`WsdomMethod`, owned by `wsdom-core::protocol`, defines the canonical protocol
keys:

| Semantic method | Canonical `_w` member |
| --- | --- |
| `Get` | `g` |
| `Delete` | `d` |
| `Set` | `s` |
| `Reply` | `r` |
| `Error` | `e` |
| `Catch` | `c` |
| `Import` | `x` |
| `RpcReply` | `rp` |
| `Allocate` | `a` |

The shared `portal-jit-host-names` crate has no dependency on WSDOM: it only
requires this WSDOM-owned enum to implement `Display`.

## Rendering calls

For a custom `HostMethodNames<WsdomMethod>` implementation, use
`protocol::protocol_call_with_names` when producing a protocol command:

```rust
use portal_jit_host_names::MappedHostMethodNames;
use portal_jit_host_names::MappedHostMethodNames;
use wsdom_core::protocol_names::{WsdomMethod, protocol_call_with_names};

let names = MappedHostMethodNames::new([
    ("g".into(), "read".into()),
    ("d".into(), "d".into()),
    ("s".into(), "s".into()),
    ("r".into(), "r".into()),
    ("e".into(), "e".into()),
    ("c".into(), "c".into()),
    ("x".into(), "x".into()),
    ("rp".into(), "rp".into()),
    ("a".into(), "a".into()),
]);
assert_eq!(protocol_call_with_names(&names, WsdomMethod::Get, "42"), "_w.read(42)");
```

`validate_host_method_names` checks all nine protocol members before source is
rendered. An incomplete scheme is rejected rather than mixing canonical and
mangled calls. Mapped values that are not JavaScript identifiers are emitted as
escaped computed accesses, such as `_w["not-a-name"](42)`.

## Compatibility and coordination

The existing protocol constants (`GET`, `SET`, and peers) remain canonical for
existing callers. New configurable emitters should use the resolver helpers.
A custom scheme is valid only when both command producers and the browser-side
`WSDOMCore` implementation install the same complete mapping for that
connection. Mapping resolution occurs at command/source generation time; the
executed command has no runtime name lookup.