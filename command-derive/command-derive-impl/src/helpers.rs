use proc_macro2::Span;
use syn::{
    parse::Parse, parse_str, spanned::Spanned, Ident, Lit, Meta, MetaList, NestedMeta, Path,
};

/// Parses this: `#key = "<val>"`
#[derive(Clone, Copy)]
struct ParseAssign<'a>(&'a str);

impl<'a> ParseAssign<'a> {
    /// Returns ParseErr::Ignore on `#key` mismatch
    /// and Parse::Fatal if "value" is not syn::LitStr
    fn parse_str(self, value: &NestedMeta) -> Result<Option<(String, Span)>, (String, Span)> {
        let name_val = match value {
            NestedMeta::Meta(Meta::NameValue(name_val)) => name_val,
            _ => return Ok(None),
        };
        if !name_val.path.is_ident(self.0) {
            return Ok(None);
        }
        match &name_val.lit {
            Lit::Str(lit) => Ok(Some((lit.value(), lit.span()))),
            other => Err((
                "Expected string literal in `key = value` assignment".into(),
                other.span(),
            )),
        }
    }
    /// Returns Ok(None) on `#key` mismatch
    /// and Err(_) if parsing of `R` fails
    fn parse_arg<T: Parse>(self, value: &NestedMeta) -> Result<Option<T>, (String, Span)> {
        self.parse_str(value)?
            .map(|(lit, span)| parse_str(&lit).map_err(move |err| (err.to_string(), span)))
            .transpose()
    }
}

#[derive(Default)]
pub(crate) struct VariantOpts {
    pub code: Option<String>,
    pub handle: Option<Ident>,
}

impl VariantOpts {
    pub(crate) fn parse_from_meta(self, value: &Meta) -> Result<Self, (String, Span)> {
        let meta_list = match value {
            Meta::List(meta_list) => meta_list,
            _ => return Ok(self),
        };
        let MetaList { path, nested, .. } = meta_list;
        if !path.is_ident("command") {
            return Ok(self);
        }
        let mut nested_it = nested.iter();

        nested_it.try_fold(self, |status, nested| /*-> Result<VariantOpts, String>*/ {
            let code_opt = ParseAssign("code").parse_str(nested)?;
            let handle_opt = ParseAssign("handle").parse_arg(nested)?;

            match (handle_opt, code_opt, status) {
                (
                    handle @ Some(_),
                    code @ Some(_),
                    VariantOpts {
                        handle: None,
                        code: None,
                    },
                ) => Ok(VariantOpts { handle, code: code.map(|(code, _)| code) }),
                (handle @ Some(_), None, VariantOpts { handle: None, code }) => {
                    Ok(VariantOpts { handle, code })
                }
                (None, code @ Some(_), VariantOpts { handle, code: None }) => {
                    Ok(VariantOpts { handle, code: code.map(|(code, _)| code) })
                }
                (None, None, VariantOpts { handle, code }) => Ok(VariantOpts { handle, code }),
                _ => Err(("Keys in `#[command(...)]` can't be repeated".into(), nested.span())),
            }
        })
    }
}

#[derive(Default)]
pub(crate) struct HandlerOpt {
    pub handler: Option<Path>,
}

impl HandlerOpt {
    pub(crate) fn parse_from_meta(self, value: &Meta) -> Result<Self, (String, proc_macro2::Span)> {
        let meta_list = match value {
            Meta::List(meta_list) => meta_list,
            _ => return Ok(self),
        };
        let MetaList { path, nested, .. } = meta_list;
        if !path.is_ident("command") {
            return Ok(self);
        }
        let mut nested_it = nested.iter();

        nested_it.try_fold(self, |status, nested| /*-> Result<HandlerOpt, String>*/ {
            let handler_opt = ParseAssign("handler").parse_arg(nested)?;

            match (handler_opt, status) {
                (handler, HandlerOpt { handler: None }) => Ok(HandlerOpt { handler }),
                (None, HandlerOpt { handler }) => Ok(HandlerOpt { handler }),
                (Some(handler), _) => Err(("`handler` specified multiply times".into(), handler.span())),
            }
        })
    }
}

/// Validates this: `#[command(#0)]`
#[derive(Clone, Copy)]
pub(crate) struct CommandMarker(pub &'static str);

impl CommandMarker {
    pub fn validate(self, value: &Meta) -> Result<Self, ()> {
        let meta_list = match *value {
            Meta::List(ref meta_list) => meta_list,
            _ => return Err(()),
        };
        let MetaList {
            ref path,
            ref nested,
            ..
        } = *meta_list;
        if !path.is_ident("command") {
            return Err(());
        }
        let mut nested_it = nested.iter().fuse();
        match (nested_it.next(), nested_it.next()) {
            (Some(NestedMeta::Meta(Meta::Path(path))), None) => {
                if !path.is_ident(self.0) {
                    return Err(());
                }
            }
            _ => return Err(()),
        }
        Ok(self)
    }
}
