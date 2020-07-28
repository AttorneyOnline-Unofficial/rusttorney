use crate::{
    command::{CasePreferences, EvidenceArgs, ServerCommand},
    server::AO2MessageHandler,
};

use futures::SinkExt;

impl AO2MessageHandler {
    pub async fn handle_handshake(
        &mut self,
        hdid: String,
    ) -> Result<(), anyhow::Error> {
        self.client.hdid = hdid.clone();
        self.client_manager.lock().await.update_client(self.client.clone());

        self.db.add_hdid(hdid, self.client.ipid).await?;

        self.socket
            .send(ServerCommand::ServerVersion(
                self.client.id,
                self.software.clone(),
                self.version.clone(),
            ))
            .await?;

        self.socket
            .send(ServerCommand::PlayerCount(
                self.player_count().await,
                self.config.general.playerlimit,
            ))
            .await
    }

    pub async fn handle_client_version(
        &mut self,
        _: u32,
        _: String,
        _: String,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_keepalive(
        &mut self,
        _: i32,
    ) -> Result<(), anyhow::Error> {
        self.ch_tx.send(()).await?;
        self.socket.send(ServerCommand::KeepAlive).await
    }

    pub async fn handle_ask_list_lengths(
        &mut self,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_ask_list_characters(
        &mut self,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_character_list(
        &mut self,
        _: u32,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_evidence_list(
        &mut self,
        _: u32,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_music_list(
        &mut self,
        _: u32,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_ao2_character_list(
        &mut self,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_ao2_music_list(&mut self) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_ao2_ready(&mut self) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_select_character(
        &mut self,
        _: u32,
        _: u32,
        _: String,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_ic_message(&mut self) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_ooc_message(
        &mut self,
        _: String,
        _: String,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_play_song(
        &mut self,
        _: u32,
        _: u32,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_wtce_buttons(
        &mut self,
        _: String,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_set_case_preferences(
        &mut self,
        _: String,
        _: CasePreferences,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_case_announce(
        &mut self,
        _: String,
        _: CasePreferences,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_penalties(
        &mut self,
        _: u32,
        _: u32,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_add_evidence(
        &mut self,
        _: EvidenceArgs,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_delete_evidence(
        &mut self,
        _: u32,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_edit_evidence(
        &mut self,
        _: u32,
        _: EvidenceArgs,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_call_mod_button(
        &mut self,
        _: String,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }
}
