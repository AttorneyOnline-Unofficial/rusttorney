use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use core::convert::TryFrom;
use syn::{
    Fields, Member, Meta, ItemEnum, ItemStruct, NestedMeta, parse_macro_input, Variant,
    // parse::{self, Parse, ParseStream}
};

struct VariantCode {
    code: String,
}

enum ParseErr {
    Fatal(&'static str),
    Ignore
}

impl TryFrom<&syn::Meta> for VariantCode {
    type Error = ParseErr;

    fn try_from(value: &syn::Meta) -> Result<Self, Self::Error> {
        let meta_list = match *value {
            Meta::List(ref meta_list) => meta_list,
            _ => return Err(ParseErr::Ignore)
        };
        let syn::MetaList {
            ref path,
            ref nested,
            ..
        } = *meta_list;
        if !path.is_ident("command") {
            return Err(ParseErr::Ignore);
        }
        let mut nested_it = nested.iter().fuse();
        let code = match (nested_it.next(), nested_it.next()) {
            (Some(NestedMeta::Meta(Meta::NameValue(name_val))), None) => {
                if !name_val.path.is_ident("code") {
                    return Err(ParseErr::Ignore)
                }
                match name_val.lit {
                    syn::Lit::Str(ref s) => s.value(),
                    _ => return Err(ParseErr::Fatal(
                        r#"Only string literal allowed as value in #[command(code = "LIT")]"#
                    )),
                }
            },
            _ => return Err(ParseErr::Ignore)
        };
        Ok(VariantCode { code })
    }
}

// Validates this: `#[command(skip)]`
struct VariantSkip;

impl TryFrom<&syn::Meta> for VariantSkip {
    type Error = ();

    fn try_from(value: &syn::Meta) -> Result<Self, Self::Error> {
        let meta_list = match *value {
            Meta::List(ref meta_list) => meta_list,
            _ => return Err(())
        };
        let syn::MetaList {
            ref path,
            ref nested,
            ..
        } = *meta_list;
        if !path.is_ident("command") {
            return Err(());
        }
        let mut nested_it = nested.iter().fuse();
        match (nested_it.next(), nested_it.next()) {
            (Some(NestedMeta::Meta(Meta::Path(path))), None) => if !path.is_ident("skip") {
                return Err(())
            },
            _ => return Err(())
        }
        Ok(Self)
    }
}

#[proc_macro_derive(Command, attributes(command))]
pub fn command_derive(input: TokenStream) -> TokenStream {
    let ItemEnum { ident: enum_ident, variants: vars_punct, .. } = parse_macro_input!(input as ItemEnum);
    let vars: Vec<_> = vars_punct.into_iter().collect();
    let mut var_idents = Vec::with_capacity(vars.len());
    let mut codes = Vec::with_capacity(vars.len());
    let mut patterns = Vec::with_capacity(vars.len());
    let mut named_fields = Vec::with_capacity(vars.len());
    let mut idx_fields = Vec::with_capacity(vars.len());
    for var in vars {
        let Variant {
            attrs,
            ident,
            fields,
            ..
        } = var;
        let metas: Vec<_> = attrs
            .into_iter()
            .filter_map(|attr| attr.parse_meta().ok())
            .collect();
        if metas
            .iter()
            .any(|meta| VariantSkip::try_from(meta).is_ok())
        {
            continue;
        }
        let parse_codes: Vec<_> = metas
            .into_iter()
            .map(|meta| VariantCode::try_from(&meta))
            .collect();
        if let Some(Err(ParseErr::Fatal(err))) = parse_codes.iter().find(|res| matches!(res, Err(ParseErr::Fatal(_)))) {
            return str_as_compile_error(err);
        }
        let mut codes_iter = parse_codes.into_iter()
            .filter_map(|res| res.ok());
        let code = match (codes_iter.next(), codes_iter.next()) {
            (Some(var_code), None) => var_code.code,
            _ => {
                return str_as_compile_error(&format!(
                    concat!(
                        r#"Variant {}::{} does not have exactly one "#,
                        r#"attribute in a form of `#[command(code = "CODE")]`"#
                    ),
                    enum_ident, ident
                ))
            }
        };
        let (named_fields_piece, idx_fields_piece, pattern) = match fields {
            Fields::Named(named) => {
                let named_iter: Vec<_> = named
                    .named
                    .into_iter()
                    .map(|field| field.ident.expect("Variant is guaranteed to be named"))
                    .collect();
                let idx_fields_piece: Vec<_> =
                    named_iter.iter().cloned().map(Member::Named).collect();
                let pattern: TokenStream2 = quote! {
                    #ident {#(
                        #named_iter,
                    )*}
                };
                (named_iter, idx_fields_piece, pattern)
            }
            Fields::Unnamed(unnamed) => {
                let field_idxs: Vec<_> = unnamed
                    .unnamed
                    .into_iter()
                    .enumerate()
                    .map(|(i, _)| i)
                    .collect();
                let named_fields_piece: Vec<_> = field_idxs.iter()
                    .map(|i| format_ident!("x{}", i))
                    .collect();
                let idx_fields_piece: Vec<_> = field_idxs.into_iter()
                    .map(|i| Member::Unnamed(i.into()))
                    .collect();
                let pattern: TokenStream2 = quote! {
                    #ident ( #(#named_fields_piece,)* )
                };
                (named_fields_piece, idx_fields_piece, pattern)
            }
            Fields::Unit => (vec![], vec![], quote! { #ident }),
        };
        var_idents.push(ident);
        codes.push(code);
        patterns.push(pattern);
        named_fields.push(named_fields_piece);
        idx_fields.push(idx_fields_piece);
    }
    (quote!{
impl ::command_derive::Command for #enum_ident {
    fn ident(&self) -> &'static str {
        match self {
            #(
                #enum_ident::#var_idents { .. } => #codes,
            )*
                _ => panic!("`ident()` on skipped variant")
        }
    }

    fn extract_args(&self) -> Vec<String> {
        match self {
            #(
                #enum_ident::#patterns => vec![#(#named_fields.to_string(),)*],
            )*
                _ => panic!("`extract_args()` on skipped variant")
        }
    }

    fn from_protocol<I, S>(code: &str, args: I) -> Result<Self, ::anyhow::Error>
    where
        I: Iterator<Item = S>,
        S: AsRef<str>
    {
        let mut args = args.map(Ok).chain(::std::iter::from_fn(|| Some(Err(::anyhow::anyhow!("Not enough args")))));

        let res = match code {
            #(
                #codes => #enum_ident::#var_idents{#(
                    #idx_fields: args.next().unwrap()?.as_ref().parse().map_err(|e| ::anyhow::anyhow!("{}", e))?,
                )*},
            )*
            code => return Err(::anyhow::anyhow!("Unknown command code: {}", code))
        };
        // FIX: don't allocate any errors, when everything is ok
        if args.next().unwrap().is_ok() {
            return Err(::anyhow::anyhow!("Too many args"));
        }
        Ok(res)
    }
}
    }).into()
}

// Derives `impl<S: AsRef<str>> ::core::iter::FromIterator<S> for Result<Self, anyhow::Error>`
#[proc_macro_derive(FromStrIter)]
pub fn from_str_iter_derive(input: TokenStream) -> TokenStream {
    let ItemStruct { ident: struct_ident, fields, .. } = parse_macro_input!(input as ItemStruct);
    let field_names: Vec<_> = match fields {
        Fields::Named(named) => named.named.into_iter()
            .map(|field| Member::Named(field.ident.expect("Fields are guaranteed to be named")))
            .collect(),
        Fields::Unnamed(unnamed) => unnamed.unnamed.into_iter()
            .enumerate()
            .map(|(i, _)| Member::Unnamed(i.into()))
            .collect(),
        Fields::Unit => vec![]
    };

    (quote!{
impl ::command_derive::FromStrIter for #struct_ident {
    type Error = ::anyhow::Error;

    fn from_str_iter<I, S>(mut it: I) -> Result<Self, ::anyhow::Error>
    where
        S: AsRef<str>,
        I: Iterator<Item=S>
    {
        let on_err = || ::anyhow::anyhow!("Not enough args");
        Ok(Self {#(
            #field_names:
                it.next()
                    .ok_or_else(on_err)?
                    .as_ref()
                    .parse()
                    .map_err(|e| ::anyhow::anyhow!("{}", e))?,
        )*})
    }
}
    }).into()
}

fn str_as_compile_error(s: &str) -> TokenStream {
    {
        quote! { compile_error!(#s); }
    }
    .into()
}
