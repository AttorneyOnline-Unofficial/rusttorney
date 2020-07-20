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
        const ARG_SEP: u8 = b'#';
        const MSG_END: &[u8] = b"#%";

        if src.len() > 8192 {
            // spam protection? Copied from legacy server
            return Err(anyhow::anyhow!("Too much data"));
        }

        // Find the end of AO message
        let msg_end = match src.windows(2).position(|s| s == MSG_END) {
            Some(idx) => idx,
            None => return Ok(None),
        };

        // Take message from the buffer
        let mut msg = src.split_to(msg_end);
        // Forget message separator
        src.advance(MSG_END.len());

        // Find the end of command name in message
        let cmd_end =
            msg.iter().position(|&c| c == ARG_SEP).unwrap_or_else(|| msg.len());
        // Take the command name
        let cmd_raw = msg.split_to(cmd_end);
        let cmd = ignore_ill_utf8(&cmd_raw[..]);

        // Divide rest of the message into chunks.
        // If there are any arguments in the slice, it starts with '#'.
        // `.skip(1)` ignores the empty string appearing because of it
        let args_iter =
            msg.as_ref().split(|&c| c == ARG_SEP).skip(1).map(ignore_ill_utf8);

        Ok(Some(ClientCommand::from_protocol(cmd, args_iter)?))
    }

    fn decode_eof(
        &mut self,
        buf: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        match self.decode(buf)? {
            Some(frame) => Ok(Some(frame)),
            None => {
                if !buf.is_empty() {
                    log::debug!("Ignoring remaining data");
                    log::trace!("Ignored data: {:?}", buf.as_ref());
                }
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_handshake() {
        let mut input = b"HI#hdid#%"[..].into();
        let expected = ClientCommand::Handshake("hdid".into());
        let actual = AOMessageCodec.decode(&mut input).unwrap().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn mismatched_number_of_args() {
        let mut input1 = b"HI#%"[..].into();
        let mut input2 = b"HI#hdid#junk#%"[..].into();
        assert!(AOMessageCodec.decode(&mut input1).is_err());
        assert!(AOMessageCodec.decode(&mut input2).is_err());
    }

    #[test]
    fn two_messages_in_one_chunk() {
        let mut src = b"HI#hdid1#%HI#hdid2#%"[..].into();
        let expected1 = ClientCommand::Handshake("hdid1".into());
        let expected2 = ClientCommand::Handshake("hdid2".into());
        let mut codec = AOMessageCodec;
        let actual1 = codec.decode(&mut src).unwrap().unwrap();
        assert_eq!(expected1, actual1);
        let actual2 = codec.decode(&mut src).unwrap().unwrap();
        assert_eq!(expected2, actual2);
    }
}
