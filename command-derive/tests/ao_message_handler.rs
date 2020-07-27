pub struct AO2MessageHandler<'a>(std::marker::PhantomData<&'a ()>);

impl<'a> AO2MessageHandler<'a> {
    pub async fn handle_handshake(&mut self, _hdid: String) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_client_version(
        &mut self,
        _id: u32,
        _name: String,
        _surname: String,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_keepalive(&mut self, _id: i32) -> Result<(), anyhow::Error> {
        unimplemented!()
    }

    pub async fn handle_edit_evidence(
        &mut self,
        _id: u32,
        _nested: super::EvidenceArgs,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }
}
