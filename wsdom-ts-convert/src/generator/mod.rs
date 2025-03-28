mod alias;
mod class;
mod function;
mod signature;
mod types;
mod util;
mod var;

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    generator::util::iter_dedupe_consecutive,
    parser::{
        comment::WithComment,
        declare_class::DeclareClass,
        declare_var::DeclareVar,
        generic::GenericsDeclaration,
        interface::Interface,
        item::Item,
        member::Member,
        method::{Method, MethodName},
        ts_type::{NamedType, TsType},
    },
};

#[derive(Default)]
struct Context<'a> {
    interfaces: HashMap<&'a str, Interface<'a>>,
    classes: HashSet<&'a str>,
    inhr_graph: HashMap<&'a str, (GenericsDeclaration<'a>, Vec<NamedType<'a>>)>,
}

pub(crate) fn generate_all<'a>(
    dts: &[WithComment<'a, Item<'a>>],
    dts_for_inhr: &[WithComment<'a, Item<'a>>],
) -> TokenStream {
    let mut generated_code = Vec::<TokenStream>::new();
    let mut interfaces = HashMap::new();
    let mut inhr_graph = HashMap::new();
    for item in dts.into_iter().chain(dts_for_inhr.into_iter()) {
        match &item.data {
            Item::Interface(interface) => {
                let interface = interface.clone();
                let (_, extends) = inhr_graph
                    .entry(interface.name)
                    .or_insert_with(|| (interface.generics.clone(), Vec::new()));
                for extend in &interface.extends {
                    match extend {
                        TsType::Named { ty } => {
                            extends.push(ty.to_owned());
                        }
                        _ => {}
                    }
                }
                interfaces.insert(interface.name, interface);
            }
            _ => {}
        }
    }
    let mut ctx = Context {
        interfaces,
        classes: HashSet::new(),
        inhr_graph,
    };

    let declare_vars = dts.iter().filter_map(|item| match &item.data {
        Item::DeclareVar(dv) => Some(dv),
        _ => None,
    });
    let declare_classlike_vars = declare_vars.clone().filter(|dv| starts_with_cap(dv.name));
    let declare_global_vars = declare_vars.filter(|dv| !starts_with_cap(dv.name));
    ctx.classes
        .extend(declare_classlike_vars.clone().map(|dv| dv.name));
    let declare_classes = dts.iter().filter_map(|item| match &item.data {
        Item::DeclareClass(c) => Some(c),
        _ => None,
    });
    for DeclareVar { name, ty } in declare_classlike_vars {
        let Some((decl_members, on_instance)) = ctx.get_members(ty) else {
            continue;
        };

        let constructor = decl_members.iter().find_map(|member| match &member.data {
            Member::Method(
                method @ Method {
                    name: MethodName::Constructor,
                    ret:
                        Some(
                            ret @ TsType::Named {
                                ty: NamedType { name, .. },
                            },
                        ),
                    ..
                },
            ) => Some((method, ret, name)),
            _ => None,
        });
        if let Some((method, ret, ret_name)) = constructor {
            if ret_name != name {
                generated_code.push(ctx.make_custom_constructor(name, &method.args, ret));
                continue;
            }
        }
        let iface = constructor
            .and_then(|(_, _, s)| ctx.interfaces.get(s))
            .or_else(|| (!on_instance).then(|| ctx.interfaces.get(name)).flatten())
            .map(Cow::Borrowed)
            .unwrap_or_else(|| {
                Cow::Owned(Interface {
                    extends: Default::default(),
                    generics: Default::default(),
                    members: Default::default(),
                    name,
                })
            });
        generated_code.push(ctx.make_class(&*iface, decl_members));
    }
    for DeclareClass {
        name,
        generics,
        members,
    } in declare_classes
    {
        generated_code.push(ctx.make_class(
            &Interface {
                extends: Default::default(),
                generics: generics.to_owned(),
                members: members.to_owned(),
                name: *name,
            },
            &[],
        ));
    }
    for DeclareVar { name, ty } in declare_global_vars {
        generated_code.push(ctx.make_global_var_getter(name, ty));
    }
    for ta in dts.iter().filter_map(|item| match &item.data {
        Item::TypeAlias(ta) => Some(ta),
        _ => None,
    }) {
        generated_code.push(ctx.make_type_alias(ta));
    }
    for df in iter_dedupe_consecutive(
        dts.iter().filter_map(|item| match &item.data {
            Item::DeclareFunction(df) => Some(df),
            _ => None,
        }),
        |df| Some(df.name),
    ) {
        generated_code.push(ctx.make_function(df));
    }

    for iface in dts.iter().filter_map(|item| match &item.data {
        Item::Interface(iface) if !ctx.classes.contains(iface.name) => Some(iface),
        _ => None,
    }) {
        generated_code.push(ctx.make_class(iface, &[]));
    }
    quote! {
        #(#generated_code)*
    }
}

fn starts_with_cap(s: &str) -> bool {
    s.chars()
        .next()
        .as_ref()
        .is_some_and(char::is_ascii_uppercase)
}
