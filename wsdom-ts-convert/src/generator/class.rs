use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    generator::{types::SimplifiedType, util::iter_dedupe_all},
    parser::{
        comment::WithComment,
        field::{Field, FieldName},
        interface::Interface,
        member::{Getter, Member, Setter},
        method::{Method, MethodName},
        ts_type::{NamedType, TsType},
    },
};

use super::{
    types::known_types,
    util::{new_ident_safe, to_snake_case},
    Context,
};

impl<'a> Context<'a> {
    pub(super) fn get_members<'b>(
        &'b self,
        ty: &'b TsType<'b>,
    ) -> Option<(&'b Vec<WithComment<'b, Member<'b>>>, bool)> {
        match ty {
            TsType::Named {
                ty: NamedType { name, .. },
            } => self
                .interfaces
                .get(name)
                .map(|interface| (&interface.members, true)),
            TsType::Interface { members } => Some((members, false)),
            _ => None,
        }
    }
    pub(super) fn make_class(
        &self,
        interface: &Interface<'_>,
        decl_members: &[WithComment<'_, Member<'_>>],
    ) -> TokenStream {
        let name = new_ident_safe(interface.name);

        let (
            generics_with_bounds,
            generics_without_bounds,
            generics_with_defaults,
            generics_for_phantom,
        ) = if interface.generics.args.is_empty() {
            (None, None, None, quote! {()})
        } else {
            let with_bounds_with_defaults = interface.generics.args.iter().map(|arg| {
                let name = new_ident_safe(arg.name);
                let bounds = arg
                    .extends
                    .clone()
                    .map(|t| {
                        let t = self.convert_type(self.simplify_type(t));
                        quote! {
                            ::core::convert::AsRef<#t> + ::core::convert::Into<#t>
                        }
                    })
                    .into_iter();
                let bounds_cloned = bounds.clone();
                let default = arg.default.clone().map(|t| {
                    let t = self.convert_type(self.simplify_type(t));
                    quote! {
                        = #t
                    }
                });
                (
                    quote! {
                        #name: __wsdom_load_ts_macro::JsCast  #(+ #bounds)*
                    },
                    quote! {
                        #name: __wsdom_load_ts_macro::JsCast  #(+ #bounds_cloned)* #default
                    },
                )
            });
            let with_bounds = with_bounds_with_defaults.clone().map(|(b, _d)| b);
            let with_defaults = with_bounds_with_defaults.map(|(_b, d)| d);
            let without_bounds = interface.generics.args.iter().map(|arg| {
                let name = new_ident_safe(arg.name);
                quote! {
                    #name
                }
            });
            let without_bounds_tokens = quote! {#(#without_bounds,)*};
            (
                Some(quote! { <#(#with_bounds,)*> }),
                Some(quote! { <#without_bounds_tokens> }),
                Some(quote! { <#(#with_defaults,)*> }),
                quote! {(#without_bounds_tokens)},
            )
        };

        let mut ancestors = Vec::new();

        static ALWAYS_EXTENDED: &[SimplifiedType] = &[known_types::UNKNOWN, known_types::OBJECT];
        interface
            .extends
            .iter()
            .map(|ty| self.simplify_type(ty.to_owned()))
            .chain(ALWAYS_EXTENDED.iter().cloned())
            .for_each(|ty| {
                self.visit_all_ancestors(&ty, &mut |ext| {
                    ancestors.push(ext.to_owned());
                    None::<()>
                });
                ancestors.push(ty);
            });
        ancestors.sort_by_key(|item| item.name);
        ancestors.dedup_by_key(|item| item.name);

        let superclass = interface.extends.iter().find_map(|iface| {
            let parent = self.simplify_type(iface.to_owned());
            self.classes.contains(parent.name).then_some(parent)
        });
        let superclass_token = self.convert_type(
            superclass
                .as_ref()
                .unwrap_or(&known_types::OBJECT)
                .to_owned(),
        );
        let extends = ancestors.into_iter().map(|anc| self.convert_type(anc));

        let tokens = quote! {
            __wsdom_load_ts_macro::expand_class_def!(
                #generics_for_phantom,
                [#generics_with_bounds],
                #name #generics_without_bounds,
                #name,
                [#generics_with_defaults],
                #superclass_token,
                #(#extends,)*
            );
        };

        let tokens = {
            let mut member_tokens = Vec::new();
            let all_members = decl_members
                .iter()
                .map(|member| (member, false))
                .chain(interface.members.iter().map(|member| (member, true)))
                .chain(
                    interface
                        .extends
                        .iter()
                        .filter(|iface| {
                            !self
                                .classes
                                .contains(self.simplify_type((*iface).to_owned()).name)
                        })
                        .filter(|iface| {
                            // TODO: this is a hack to filter out generics because I'm too lazy to implement substitution
                            match iface {
                                TsType::Named { ty } => self
                                    .interfaces
                                    .get(ty.name)
                                    .is_some_and(|iface| iface.generics.args.is_empty()),
                                _ => true,
                            }
                        })
                        .flat_map(|iface| {
                            self.get_members(iface)
                                .into_iter()
                                .filter_map(|(members, on_instance)| {
                                    on_instance
                                        .then_some(members.iter().map(|member| (member, true)))
                                })
                                .flatten()
                        }),
                );

            let methods =
                all_members
                    .clone()
                    .filter_map(|(member, on_instance)| match &member.data {
                        Member::Method(x) => Some((x, on_instance)),
                        _ => None,
                    });
            let fields =
                all_members
                    .clone()
                    .filter_map(|(member, on_instance)| match &member.data {
                        Member::Field(x) => Some((x, on_instance)),
                        _ => None,
                    });
            let getters =
                all_members
                    .clone()
                    .filter_map(|(member, on_instance)| match &member.data {
                        Member::Getter(x) => Some((x, on_instance)),
                        _ => None,
                    });
            let setters =
                all_members
                    .clone()
                    .filter_map(|(member, on_instance)| match &member.data {
                        Member::Setter(x) => Some((x, on_instance)),
                        _ => None,
                    });
            {
                let mut methods = iter_dedupe_all(methods.rev(), |(m, _)| (&m.name, m.args.len()))
                    .collect::<Vec<_>>();
                methods.sort_unstable_by_key(|(m, _)| m.args.len());
                let mut generated_methods = HashSet::new();

                for (method, on_instance) in methods.into_iter() {
                    let field_name_conflict = match method.name {
                        MethodName::Name(name)
                            if (name.starts_with("set") || name.starts_with("get"))
                                && name.chars().nth(3).is_some_and(|c| c.is_ascii_uppercase()) =>
                        {
                            fields
                                .clone()
                                .find(|field| match field.0.name {
                                    FieldName::Name(field_name)
                                        if name.chars().skip(3).zip(field_name.chars()).all(
                                            |(c1, c2)| {
                                                c1.to_ascii_lowercase() == c2.to_ascii_lowercase()
                                            },
                                        ) =>
                                    {
                                        true
                                    }
                                    _ => false,
                                })
                                .is_some()
                        }
                        _ => false,
                    };
                    let is_overload = generated_methods.contains(&method.name);
                    member_tokens.push(self.make_method_code(
                        interface.name,
                        method,
                        on_instance,
                        is_overload,
                        field_name_conflict,
                    ));
                    generated_methods.insert(&method.name);
                }
            }
            {
                let fields = iter_dedupe_all(fields.rev(), |(f, _)| match &f.name {
                    FieldName::Name(s) => *s,
                    FieldName::Wildcard { .. } => "[]",
                });
                for (field, on_instance) in fields {
                    member_tokens.push(self.make_field_code(interface.name, field, on_instance));
                }
            }
            {
                for (getter, on_instance) in getters {
                    member_tokens.push(self.make_getter_code(interface.name, getter, on_instance));
                }
            }
            {
                for (setter, on_instance) in setters {
                    member_tokens.push(self.make_setter_code(interface.name, setter, on_instance));
                }
            }

            quote! {
                #tokens
                impl #generics_with_bounds #name #generics_without_bounds {
                    #(
                        #member_tokens
                    )*
                }
            }
        };

        tokens
    }
    fn make_method_code(
        &self,
        interface_name: &'_ str,
        method: &Method<'_>,
        on_instance: bool,
        is_overload: bool,
        field_name_conflict: bool,
    ) -> Option<TokenStream> {
        let is_constructor = matches!(method.name, crate::parser::method::MethodName::Constructor);
        let method_name_str = match method.name {
            crate::parser::method::MethodName::Nothing => "call_self",
            crate::parser::method::MethodName::Constructor => "new",
            crate::parser::method::MethodName::Iterator => return None,
            crate::parser::method::MethodName::Name(name) => name,
        };
        let mut rust_name_str = to_snake_case(method_name_str);

        if is_overload {
            rust_name_str.push_str("_with");
            for arg in &method.args {
                rust_name_str.push('_');
                rust_name_str.push_str(arg.name);
            }
        } else {
            if field_name_conflict {
                rust_name_str.push_str("_method");
            }
        }
        let method_name_ident = new_ident_safe(&rust_name_str);
        let (arg_types, arg_names, last_arg_variadic) = self.make_sig_args(&method.args);
        // let arg_names_body = arg_names_sig.clone();
        let ret = self.convert_type(
            method
                .ret
                .to_owned()
                .map(|t| self.simplify_type(t))
                .unwrap_or(known_types::NULL),
        );
        let method_generics = self.make_sig_generics(&method.generics.args);
        Some(match (on_instance, is_constructor) {
            (true, true) => quote! {
                __wsdom_load_ts_macro::expand_method!(constructor @ #method_name_ident, [#method_generics], [#(#arg_names : #arg_types,)*], Self, #interface_name, #last_arg_variadic);
            },
            (false, true) => quote! {
                __wsdom_load_ts_macro::expand_method!(constructor @ #method_name_ident, [], [#(#arg_names : #arg_types,)*], #ret, #interface_name, #last_arg_variadic);
            },
            (true, false) => quote! {
                __wsdom_load_ts_macro::expand_method!(self @ #method_name_ident, [#method_generics], [#(#arg_names : #arg_types,)*], #ret, #method_name_str, #last_arg_variadic);
            },
            (false, false) => {
                let function = format!("{}.{}", interface_name, method_name_str);
                quote! {
                    __wsdom_load_ts_macro::expand_method!(free @ #method_name_ident, [#method_generics], [#(#arg_names : #arg_types,)*], #ret, #function, #last_arg_variadic);
                }
            }
        })
    }
    fn make_field_code(
        &self,
        interface_name: &'_ str,
        field: &Field<'_>,
        on_instance: bool,
    ) -> Option<TokenStream> {
        let field_name_str = match &field.name {
            crate::parser::field::FieldName::Name(name) => *name,
            crate::parser::field::FieldName::Wildcard { .. } => return None,
        };
        let mut ty = self.simplify_type(field.ty.to_owned());
        if field.optional {
            ty = SimplifiedType {
                name: "__translate_nullable",
                args: vec![ty],
            };
        }
        let ty_name = ty.name;
        let ty_tokens = self.convert_type(ty);
        let field_name_snake_case = to_snake_case(field_name_str);

        let getter_name_ident = new_ident_safe(&format!("get_{field_name_snake_case}"));
        let setter_name_ident = new_ident_safe(&format!("set_{field_name_snake_case}"));
        let setter_ty_tokens = if self.classes.contains(ty_name) {
            quote! {& #ty_tokens}
        } else {
            quote! {&dyn __wsdom_load_ts_macro::ToJs< #ty_tokens >}
        };
        Some(if on_instance {
            quote! {
                __wsdom_load_ts_macro::expand_field_getter_setter!(self @ #getter_name_ident, #ty_tokens, #setter_name_ident, #setter_ty_tokens, #field_name_str);
            }
        } else {
            quote! {
                __wsdom_load_ts_macro::expand_field_getter_setter!(browser @ #getter_name_ident, #ty_tokens, #setter_name_ident, #setter_ty_tokens, #field_name_str, #interface_name);
            }
        })

        // let getter = {
        //     if on_instance {
        //         quote! {
        //             pub fn #getter_name_ident (&self) -> #ty_tokens {
        //                 __wsdom_load_ts_macro::JsCast::unchecked_from_js(
        //                     __wsdom_load_ts_macro::JsObject::js_get_field(self.as_ref(), &#field_name_str)
        //                 )
        //             }
        //         }
        //     } else {
        //         quote! {
        //             pub fn #getter_name_ident (browser: &__wsdom_load_ts_macro::Browser) -> #ty_tokens {
        //                 __wsdom_load_ts_macro::JsCast::unchecked_from_js(
        //                     browser.get_field(&__wsdom_load_ts_macro::RawCodeImmediate( #interface_name ), &#field_name_str)
        //                 )
        //             }
        //         }
        //     }
        // };
        // let setter = (!field.readonly).then(|| {
        //             let ty_tokens = if self.classes.contains(ty_name) {
        //                 quote! {& #ty_tokens}
        //             }
        //             else {
        //                 quote! {&dyn __wsdom_load_ts_macro::ToJs< #ty_tokens >}
        //             };
        //             if on_instance {
        //                 quote!{
        //                     pub fn #setter_name_ident (&self, value: #ty_tokens) {
        //                         __wsdom_load_ts_macro::JsObject::js_set_field(self.as_ref(), &#field_name_str, __wsdom_load_ts_macro::UpcastWorkaround::new(value).cast())
        //                     }
        //                 }
        //             }
        //             else {
        //                 quote!{
        //                     pub fn #setter_name_ident (browser: &__wsdom_load_ts_macro::Browser, value: #ty_tokens) {
        //                         browser.set_field(&__wsdom_load_ts_macro::RawCodeImmediate( #interface_name ), &#field_name_str, value)
        //                     }
        //                 }
        //             }
        //         });
        // Some(quote! {
        //     #getter
        //     #setter
        // })
    }
    fn make_getter_code(
        &self,
        interface_name: &'_ str,
        getter: &Getter<'_>,
        on_instance: bool,
    ) -> Option<TokenStream> {
        if !on_instance {
            todo!("getter {} on constructor {}", getter.name, interface_name);
        }
        let field_name_str = getter.name;
        let getter_name_ident = new_ident_safe(&format!("get_{}", to_snake_case(field_name_str)));
        let ret = self.convert_type(self.simplify_type(getter.ret.to_owned()));
        Some(quote! {
            pub fn #getter_name_ident (&self) -> #ret {
                __wsdom_load_ts_macro::JsCast::unchecked_from_js(
                    __wsdom_load_ts_macro::JsObject::js_get_field(self.as_ref(), &#field_name_str)
                )
            }
        })
    }
    fn make_setter_code(
        &self,
        interface_name: &'_ str,
        setter: &Setter<'_>,
        on_instance: bool,
    ) -> Option<TokenStream> {
        if !on_instance {
            todo!("setter {} on constructor {}", setter.name, interface_name);
        }
        let field_name_str = setter.name;
        let setter_name_ident = new_ident_safe(&format!("set_{}", to_snake_case(field_name_str)));
        let ty = self.simplify_type(setter.arg_ty.to_owned());
        let ty_name = ty.name;
        let ty_tokens = self.convert_type(ty);
        let ty_tokens = if self.classes.contains(ty_name) {
            quote! {& #ty_tokens}
        } else {
            quote! {&dyn __wsdom_load_ts_macro::ToJs< #ty_tokens >}
        };

        Some(quote! {
            pub fn #setter_name_ident (&self, value: #ty_tokens) {
                __wsdom_load_ts_macro::JsObject::js_set_field(self.as_ref(), &#field_name_str, __wsdom_load_ts_macro::UpcastWorkaround::new( value ).cast() )
            }
        })
    }
}
