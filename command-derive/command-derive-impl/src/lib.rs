use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Fields, Member, ItemEnum, ItemStruct, parse_macro_input, Variant};

#[derive(FromMeta)]
struct VariantCode {
    code: String,
}

#[derive(FromMeta)]
struct VariantSkip {
    #[darling(rename = "skip")]
    _skip: (),
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
            .any(|meta| VariantSkip::from_meta(meta).is_ok())
        {
            continue;
        }
        let mut codes_iter = metas
            .into_iter()
            .filter_map(|meta| VariantCode::from_meta(&meta).ok())
            .fuse();
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
                let named_fields_piece: Vec<_> = unnamed
                    .unnamed
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(i, _)| format_ident!("x{}", i))
                    .collect();
                let idx_fields_piece: Vec<_> = unnamed
                    .unnamed
                    .into_iter()
                    .enumerate()
                    .map(|(i, _)| Member::Unnamed(i.into()))
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
