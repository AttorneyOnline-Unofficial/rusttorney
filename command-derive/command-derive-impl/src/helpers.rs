use syn::{parse::Parse, parse_str, Ident, Lit, Meta, MetaList, NestedMeta, Path};

type ParseRes<T> = Result<Option<T>, String>;

/// Parses this: `#key = "<val>"`
#[derive(Clone, Copy)]
struct ParseAssign<'a>(&'a str);

impl<'a> ParseAssign<'a> {
    /// Returns ParseErr::Ignore on `#key` mismatch
    /// and Parse::Fatal if "value" is not syn::LitStr
    fn parse_str(self, value: &NestedMeta) -> ParseRes<String> {
        let name_val = match value {
            NestedMeta::Meta(Meta::NameValue(name_val)) => name_val,
            _ => return Ok(None),
        };
        if !name_val.path.is_ident(self.0) {
            return Ok(None);
        }
        match &name_val.lit {
            Lit::Str(s) => Ok(Some(s.value())),
            other => Err(format!(
                "Expected string literal in `key = value` assignment, found {:?}",
                other
            )),
        }
    }
    /// Returns Ok(None) on `#key` mismatch
    /// and Err(_) if parsing of `R` fails
    fn parse_arg<T: Parse>(self, value: &NestedMeta) -> ParseRes<T> {
        self.parse_str(value)?
            .map(|s| parse_str(&s).map_err(|err| err.to_string()))
            .transpose()
    }
}

#[derive(Default)]
pub(crate) struct VariantOpts {
    pub code: Option<String>,
    pub handle: Option<Ident>,
}

impl VariantOpts {
    pub(crate) fn parse_from_meta(self, value: &Meta) -> Result<Self, String> {
        let meta_list = match value {
            Meta::List(meta_list) => meta_list,
            _ => return Ok(self),
        };
        let MetaList { path, nested, .. } = meta_list;
        if !path.is_ident("command") {
            return Ok(self);
        }
        let mut nested_it = nested.iter();

        nested_it.try_fold(self, |status, nested| -> Result<VariantOpts, String> {
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
                ) => Ok(VariantOpts { handle, code }),
                (handle @ Some(_), None, VariantOpts { handle: None, code }) => {
                    Ok(VariantOpts { handle, code })
                }
                (None, code @ Some(_), VariantOpts { handle, code: None }) => {
                    Ok(VariantOpts { handle, code })
                }
                (None, None, VariantOpts { handle, code }) => Ok(VariantOpts { handle, code }),
                _ => Err("Keys in `#[command(...)]` can't be repeated".into()),
            }
        })
    }
}

#[derive(Default)]
pub(crate) struct HandlerOpt {
    pub handler: Option<Path>,
}

impl HandlerOpt {
    pub(crate) fn parse_from_meta(self, value: &Meta) -> Result<Self, String> {
        let meta_list = match value {
            Meta::List(meta_list) => meta_list,
            _ => return Ok(self),
        };
        let MetaList { path, nested, .. } = meta_list;
        if !path.is_ident("command") {
            return Ok(self);
        }
        let mut nested_it = nested.iter();

        nested_it.try_fold(self, |status, nested| -> Result<HandlerOpt, String> {
            let handler_opt = ParseAssign("handler").parse_arg(nested)?;

            match (handler_opt, status) {
                (handler, HandlerOpt { handler: None }) => Ok(HandlerOpt { handler }),
                (None, HandlerOpt { handler }) => Ok(HandlerOpt { handler }),
                _ => Err("`handler` specified multiply times".into()),
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
