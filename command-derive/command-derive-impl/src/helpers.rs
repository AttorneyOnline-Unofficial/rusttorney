use std::convert::TryFrom;
use syn::{Meta, MetaList, NestedMeta};

pub(crate) struct VariantCode {
    pub code: String,
}

pub(crate) enum ParseErr {
    Fatal(&'static str),
    Ignore
}

// impl<T> std::iter::Product<Result<T, ParseErr>> for Result<T, ParseErr> {
//     fn product<I>(iter: I) -> Self
//     where
//         I: Iterator<Item = Self>
//     {
//         unimplemented!()
//     }
// }

impl TryFrom<&Meta> for VariantCode {
    type Error = ParseErr;

    fn try_from(value: &Meta) -> Result<Self, Self::Error> {
        let meta_list = match *value {
            Meta::List(ref meta_list) => meta_list,
            _ => return Err(ParseErr::Ignore)
        };
        let MetaList {
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

// Validates this: `#[command(#0)]`
#[derive(Clone, Copy)]
pub(crate) struct CommandMarker(pub &'static str);

impl CommandMarker {
    pub fn validate(self, value: &Meta) -> Result<Self, ()> {
        let meta_list = match *value {
            Meta::List(ref meta_list) => meta_list,
            _ => return Err(())
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
            (Some(NestedMeta::Meta(Meta::Path(path))), None) => if !path.is_ident(self.0) {
                return Err(())
            },
            _ => return Err(())
        }
        Ok(self)
    }
}
