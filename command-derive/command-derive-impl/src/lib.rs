use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{spanned::Spanned, parse_macro_input, Fields, ItemEnum, ItemStruct, Member, Variant};

mod helpers;
use helpers::{CommandMarker, HandlerOpt, VariantOpts};

#[proc_macro_derive(Command, attributes(command))]
pub fn command_derive(input: TokenStream) -> TokenStream {
    let ItemEnum {
        ident: enum_ident,
        variants: vars_punct,
        attrs,
        ..
    } = parse_macro_input!(input as ItemEnum);

    let handler_opt_res = attrs
        .into_iter()
        .filter_map(|attr| attr.parse_meta().ok())
        .try_fold(HandlerOpt::default(), |handler_opt, meta| {
            handler_opt.parse_from_meta(&meta)
        });
    let handler_opt = match handler_opt_res {
        Ok(HandlerOpt { handler }) => handler,
        Err((err, span)) => return str_as_compile_error(&err, span),
    };

    let vars: Vec<_> = vars_punct.into_iter().collect();
    let mut var_idents = Vec::with_capacity(vars.len());
    let mut codes = Vec::with_capacity(vars.len());
    let mut handles = Vec::with_capacity(if handler_opt.is_some() { vars.len() } else { 0 });
    let mut patterns = Vec::with_capacity(vars.len());
    let mut named_fields = Vec::with_capacity(vars.len());
    let mut named_fields_to_str = Vec::with_capacity(vars.len());
    let mut idx_fields = Vec::with_capacity(vars.len());
    let mut read_fields = Vec::with_capacity(vars.len());
    for var in vars {
        let var_span = var.span();
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
        if let Some(meta) = metas
            .iter()
            .find(|meta| CommandMarker("skip").validate(meta).is_ok())
        {
            return str_as_compile_error("#[command(skip)] was hard-deprecated, sorry", meta.span());
        }
        let var_opts_res = metas
            .into_iter()
            .try_fold(VariantOpts::default(), |var_opts, meta| {
                var_opts.parse_from_meta(&meta)
            });
        let (code, handle) = match var_opts_res {
            Ok(VariantOpts { code: None, .. }) => {
                return str_as_compile_error(&format!(
                    "No `code` parameter on {}::{}",
                    enum_ident, ident
                ), var_span)
            }
            Ok(VariantOpts { handle: None, .. }) if handler_opt.is_some() => {
                return str_as_compile_error(&format!(
                    "No `handle` parameter on {}::{}",
                    enum_ident, ident
                ), var_span)
            }
            Ok(VariantOpts {
                code: Some(code),
                handle,
            }) => (code, handle),
            Err((err, span)) => return str_as_compile_error(&err, span),
        };
        let (
            named_fields_piece,
            named_fields_to_str_piece,
            read_fields_piece,
            idx_fields_piece,
            pattern,
        ) = match fields {
            Fields::Named(named) => {
                let named_iter: Vec<_> = named
                    .named
                    .into_iter()
                    .map(|field| {
                        (
                            field.ident.expect("Variant is guaranteed to be named"),
                            field
                                .attrs
                                .into_iter()
                                .filter_map(|attr| attr.parse_meta().ok())
                                .any(|meta| CommandMarker("flatten").validate(&meta).is_ok()),
                        )
                    })
                    .collect();
                let named_fields_piece: Vec<_> =
                    named_iter.iter().map(|(ident, _)| ident.clone()).collect();
                let idx_fields_piece: Vec<_> = named_iter
                    .iter()
                    .cloned()
                    .map(|(ident, _)| Member::Named(ident))
                    .collect();
                let (named_fields_to_str_piece, read_fields_piece): (Vec<_>, Vec<_>) = named_iter.iter()
                    .map(|(ident, flatten)| match flatten {
                        false => (
                            quote!{ res.push(#ident.to_string()) },
                            quote!{ args.next().ok_or_else(|| ::anyhow::anyhow!("Not enough args"))?.as_ref().parse().map_err(|e| ::anyhow::anyhow!("{}", e))? }
                        ),
                        true => (
                            quote!{ res.extend(#ident.into_iter()) },
                            quote!{ ::command_derive::FromStrIter::from_str_iter(&mut args)? }
                        )
                    })
                    .unzip();
                let pattern = quote! {
                    #ident {#(#named_fields_piece,)*}
                };
                (
                    named_fields_piece,
                    named_fields_to_str_piece,
                    read_fields_piece,
                    idx_fields_piece,
                    pattern,
                )
            }
            Fields::Unnamed(unnamed) => {
                let field_num = unnamed.unnamed.len();
                let flatten_flags = unnamed.unnamed.into_iter().map(|field| {
                    field
                        .attrs
                        .into_iter()
                        .filter_map(|attr| attr.parse_meta().ok())
                        .any(|meta| CommandMarker("flatten").validate(&meta).is_ok())
                });
                let named_fields_piece: Vec<_> =
                    (0..field_num).map(|i| format_ident!("x{}", i)).collect();
                let (named_fields_to_str_piece, read_fields_piece): (Vec<_>, Vec<_>) = (0..field_num)
                    .zip(flatten_flags)
                    .map(|(i, flatten)| {
                        let fld_ident = format_ident!("x{}", i);
                        match flatten {
                            false => (
                                quote!{ res.push(#fld_ident.to_string()) },
                                quote!{ args.next().ok_or_else(|| ::anyhow::anyhow!("Not enough args"))?.as_ref().parse().map_err(|e| ::anyhow::anyhow!("{}", e))? }
                            ),
                            true => (
                                quote!{ res.extend(#fld_ident.into_iter()) },
                                quote!{ ::command_derive::FromStrIter::from_str_iter(&mut args)? }
                            )
                        }
                    })
                    .unzip();
                let idx_fields_piece: Vec<_> =
                    (0..field_num).map(|i| Member::Unnamed(i.into())).collect();
                let pattern = quote! {
                    #ident (#(#named_fields_piece,)*)
                };
                (
                    named_fields_piece,
                    named_fields_to_str_piece,
                    read_fields_piece,
                    idx_fields_piece,
                    pattern,
                )
            }
            Fields::Unit => (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                quote! { #ident },
            ),
        };
        var_idents.push(ident);
        codes.push(code);
        if let Some(handle) = handle {
            handles.push(handle)
        }
        named_fields.push(named_fields_piece);
        named_fields_to_str.push(named_fields_to_str_piece);
        read_fields.push(read_fields_piece);
        idx_fields.push(idx_fields_piece);
        patterns.push(pattern);
    }
    let mut res = quote! {
    impl ::command_derive::Command for #enum_ident {
        fn ident(&self) -> &'static str {
            match self {
                #(
                    #enum_ident::#var_idents { .. } => #codes,
                )*
            }
        }

        fn extract_args(&self) -> Vec<String> {
            use ::core::iter::IntoIterator;

            let mut res = Vec::new();
            match self {
                #(
                    #enum_ident::#patterns => {
                        #(#named_fields_to_str;)*
                    },
                )*
            }
            res
        }

        fn from_protocol<I, S>(code: &str, mut args: I) -> Result<Self, ::anyhow::Error>
        where
            I: Iterator<Item = S>,
            S: AsRef<str>
        {
            let res = match code {
                #(
                    #codes => #enum_ident::#var_idents{#(
                        #idx_fields: #read_fields,
                    )*},
                )*
                code => return Err(::anyhow::anyhow!("Unknown command code: {}", code))
            };
            if args.next().is_some() {
                return Err(::anyhow::anyhow!("Too many args"));
            }
            Ok(res)
        }
    }
    };
    if let Some(handler) = handler_opt {
        res.extend(quote!{
    impl #enum_ident {
        pub async fn handle<'a>(self, handler: &'a mut #handler) -> Result<(), ::anyhow::Error> {
            match self {
                #(
                    #enum_ident::#patterns => handler.#handles(#(#named_fields,)*).await,
                )*
            }
        }
    }
        });
    }
    res.into()
}

/// Derives `IntoIterator<IntoIter=Vec<String>::IntoIter>` for `&Self`
/// and `FromStrIter` for `Self`
/// Applyable for struct-s
#[proc_macro_derive(WithStrIter)]
pub fn with_str_iter_derive(input: TokenStream) -> TokenStream {
    let ItemStruct {
        ident: struct_ident,
        fields,
        ..
    } = parse_macro_input!(input as ItemStruct);
    let field_names: Vec<_> = match fields {
        Fields::Named(named) => named
            .named
            .into_iter()
            .map(|field| Member::Named(field.ident.expect("Fields are guaranteed to be named")))
            .collect(),
        Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .into_iter()
            .enumerate()
            .map(|(i, _)| Member::Unnamed(i.into()))
            .collect(),
        Fields::Unit => vec![],
    };

    (quote! {
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

    impl ::core::iter::IntoIterator for &#struct_ident {
        type Item = String;
        type IntoIter = <Vec<String> as ::core::iter::IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            vec![#(
                self.#field_names.to_string(),
            )*].into_iter()
        }
    }
        })
    .into()
}

fn str_as_compile_error(err: &str, span: proc_macro2::Span) -> TokenStream {
    {
        quote_spanned! {span=> compile_error!(#err); }
    }
    .into()
}
