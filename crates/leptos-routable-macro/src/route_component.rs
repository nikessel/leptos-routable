use proc_macro::TokenStream;
use quote::quote;
use syn::{
    FnArg, ItemFn, PatType, Type, Path as SynPath,
    Meta, Attribute,
};
use darling::{
    ast::NestedMeta,
    FromMeta, Error as DarlingError,
};

#[derive(Debug, Default, FromMeta)]
struct PathAttributeArgs {
    #[darling(default)]
    pub result: bool,
    #[darling(default)]
    pub redirect: Option<String>,
}

fn parse_path_attr(attrs: &[Attribute]) -> Option<PathAttributeArgs> {
    for attr in attrs {
        if attr.path().is_ident("path_param") {
            match &attr.meta {
                Meta::Path(_) => {
                    return Some(PathAttributeArgs::default());
                }
                Meta::List(list) => {
                    let meta = Meta::List(list.clone());
                    let res = PathAttributeArgs::from_meta(&meta);
                    match res {
                        Ok(args) => return Some(args),
                        Err(_) => {
                            return Some(PathAttributeArgs {
                                result: false,
                                redirect: None,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }
    None
}

fn has_query_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("query"))
}

pub fn route_component_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let fn_ast = match syn::parse::<ItemFn>(item) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
    };

    let variant_path = match parse_single_variant_path(attr) {
        Ok(vp) => vp,
        Err(e) => return e.write_errors().into(),
    };

    let hooking_func_name = crate::utils::build_registry_func_name(
        &variant_path_to_string(&variant_path),
    );

    let (param_stmts, param_idents) = match build_param_statements(&fn_ast) {
        Ok((stmts, idents)) => (stmts, idents),
        Err(e) => return e.to_compile_error().into(),
    };

    let fn_name = &fn_ast.sig.ident;
    let fn_vis = &fn_ast.vis;
    let fn_generics = &fn_ast.sig.generics;
    let fn_body = &fn_ast.block;
    let inputs = &fn_ast.sig.inputs;

    let hooking_func = quote! {
        #[::leptos::component]
        pub fn #hooking_func_name() -> impl ::leptos::IntoView {
            #(#param_stmts)*
            #fn_name(#(#param_idents),*)
        }
    };

    let original_fn = quote! {
        #[allow(non_snake_case)]
        #fn_vis fn #fn_name #fn_generics (#inputs) -> impl ::leptos::IntoView {
            #fn_body
        }
    };

    let expanded = quote! {
        #original_fn
        #hooking_func
    };

    crate::utils::format_generated_code(expanded).into()
}

fn parse_single_variant_path(attr: TokenStream) -> darling::Result<SynPath> {
    let list = NestedMeta::parse_meta_list(attr.into()).map_err(DarlingError::from)?;
    if list.is_empty() {
        return Err(DarlingError::custom(
            "Expected one path, e.g. `#[route_component(AppRouter::Foo)]`.",
        ));
    }
    if list.len() > 1 {
        return Err(DarlingError::custom(
            "Only one path is allowed in `#[route_component(...)]`.",
        ));
    }
    match &list[0] {
        NestedMeta::Meta(Meta::Path(p)) => Ok(p.clone()),
        _ => Err(DarlingError::custom(
            "Expected a single path like `AppRouter::Foo`.",
        )),
    }
}

fn variant_path_to_string(p: &SynPath) -> String {
    p.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("_")
}

fn build_param_statements(
    fn_ast: &ItemFn
) -> Result<(Vec<proc_macro2::TokenStream>, Vec<syn::Ident>), syn::Error> {
    let mut param_stmts = Vec::new();
    let mut param_idents = Vec::new();

    for arg in &fn_ast.sig.inputs {
        let FnArg::Typed(PatType { pat, attrs, ty, .. }) = arg else {
            return Err(syn::Error::new_spanned(
                arg,
                "Only typed parameters are supported.",
            ));
        };
        let param_ident = match &**pat {
            syn::Pat::Ident(pid) => pid.ident.clone(),
            _ => {
                return Err(syn::Error::new_spanned(
                    pat,
                    "Expected a simple identifier pattern.",
                ));
            }
        };

        if let Some(path_args) = parse_path_attr(attrs) {
            let parse_stmt = generate_path_parse_stmt(&param_ident, ty, &path_args)?;
            param_stmts.push(parse_stmt);
            param_idents.push(param_ident);
            continue;
        }

        if has_query_attr(attrs) {
            param_stmts.push(quote! {
                let #param_ident = {
                    ::leptos_router::hooks::use_query::<#ty>()
                };
            });
            param_idents.push(param_ident);
            continue;
        }

        return Err(syn::Error::new_spanned(
            arg,
            "Parameter must have either `#[path_param]` or `#[query]`.",
        ));
    }

    Ok((param_stmts, param_idents))
}

fn generate_path_parse_stmt(
    param_ident: &syn::Ident,
    param_type: &Type,
    path_args: &PathAttributeArgs,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    if check_if_memo_result(param_type) {
        // not implemented
        return Ok(quote! {
            let #param_ident = {
                compile_error!("Detecting and constructing MemoResult not implemented.")
            };
        });
    }

    if path_args.result {
        return Ok(quote! {
            let #param_ident = ::leptos_router::hooks::use_params::<#param_type>();
        });
    }

    if let Some(redirect_url) = &path_args.redirect {
        Ok(quote! {
            let __memo_res = ::leptos_router::hooks::use_params::<#param_type>();
            let #param_ident = match __memo_res.read() {
                Ok(val) => val,
                Err(_) => {
                    let nav = ::leptos_router::hooks::use_navigate();
                    nav(#redirect_url, Default::default());
                    return ::leptos::view! { <div></div> }.into_view();
                }
            };
        })
    } else {
        Ok(quote! {
            let __memo_res = ::leptos_router::hooks::use_params::<#param_type>();
            let #param_ident = match __memo_res.read() {
                Ok(val) => val,
                Err(_) => {
                    return ::leptos::view! { <div>Invalid param</div> }.into_view();
                }
            };
        })
    }
}

fn check_if_memo_result(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            return seg.ident == "MemoResult";
        }
    }
    false
}
