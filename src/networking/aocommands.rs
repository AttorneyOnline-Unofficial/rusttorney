use crate::networking::Command;

use std::fmt::Display;
use std::str::FromStr;

#[rustfmt::skip]
#[derive(Debug, PartialEq)]
pub enum ClientCommand {
    Handshake(String),                           // HI#<hdid:String>#%
    ClientVersion(u32, String, String),          // ID#<pv:u32>#<software:String>#<version:String>#%
    KeepAlive,                                   // CH
    AskListLengths,                              // askchaa
    AskListCharacters,                           // askchar
    CharacterList(u32),                          // AN#<page:u32>#%
    EvidenceList(u32),                           // AE#<page:u32>#%
    MusicList(u32),                              // AM#<page:u32>#%
    AO2CharacterList,                            // AC#%
    AO2MusicList,                                // AM#%
    AO2Ready,                                    // RD#%
    SelectCharacter(u32, u32, String),           // CC<client_id:u32>#<char_id:u32#<hdid:String>#%
    ICMessage,                                   // MS
    OOCMessage(String, String),                  // CT#<name:String>#<message:String>#%
    PlaySong(u32, u32),                          // MC#<song_name:u32>#<???:u32>#%
    WTCEButtons(String),                         // RT#<type:String>#%
    SetCasePreferences(String, CasePreferences), // SETCASE#<cases:String>#<will_cm:boolean>#<will_def:boolean>#<will_pro:boolean>#<will_judge:boolean>#<will_jury:boolean>#<will_steno:boolean>#%
    CaseAnnounce(String, CasePreferences),       // CASEA
    Penalties(u32, u32),                         // HP#<type:u32>#<new_value:u32>#%
    AddEvidence(EvidenceArgs),                   // PE#<name:String>#<description:String>#<image:String>#%
    DeleteEvidence(u32),                         // DE#<id:u32>#%
    EditEvidence(u32, EvidenceArgs),             // EE#<id:u32>#<name:String>#<description:String>#<image:String>#%
    CallModButton(Option<String>),               // ZZ?#<reason:String>?#%
}

#[rustfmt::skip]
#[derive(Debug)]
pub enum ServerCommand {
    Handshake(String)
}

impl ServerCommand {
    pub fn extract_args(&self) -> Option<Vec<&str>> {
        use ServerCommand::*;

        match self {
            Handshake(str) => Some(vec![str]),
        }
    }

    pub fn ident(&self) -> &str {
        use ServerCommand::*;

        match self {
            Handshake(_) => "HI",
        }
    }
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
