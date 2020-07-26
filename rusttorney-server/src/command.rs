use crate::networking::{Command, WithStrIter};

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

#[rustfmt::skip]
#[derive(Debug, Command, PartialEq)]
#[command(handler = "crate::server::AO2MessageHandler")]
pub enum ClientCommand {
    #[command(code = "HI", handle = "handle_handshake")]
    Handshake(String),                           // HI#<hdid:String>#%
    #[command(code = "ID", handle = "handle_client_version")]
    ClientVersion(u32, String, String),          /* ID#<pv:u32>#<software:String>#
                                                  * <version:String>#% */
    #[command(code = "CH", handle = "handle_keepalive")]
    KeepAlive(i32),                              // CH
    #[command(code = "askchaa", handle = "handle_ask_list_lengths")]
    AskListLengths,                              // askchaa
    #[command(code = "askchar2", handle = "handle_ask_list_characters")]
    AskListCharacters,                           // askchar
    #[command(code = "AN", handle = "handle_character_list")]
    CharacterList(u32),                          // AN#<page:u32>#%
    #[command(code = "AE", handle = "handle_evidence_list")]
    EvidenceList(u32),                           // AE#<page:u32>#%
    #[command(code = "AM", handle = "handle_music_list")]
    MusicList(u32),                              // AM#<page:u32>#%
    #[command(code = "AC", handle = "handle_ao2_character_list")]
    AO2CharacterList,                            // AC#%
    #[command(code = "AM", handle = "handle_ao2_music_list")]
    AO2MusicList,                                // AM#%
    #[command(code = "RD", handle = "handle_ao2_ready")]
    AO2Ready,                                    // RD#%
    #[command(code = "CC", handle = "handle_select_character")]
    SelectCharacter(u32, u32, String),           /* CC<client_id:u32>#
                                                  * <char_id:u32#<hdid:
                                                  * String>#% */
    #[command(code = "MS", handle = "handle_ic_message")]
    ICMessage,                                   // MS
    #[command(code = "CT", handle = "handle_ooc_message")]
    OOCMessage(String, String),                  /* CT#<name:String>#
                                                  * <message:String>#% */
    #[command(code = "MC", handle = "handle_play_song")]
    PlaySong(u32, u32),                          // MC#<song_name:u32>#<???:u32>#%
    #[command(code = "RT", handle = "handle_wtce_buttons")]
    WTCEButtons(String),                         // RT#<type:String>#%
    #[command(code = "SETCASE", handle = "handle_set_case_preferences")]                 /* SETCASE#<cases:String>#<will_cm:boolean>#<will_def:boolean>#<will_pro:boolean>#<will_judge:boolean>#<will_jury:boolean>#<will_steno:boolean>#% */
    SetCasePreferences(String, #[command(flatten)] CasePreferences),
    #[command(code = "CASEA", handle = "handle_case_announce")]                   // CASEA
    CaseAnnounce(String, #[command(flatten)] CasePreferences),
    #[command(code = "HP", handle = "handle_penalties")]
    Penalties(u32, u32),                         /* HP#<type:u32>#
                                                  * <new_value:u32>#% */
    #[command(code = "PE", handle = "handle_add_evidence")]
    AddEvidence(
        #[command(flatten)] EvidenceArgs),       /* PE#<name:String>#<description:String>#
                                                  * <image:String>#% */
    #[command(code = "DE", handle = "handle_delete_evidence")]
    DeleteEvidence(u32),                         // DE#<id:u32>#%
    #[command(code = "EE", handle = "handle_edit_evidence")]
    EditEvidence(u32, #[command(flatten)] EvidenceArgs),
                                                 /* EE#<id:u32>#<name:String>#
                                                  * <description:String>#<image:
                                                  * String>#% */
    #[command(code = "ZZ", handle = "handle_call_mod_button")]
    CallModButton(String),                       // ZZ?#<reason:String>?#%
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
