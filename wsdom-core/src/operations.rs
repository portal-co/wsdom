use crate::{
    js::{object::JsObject, value::JsValue},
    js_cast::JsCast,
    link::{Browser, Error, RpcCell},
    protocol::{ERR, GET, IMPORT, SET},
    retrieve::RetrieveFuture,
    serialize::{ToJs, UseInJsCode, UseInJsCodeWriter},
    Endpoint, RpcHandle,
};
use alloc::{borrow::ToOwned, sync::Arc};
use core::{
    fmt::Write,
    marker::PhantomData,
    // sync::{Arc, Mutex},
    task::Waker,
};
use futures_util::task::noop_waker_ref;
use sha3::Digest;
use spin::Mutex;

impl Browser {
    /// Creates a new RPC endpoint
    pub fn new_rpc<T, C>(&self, a: &str) -> crate::RpcHandle<Endpoint<T, C>> {
        let mut lock = self.0.lock();
        let a = lock
            .rpc_state
            .entry(a.to_owned())
            .or_insert_with(|| {
                crate::RpcCellAM(Arc::new(Mutex::new(RpcCell {
                    waker: noop_waker_ref().clone(),
                    queue: Default::default(),
                })))
            })
            .clone();
        return RpcHandle {
            browser: self.clone(),
            recv: a,
            data: Endpoint {
                phantom: PhantomData,
            },
        };
    }
    /// Call a standalone JavaScript function.
    ///
    /// ```rust
    /// # use wsdom_core::Browser;
    /// fn example(browser: Browser) {
    ///     let _return_value = browser.call_function(
    ///         "alert",
    ///         [&"hello world" as &_],
    ///         false
    ///     );
    /// }
    /// ```
    ///
    /// This method is "low-level" and you shouldn't need to use it.
    /// Instead, use the `wsdom` crate which provides mostly type-safe wrappers to the Web API.
    ///
    /// If you still want to use `call_function`,
    /// be aware that the first argument (`function_name`) is NOT escaped.
    /// Do NOT allow user-supplied function name.
    pub fn call_function<'a>(
        &'a self,
        function_name: &'a str,
        args: impl IntoIterator<Item = &'a dyn UseInJsCode>,
        last_arg_variadic: bool,
    ) -> JsValue {
        self.call_function_inner(&format_args!("{}", function_name), args, last_arg_variadic)
    }

    /// Call constructor for a class.
    ///
    /// ```rust
    /// # use wsdom_core::Browser;
    /// fn example(browser: Browser) {
    ///     let _regexp_object = browser.call_constructor(
    ///         "RegExp",
    ///         [&"hello" as &_],
    ///         false
    ///     );
    /// }
    /// ```
    ///
    /// This method is "low-level" and you shouldn't need to use it.
    /// Instead, use the `wsdom` crate which provides mostly type-safe wrappers to the Web API.
    ///
    /// If you still want to use `call_constructor`,
    /// be aware that the first argument (`class_name`) is NOT escaped.
    /// Do NOT allow user-supplied class name.
    pub fn call_constructor<'a>(
        &'a self,
        class_name: &'a str,
        args: impl IntoIterator<Item = &'a dyn UseInJsCode>,
        last_arg_variadic: bool,
    ) -> JsValue {
        self.call_function_inner(&format_args!("new {}", class_name), args, last_arg_variadic)
    }

    fn call_function_inner<'a>(
        &'a self,
        function: &core::fmt::Arguments<'_>,
        args: impl IntoIterator<Item = &'a dyn UseInJsCode>,
        last_arg_variadic: bool,
    ) -> JsValue {
        let id = {
            let mut link = self.0.lock();
            let out_id = link.get_new_id();
            write!(link.raw_commands_buf(), "try{{{SET}({out_id},{function}(").unwrap();
            let mut iter = args.into_iter().peekable();
            while let Some(arg) = iter.next() {
                let arg = UseInJsCodeWriter(arg);
                let res = if last_arg_variadic && iter.peek().is_none() {
                    write!(link.raw_commands_buf(), "...{arg},")
                } else {
                    write!(link.raw_commands_buf(), "{arg},")
                };
                if let Err(e) = res {
                    link.kill(Error::CommandSerialize(e));
                }
            }
            write!(
                link.raw_commands_buf(),
                "))}}catch($){{{ERR}({out_id},$)}};\n"
            )
            .unwrap();
            link.wake_outgoing();
            out_id
        };
        JsValue {
            id,
            browser: self.clone(),
        }
    }

    /// Get a field in an object.
    ///
    /// This returns the value of `base_obj[property]`.
    pub fn get_field(&self, base_obj: &dyn UseInJsCode, property: &dyn UseInJsCode) -> JsValue {
        let browser = self.clone();
        let id = {
            let mut link = browser.0.lock();
            let out_id = link.get_new_id();
            let base_obj = UseInJsCodeWriter(base_obj);
            let property = UseInJsCodeWriter(property);
            if let Err(e) = writeln!(
                link.raw_commands_buf(),
                "try{{{SET}({out_id},({base_obj})[{property}])}}catch($){{{ERR}({out_id},$)}};"
            ) {
                link.kill(Error::CommandSerialize(e));
            }
            link.wake_outgoing_lazy();
            out_id
        };
        JsValue { id, browser }
    }

    /// Set a field in an object.
    ///
    /// This executes the JavaScript code `base_obj[property]=value;`
    pub fn set_field(
        &self,
        base_obj: &dyn UseInJsCode,
        property: &dyn UseInJsCode,
        value: &dyn UseInJsCode,
    ) {
        let mut link = self.0.lock();
        let (base_obj, property, value) = (
            UseInJsCodeWriter(base_obj),
            UseInJsCodeWriter(property),
            UseInJsCodeWriter(value),
        );
        if let Err(e) = writeln!(link.raw_commands_buf(), "({base_obj})[{property}]={value};") {
            link.kill(Error::CommandSerialize(e));
        }
        link.wake_outgoing();
    }

    /// Create a new value on the JavaScript side from a [ToJs] type.
    pub fn new_value<'a, T: JsCast>(&'a self, value: &'a dyn ToJs<T>) -> T {
        let val = self.value_from_raw_code(format_args!("{}", UseInJsCodeWriter(value)));
        JsCast::unchecked_from_js(val)
    }

    /// Executes arbitrary JavaScript code.
    ///
    /// Don't use this unless you really have to.
    pub fn run_raw_code<'a>(&'a self, code: core::fmt::Arguments<'a>) {
        let mut link = self.0.lock();
        if let Err(e) = writeln!(link.raw_commands_buf(), "{{ {code} }}") {
            link.kill(Error::CommandSerialize(e));
        }
        link.wake_outgoing();
    }

    /// Executes arbitrary JavaScript expression and return the result.
    ///
    /// Don't use this unless you really have to.
    pub fn value_from_raw_code<'a>(&'a self, code: core::fmt::Arguments<'a>) -> JsValue {
        let mut link = self.0.lock();
        let out_id = link.get_new_id();
        writeln!(
            link.raw_commands_buf(),
            "try{{{SET}({out_id},{code})}}catch($){{{ERR}({out_id},$)}};"
        )
        .unwrap();
        link.wake_outgoing();
        JsValue {
            id: out_id,
            browser: self.to_owned(),
        }
    }

    /// Executesand caches  arbitrary JavaScript expression and return the result.
    ///
    /// Don't use this unless you really have to.
    pub fn value_from_pure_raw_code(&self, x: &str) -> JsValue {
        let mut link = self.0.lock();
        let a = match link.pure_values.get(x).cloned() {
            None => {
                let out = self.value_from_raw_code(format_args!("{x}"));
                link.pure_values.insert(x.to_owned(), out.clone());
                out
            }
            Some(a) => a,
        };
        return a;
    }

    /// Gets an import from the available ones
    pub fn import(&self, name: &str) -> JsValue {
        let browser = self.clone();
        let mut link = self.0.lock();
        let a = match link.imports.get(name).cloned() {
            None => {
                let out_id = link.get_new_id();
                writeln!(
                    link.raw_commands_buf(),
                    "try{{{SET}({out_id},{IMPORT}._{})}}catch($){{{ERR}({out_id},$)}};",
                    hex::encode(&sha3::Sha3_256::digest(name.as_bytes()))
                )
                .unwrap();
                link.wake_outgoing();
                let out = JsValue {
                    id: out_id,
                    browser,
                };
                link.imports.insert(name.to_owned(), out.clone());
                out
            }
            Some(a) => a,
        };
        return a;
    }
}

impl JsValue {
    pub(crate) fn retrieve_and_deserialize<U: serde::de::DeserializeOwned>(
        &self,
    ) -> RetrieveFuture<'_, U> {
        RetrieveFuture::new(self.id, &self.browser)
    }
    /// Retrive this value from the JS side to the Rust side.
    /// Returns Future whose output is a [serde_json::Value].
    ///
    /// ```rust
    /// # use wsdom::Browser;
    /// # use wsdom::dom::HTMLInputElement;
    /// async fn example(input: &HTMLInputElement) {
    ///     let _val = input.get_value().retrieve_json().await;
    /// }
    /// ```
    pub fn retrieve_json(&self) -> RetrieveFuture<'_, serde_json::Value> {
        self.retrieve_and_deserialize()
    }
}
impl JsObject {
    /// Get a field value of in this object.
    ///
    /// WSDOM provides built-in getters so you should use that instead when possible.
    ///
    /// Use `js_get_field` only when needed
    ///
    /// ```rust
    /// # use wsdom_core::Browser;
    /// # use wsdom_core::js_types::*;
    /// fn example(browser: Browser) {
    ///     // you can get `window["location"]["href"]` like this
    ///     let href: JsValue = wsdom::dom::location(&browser).js_get_field(&"href");
    ///
    ///     // but you should use built-in getters instead
    ///     let href: JsString = wsdom::dom::location(&browser).get_href();
    /// }
    /// ```
    pub fn js_get_field(&self, property: &dyn UseInJsCode) -> JsValue {
        let browser = self.browser.clone();
        let id = {
            let mut link = browser.0.lock();
            let out_id = link.get_new_id();
            let self_id = self.id;
            let property = UseInJsCodeWriter(property);
            if let Err(e) = writeln!(
                link.raw_commands_buf(),
                "try{{{SET}({out_id},{GET}({self_id})[{property}])}}catch($){{{ERR}({out_id},$)}};"
            ) {
                link.kill(Error::CommandSerialize(e));
            }
            link.wake_outgoing_lazy();
            out_id
        };
        JsValue { id, browser }
    }
    /// Set a field value of in this object.
    ///
    /// WSDOM provides built-in setters so you should use that instead when possible.
    ///
    /// Use `js_set_field` only when needed
    ///
    /// ```rust
    /// # use wsdom_core::Browser;
    /// # use wsdom_core::js_types::*;
    /// fn example(browser: Browser) {
    ///     // you can set `window["location"]["href"]` like this
    ///     wsdom::dom::location(&browser).js_set_field(&"href", &"https://example.com/");
    ///
    ///     // but you should use built-in setters instead
    ///     wsdom::dom::location(&browser).set_href(&"https://example.com");
    /// }
    /// ```
    pub fn js_set_field(&self, property: &dyn UseInJsCode, value: &dyn UseInJsCode) {
        let self_id = self.id;
        let mut link = self.browser.0.lock();
        let (property, value) = (UseInJsCodeWriter(property), UseInJsCodeWriter(value));
        if let Err(e) = writeln!(
            link.raw_commands_buf(),
            "{GET}({self_id})[{property}]={value};"
        ) {
            link.kill(Error::CommandSerialize(e));
        }
        link.wake_outgoing();
    }

    /// Call a method on this object.
    ///
    /// Most types in WSDOM already come with safe Rust wrappers for their methods, so you should use those instead.
    ///
    /// ```rust
    /// # use wsdom_core::Browser;
    /// fn example(browser: &Browser) {
    ///     let console = wsdom::dom::console(browser);
    ///     // you can call console.log like this
    ///     console.js_call_method("log", [&"hello" as &_], false);
    ///     
    ///     // but the better way is to use
    ///     wsdom::dom::console(&browser).log(&[&"Hello" as &_]);
    /// }
    /// ```
    ///
    /// Be aware that the first argument (`method_name`) is NOT escaped.
    ///
    /// Set `last_arg_variadic` to `true` if you want to "spread" the last argument as `obj.method(arg1, arg2, ...arg3)`.
    pub fn js_call_method<'a>(
        &'a self,
        method_name: &'a str,
        args: impl IntoIterator<Item = &'a dyn UseInJsCode>,
        last_arg_variadic: bool,
    ) -> JsValue {
        let self_id = self.id;
        self.browser.call_function_inner(
            &format_args!("{GET}({self_id}).{method_name}"),
            args,
            last_arg_variadic,
        )
    }
    /// Call this object: `obj()`.
    ///
    /// Most types in WSDOM already come with safe Rust wrappers for their methods, so you should use those instead.
    pub fn js_call_self<'a>(
        &'a self,
        args: impl IntoIterator<Item = &'a dyn UseInJsCode>,
        last_arg_variadic: bool,
    ) -> JsValue {
        let self_id = self.id;
        self.browser.call_function_inner(
            &format_args!("({GET}({self_id}))"),
            args,
            last_arg_variadic,
        )
    }
}

struct CommandSerializeFailed;

impl core::fmt::Display for CommandSerializeFailed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}
impl core::fmt::Debug for CommandSerializeFailed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CommandSerializeFailed").finish()
    }
}

impl core::error::Error for CommandSerializeFailed {}
