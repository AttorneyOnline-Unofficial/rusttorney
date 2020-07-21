use darling::{ast::Data, FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Ident, Fields, Member, Variant};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(command))]
struct CommandOps {
    ident: Ident,
    data: Data<Variant, ()>,
}

#[derive(Debug, FromMeta)]
struct VariantCode {
    code: String,
}

#[proc_macro_derive(Command, attributes(command))]
pub fn command_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let CommandOps {
        ident: enum_ident,
        data,
    } = CommandOps::from_derive_input(&input).unwrap();
    let vars = match data {
        Data::Enum(vars) => vars,
        _ => return str_as_compile_error("`Command` macro might be used only with enums"),
    };
    let mut var_idents = Vec::with_capacity(vars.len());
    let mut codes = Vec::with_capacity(vars.len());
    let mut patterns = Vec::with_capacity(vars.len());
    let mut named_fields = Vec::with_capacity(vars.len());
    let mut idx_fields = Vec::with_capacity(vars.len());
    for var in vars {
        let Variant { attrs, ident, fields, .. } = var;
        let mut codes_iter = attrs
            .iter()
            .filter_map(|attr| attr.parse_meta().ok())
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
                let named_iter: Vec<_> = named.named
                    .into_iter()
                    .map(|field| field.ident.expect("Variant is guaranteed to be named"))
                    .collect();
                let idx_fields_piece: Vec<_> = named_iter
                    .iter().cloned()
                    .map(|ident| Member::Named(ident))
                    .collect();
                let pattern: TokenStream2 = quote!{
                    #ident {#(
                        #named_iter,
                    )*}
                };
                (named_iter, idx_fields_piece, pattern)
            },
            Fields::Unnamed(unnamed) => {
                let named_fields_piece: Vec<_> = unnamed.unnamed.iter().cloned()
                    .enumerate()
                    .map(|(i, _)| format_ident!("x{}", i))
                    .collect();
                let idx_fields_piece: Vec<_> = unnamed.unnamed.into_iter()
                    .enumerate()
                    .map(|(i, _)| Member::Unnamed(i.into()))
                    .collect();
                let pattern: TokenStream2 = quote!{
                    #ident ( #(#named_fields_piece,)* )
                };
                (named_fields_piece, idx_fields_piece, pattern)
            }
            Fields::Unit => (vec![], vec![], (quote!{ #ident }).into())
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
        }
    }

    fn extract_args(&self) -> Vec<String> {
        match self {
            #(
                #enum_ident::#patterns => vec![#(#named_fields.to_string(),)*],
            )*
        }
    }

    fn from_protocol<I>(code: &str, args: I) -> Result<Self, ::anyhow::Error> where I: Iterator<Item = String> {
        let mut args = args.map(Ok).chain(::std::iter::from_fn(|| Some(Err(::anyhow::anyhow!("Not enough args")))));

        let res = match code {
            #(
                #codes => #enum_ident::#var_idents{#(
                    #idx_fields: args.next().unwrap()?.parse().map_err(|e| ::anyhow::anyhow!("{}", e))?,
                )*},
            )*
            code => return Err(::anyhow::anyhow!("Unknown command code: {}", code))
        };
        if args.next().unwrap().is_ok() {
            return Err(::anyhow::anyhow!("Too much args"));
        }
        Ok(res)
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
