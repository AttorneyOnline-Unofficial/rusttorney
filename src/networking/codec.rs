use crate::networking::aocommands::{ClientCommand, ServerCommand};
use crate::networking::Command;
use bytes::{Buf, BufMut, BytesMut};
use std::borrow::Cow;
use std::char::REPLACEMENT_CHARACTER;
use tokio_util::codec::{Decoder, Encoder};

const MAGIC_SEPARATOR: u8 = b'#';
const MAGIC_END: u8 = b'%';

impl Encoder<ServerCommand> for AOMessageCodec {
    type Error = anyhow::Error;

    fn encode(
        &mut self,
        item: ServerCommand,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let args_len = match item.extract_args() {
            Some(args) => args.iter().fold(0, |i, s| i + s.len() + 1),
            None => 0,
        };
        let ident = item.ident();
        #[rustfmt::skip]
            let reserve_len =
            // 2 - 8
            ident.len() +
                // #
                1 +
                // args_len is every arg + #
                args_len +
                // %
                1;
        dst.reserve(reserve_len);
        dst.put(ident.as_bytes());
        dst.put_u8(b'#');

        if let Some(args) = item.extract_args() {
            for arg in args {
                dst.put(arg.as_bytes());
                dst.put_u8(b'#');
            }
        }

        dst.put_u8(b'%');
        Ok(())
    }
}

fn ignore_ill_utf8(v: &[u8]) -> String {
    let str = String::from_utf8_lossy(&v);

    match str {
        Cow::Owned(mut own) => {
            own.retain(|c| c != REPLACEMENT_CHARACTER);
            own
        }
        Cow::Borrowed(brw) => brw.to_owned(),
    }
}

pub struct AOMessageCodec;

impl Decoder for AOMessageCodec {
    type Item = ClientCommand;
    type Error = anyhow::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let magic_b = src.iter().position(|&byte| byte == MAGIC_SEPARATOR);
        if let Some(i) = magic_b {
            let cmd = src.split_to(i);

            let cmd_name = ignore_ill_utf8(&cmd);
            src.advance(1);

            let protocol_end = src.iter().rposition(|&b| b == MAGIC_END);

            if let Some(i) = protocol_end {
                let args = src.split_to(i - 2);

                src.clear();

                return Ok(Some(Command::from_protocol(
                    cmd_name,
                    args.as_ref()
                        .split(|&b| b == MAGIC_SEPARATOR)
                        .map(|s| ignore_ill_utf8(s)),
                )?));
            }
        }

        Ok(None)
    }
}
