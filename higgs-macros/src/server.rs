use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{parse_macro_input, parse_quote, Expr, ItemFn, Token};

struct ServerArgs {
    permission: Option<Expr>,
    auth: bool,
}

impl Parse for ServerArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                permission: None,
                auth: false,
            });
        }

        let ident: syn::Ident = input.parse()?;
        if ident == "auth" {
            if !input.is_empty() {
                return Err(input.error("unexpected trailing tokens after `auth`"));
            }
            return Ok(Self {
                permission: None,
                auth: true,
            });
        }

        if ident != "permission" {
            return Err(syn::Error::new_spanned(
                ident,
                "unsupported argument; expected `auth` or `permission = <expr>`",
            ));
        }
        input.parse::<Token![=]>()?;
        let permission: Expr = input.parse()?;

        if !input.is_empty() {
            return Err(input.error("unexpected trailing tokens in server macro arguments"));
        }

        Ok(Self {
            permission: Some(permission),
            auth: false,
        })
    }
}

/// Expand `#[higgs_macros::server]` into a Leptos `#[server]` fn wrapped with operation attribution.
pub fn expand_server(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ServerArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    if let Some(permission) = &args.permission {
        return syn::Error::new_spanned(
            permission,
            "`permission = …` is not available in higgs-macros v0.1.0; enable after gauge lands (higgs v0.3.0)",
        )
        .to_compile_error()
        .into();
    }

    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "#[higgs_macros::server] can only be used on async functions",
        )
        .to_compile_error()
        .into();
    }

    let mut server_attrs = Vec::new();
    let mut other_attrs = Vec::new();

    for attr in &input_fn.attrs {
        if attr.path().is_ident("server") {
            server_attrs.push(attr.clone());
        } else {
            other_attrs.push(attr.clone());
        }
    }

    if server_attrs.is_empty() {
        server_attrs.push(parse_quote!(#[server]));
    }

    let body = &input_fn.block;
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let fn_name_str_lit = syn::LitStr::new(&fn_name_str, proc_macro2::Span::call_site());

    let auth_prelude = if args.auth {
        quote! {
            let _auth_session = ::higgs::require_session().await?;
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #(#other_attrs)*
        #(#server_attrs)*
        #vis #sig {
            ::higgs::with_operation(#fn_name_str_lit, async move {
                #auth_prelude
                #body
            }).await
        }
    };

    expanded.into()
}
