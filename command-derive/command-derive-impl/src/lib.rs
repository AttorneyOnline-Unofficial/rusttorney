use darling::{ast::Data, FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(command))]
struct CommandOps {
    ident: syn::Ident,
    data: Data<syn::Variant, ()>,
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
    for var in vars {
        let mut codes = var
            .attrs
            .iter()
            .filter_map(|attr| attr.parse_meta().ok())
            .filter_map(|meta| VariantCode::from_meta(&meta).ok())
            .fuse();
        let _code = match (codes.next(), codes.next()) {
            (Some(code), None) => code,
            _ => {
                return str_as_compile_error(&format!(
                    concat!(
                        r#"Variant {}::{} does not have exactly one "#,
                        r#"attribute in a form of `#[command(code = "CODE")]`"#
                    ),
                    enum_ident, var.ident
                ))
            }
        };
        // eprintln!("{}.code = {:?}", var.ident, code);
    }
    // eprint!("{:#?}", vars);
    Default::default()
}

fn str_as_compile_error(s: &str) -> TokenStream {
    {
        quote! { compile_error!(#s); }
    }
    .into()
}
