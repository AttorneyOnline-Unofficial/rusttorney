use serde::Deserialize;

type Category = String;

#[derive(Debug, Deserialize)]
pub struct MusicList {
    pub music: Vec<Music>,
}

#[derive(Debug, Deserialize)]
pub struct Music {
    category: String,
    songs: Vec<Song>,
}

#[derive(Debug, Deserialize)]
pub struct Song {
    pub name: String,
    pub length: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_music_list_de() {
        let music_list_str = r#"
[[music]]

category = "== Vanilla =="

[[music.songs]]
name = "01_turnabout_courtroom_-_prologue.mp3"
length = 40.099833
        "#;

        let music_list: MusicList = toml::from_str(music_list_str).unwrap();
        assert_eq!(music_list.music[0].category, "== Vanilla ==");
        assert_eq!(
            music_list.music[0].songs[0].name,
            "01_turnabout_courtroom_-_prologue.mp3"
        );
    }
}
