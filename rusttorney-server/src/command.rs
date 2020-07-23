use crate::networking::{Command, WithStrIter};

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

#[rustfmt::skip]
#[derive(Debug, Command, PartialEq)]
pub enum ClientCommand {
    #[command(code = "HI")]
    Handshake(String),                           // HI#<hdid:String>#%
    #[command(code = "ID")]
    ClientVersion(u32, String, String),          /* ID#<pv:u32>#<software:String>#
                                                  * <version:String>#% */
    #[command(code = "CH")]
    KeepAlive(i32),                              // CH
    #[command(code = "askchaa")]
    AskListLengths,                              // askchaa
    #[command(code = "askchar")]
    AskListCharacters,                           // askchar
    #[command(code = "AN")]
    CharacterList(u32),                          // AN#<page:u32>#%
    #[command(code = "AE")]
    EvidenceList(u32),                           // AE#<page:u32>#%
    #[command(code = "AM")]
    MusicList(u32),                              // AM#<page:u32>#%
    #[command(code = "AC")]
    AO2CharacterList,                            // AC#%
    #[command(code = "AM")]
    AO2MusicList,                                // AM#%
    #[command(code = "RD")]
    AO2Ready,                                    // RD#%
    #[command(code = "CC")]
    SelectCharacter(u32, u32, String),           /* CC<client_id:u32>#
                                                  * <char_id:u32#<hdid:
                                                  * String>#% */
    #[command(code = "MS")]
    ICMessage,                                   // MS
    #[command(code = "CT")]
    OOCMessage(String, String),                  /* CT#<name:String>#
                                                  * <message:String>#% */
    #[command(code = "MC")]
    PlaySong(u32, u32),                          // MC#<song_name:u32>#<???:u32>#%
    #[command(code = "RT")]
    WTCEButtons(String),                         // RT#<type:String>#%
    // #[command(skip)]
    #[command(code = "SETCASE")]
    SetCasePreferences(String, #[command(flatten)] CasePreferences), /* SETCASE#<cases:String>#<will_cm:boolean>#<will_def:boolean>#<will_pro:boolean>#<will_judge:boolean>#<will_jury:boolean>#<will_steno:boolean>#% */
    // #[command(skip)]
    #[command(code = "CASEA")]
    CaseAnnounce(String, #[command(flatten)] CasePreferences),       // CASEA
    #[command(code = "HP")]
    Penalties(u32, u32),                         /* HP#<type:u32>#
                                                  * <new_value:u32>#% */
    // #[command(skip)]
    #[command(code = "PE")]
    AddEvidence(#[command(flatten)] EvidenceArgs),                   /* PE#<name:String>#<description:String>#
                                                  * <image:String>#% */
    #[command(code = "DE")]
    DeleteEvidence(u32),                         // DE#<id:u32>#%
    // #[command(skip)]
    #[command(code = "EE")]
    EditEvidence(u32, #[command(flatten)] EvidenceArgs),             /* EE#<id:u32>#<name:String>#
                                                  * <description:String>#<image:
                                                  * String>#% */
    #[command(skip)]
    #[command(code = "ZZ")]
    CallModButton(Option<String>),               // ZZ?#<reason:String>?#%
}

#[derive(Debug, PartialEq, WithStrIter)]
pub struct EvidenceArgs {
    pub name: String,
    pub description: String,
    pub image: String,
}

#[derive(Debug, PartialEq, WithStrIter)]
pub struct CasePreferences {
    pub cm: bool,
    pub def: bool,
    pub pro: bool,
    pub judge: bool,
    pub jury: bool,
    pub steno: bool,
}

#[rustfmt::skip]
#[derive(Debug, Command)]
pub enum ServerCommand {
    #[command(code = "HI")]
    Handshake(String),                  // HI#<hdid:String>#%
    #[command(code = "CHECK")]
    KeepAlive,                          // CHECK#%
    #[command(code = "decryptor")]
    Decryptor(u32),                     // decryptor#<i:u32>#%
    #[command(code = "BD")]
    BanReason(String),                  // BD#<reason:String>#%,
    #[command(code = "ID")]
    ServerVersion(u8, String, String),  // ID#<client_id:u32>#<software:String>#<version:String>#%
    #[command(code = "PN")]
    PlayerCount(u8, u8),                // PN#<player_count:u8>#<max_players:u8>#%
}
