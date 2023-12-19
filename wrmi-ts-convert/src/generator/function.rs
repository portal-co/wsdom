use crate::{generator::utils::to_snake_case, parser::declare_function::DeclareFunction};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::Context;

impl<'a> Context<'a> {
    pub(super) fn make_function(&self, df: &DeclareFunction<'a>) -> TokenStream {
        let function_name_ident = Ident::new(&to_snake_case(df.name), Span::call_site());
        let arg_names_sig = df
            .args
            .iter()
            .map(|arg| Ident::new(arg.name, Span::call_site()));
        let arg_names_body = arg_names_sig.clone();
        let arg_types = df.args.iter().map(|arg| {
            let arg_type = self.convert_type(arg.ty.to_owned());
            quote! {&impl __wrmi_load_ts_macro::ToJs<#arg_type>}
        });
        let last_arg_variadic = df.args.iter().any(|arg| arg.variadic);
        let ret = self.convert_type(df.ret.to_owned());
        let function_generics = df.generics.args.iter().map(|gen| {
            let name = Ident::new(gen.name, Span::call_site());
            let extends = gen
                .extends
                .clone()
                .map(|ty| self.convert_type(ty))
                .into_iter();
            quote! {
                #name #(: ::core::convert::AsRef<#extends> + ::core::convert::Into<#extends>)*
            }
        });
        let function_generics = if df.generics.args.is_empty() {
            None
        } else {
            Some(quote! {
                <#(#function_generics,)*>
            })
        };
        let function = df.name;
        quote! {
            pub fn #function_name_ident #function_generics (browser: &__wrmi_load_ts_macro::Browser, #(#arg_names_sig: #arg_types,)*) -> #ret {
                __wrmi_load_ts_macro::JsCast::unchecked_from_js(
                    browser.call_function(#function, [
                        #( #arg_names_body as &dyn __wrmi_load_ts_macro::UseInJsCode,)*
                    ], #last_arg_variadic)
                )
            }
        }
    }
}