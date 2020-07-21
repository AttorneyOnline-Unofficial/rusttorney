use darling::{
    ast::Data,
    FromDeriveInput
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(command))]
struct CommandOps {
    ident: syn::Ident,
    data: Data<syn::Variant, ()>
}

#[proc_macro_derive(Command, attributes(command))]
pub fn command_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let CommandOps { ident: _, data } = CommandOps::from_derive_input(&input).unwrap();
    let _vars = match data {
        Data::Enum(vars) => vars,
        _ => return str_as_compile_error("`Command` macro might be used only with enums"),
    };
    // eprint!("{:#?}", vars);
    Default::default()
}

fn str_as_compile_error(s: &str) -> TokenStream {
    {quote!{ compile_error!(#s); }}.into()
}
