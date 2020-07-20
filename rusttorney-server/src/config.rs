use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config<'a> {
    pub debug: bool,
    pub timeout: u32,
    pub multiclient_limit: u8,
    pub max_chars: u32,
    pub zalgo_tolerance: u8,
    #[serde(borrow)]
    pub general: GeneralConfig<'a>,
    #[serde(borrow)]
    pub masterserver: MasterServerConfig<'a>,
    pub wtce_floodguard: FloodGuardConfig,
    pub music_change_floodguard: FloodGuardConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig<'a> {
    pub hostname: &'a str,
    pub playerlimit: u8,
    pub port: u32,
    pub local: bool,
    pub modpass: &'a str,
    pub motd: &'a str,
    pub websocket_port: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct MasterServerConfig<'a> {
    #[serde(rename = "use")]
    pub use_masterserver: bool,
    pub ip: &'a str,
    pub port: u16,
    pub name: &'a str,
    pub description: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct FloodGuardConfig {
    pub times_per_interval: u8,
    pub interval_length: u8,
    pub mute_length: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let config_str = r#"
        debug = false
        timeout = 250
        multiclient_limit = 16
        max_chars = 256
        zalgo_tolerance = 3


        [general]
        hostname = "<dollar>H"
        playerlimit = 100
        port = 27016
        local = false
        modpass = "mod"
        motd = "Welcome to my server!"
        use_websockets = true
        websocket_port = 50001

        [masterserver]
        use = true
        ip = "master.aceattorneyonline.com"
        port = 27016
        name = "My server"
        description = "My server description!"

        [music_change_floodguard]
        times_per_interval = 3
        interval_length = 20
        mute_length = 180

        [wtce_floodguard]
        times_per_interval = 5
        interval_length = 10
        mute_length = 1000
        "#;
        let config: Config = toml::from_str(&config_str).unwrap();

        assert_eq!(config.debug, false);
        assert_eq!(config.masterserver.name, "My server")
    }
}
