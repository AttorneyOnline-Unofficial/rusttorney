use crate::networking::Command;
use bytes::{Buf, BytesMut};
use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    str::FromStr,
};
use tokio_util::codec::Decoder;

#[rustfmt::skip]
#[derive(Debug, PartialEq)]
pub enum ClientCommand {
    Handshake(String),                  // HI#<hdid:String>#%
    ClientVersion(u32, String, String), /* ID#<pv:u32>#<software:String>#
                                         * <version:String>#% */
    KeepAlive,                                   // CH
    AskListLengths,                              // askchaa
    AskListCharacters,                           // askchar
    CharacterList(u32),                          // AN#<page:u32>#%
    EvidenceList(u32),                           // AE#<page:u32>#%
    MusicList(u32),                              // AM#<page:u32>#%
    AO2CharacterList,                            // AC#%
    AO2MusicList,                                // AM#%
    AO2Ready,                                    // RD#%
    SelectCharacter(u32, u32, String),           /* CC<client_id:u32>#
                                                  * <char_id:u32#<hdid:
                                                  * String>#% */
    ICMessage,                                   // MS
    OOCMessage(String, String),                  /* CT#<name:String>#
                                                  * <message:String>#% */
    PlaySong(u32, u32),  // MC#<song_name:u32>#<???:u32>#%
    WTCEButtons(String), // RT#<type:String>#%
    SetCasePreferences(String, CasePreferences), /* SETCASE#<cases:String>#<will_cm:boolean>#<will_def:boolean>#<will_pro:boolean>#<will_judge:boolean>#<will_jury:boolean>#<will_steno:boolean>#% */
    CaseAnnounce(String, CasePreferences),       // CASEA
    Penalties(u32, u32),                         /* HP#<type:u32>#
                                                  * <new_value:u32>#% */
    AddEvidence(EvidenceArgs), /* PE#<name:String>#<description:String>#
                                * <image:String>#% */
    DeleteEvidence(u32),             // DE#<id:u32>#%
    EditEvidence(u32, EvidenceArgs), /* EE#<id:u32>#<name:String>#
                                      * <description:String>#<image:
                                      * String>#% */
    CallModButton(Option<String>), // ZZ?#<reason:String>?#%
}

impl Command for ClientCommand {
    fn from_protocol<'a>(
        name: String,
        mut args: impl Iterator<Item = String>,
    ) -> Result<Self, anyhow::Error> {
        let on_err = || {
            anyhow::anyhow!(
                "Amount of arguments for command {} does not match!",
                &name
            )
        };

        fn next<E, T, F>(
            mut args: impl Iterator<Item = String>,
            on_err: F,
        ) -> Result<T, anyhow::Error>
        where
            E: Display,
            T: FromStr<Err = E>,
            F: Fn() -> anyhow::Error,
        {
            args.next()
                .ok_or_else(on_err)
                .map(|s| s.parse::<T>().map_err(|e| anyhow::anyhow!("{}", e)))
                .and_then(std::convert::identity)
        }

        match name.as_str() {
            "HI" => {
                let res = Ok(Self::Handshake(next(&mut args, on_err)?));
                if args.next().is_some() {
                    return Err(on_err());
                }
                res
            }
            _ => Err(on_err()),
        }
    }
    fn handle(&self) -> futures::future::BoxFuture<'static, ()> {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct EvidenceArgs {
    pub name: String,
    pub description: String,
    pub image: String,
}

#[derive(Debug, PartialEq)]
pub struct CasePreferences {
    pub cm: bool,
    pub def: bool,
    pub pro: bool,
    pub judge: bool,
    pub jury: bool,
    pub steno: bool,
}

// #[derive(Debug)]
// pub struct AOMessage {
//     pub command: ClientCommand,
// }

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
}

fn ignore_ill_utf8(v: &[u8]) -> String {
    use std::char::REPLACEMENT_CHARACTER;

    let str = String::from_utf8_lossy(&v);

    match str {
        Cow::Owned(mut own) => {
            own.retain(|c| c != REPLACEMENT_CHARACTER);
            own
        }
        Cow::Borrowed(brw) => brw.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_handshake() {
        let mut input = b"HI#hdid#%"[..].into();
        let expected = ClientCommand::Handshake("hdid".into());
        let actual =
            AOMessageCodec.decode(&mut input).unwrap().unwrap();
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
