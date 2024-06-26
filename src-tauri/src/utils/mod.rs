use crate::models::{SavedLyric, SavedLyrics};
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::hash::Hasher;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct Utils {
    path: String,
}

impl Utils {
    pub fn new() -> Self {
        Utils {
            path: "./data".to_string(),
        }
    }

    fn create_data_folder(&self) -> Result<&Self, Box<dyn Error>> {
        if !fs::metadata(self.path.clone()).is_ok() {
            fs::create_dir(self.path.as_str())?;
        }

        Ok(self)
    }

    fn create_folder_if_not_exists(&self, path: &str) -> Result<&Self, Box<dyn Error>> {
        let folder_path = format!("{}/{}", self.path.as_str(), path);
        if !fs::metadata(folder_path.clone()).is_ok() {
            fs::create_dir(folder_path.as_str())?;
        }

        Ok(self)
    }

    fn create_file(&self, file_path: String) -> Result<File, Box<dyn Error>> {
        let file = File::create(file_path)?;
        Ok(file)
    }

    fn hash_utf8(data: &str) -> String {
        let mut hasher = DefaultHasher::new();
        hasher.write(data.as_bytes());
        hasher.finish().to_string()
    }

    pub async fn save_lyric(
        &self,
        title: String,
        artist: String,
        img: String,
        lyrics: String,
    ) -> Result<(), Box<dyn Error>> {
        let file_path = format!("{}/lyrics.json", self.path);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        let mut json_data = SavedLyrics { lyrics: vec![] };

        if !data.is_empty() {
            // Parse the JSON content
            json_data = serde_json::from_str(&data).expect("Error parsing JSON");
        }

        let write_data = SavedLyric {
            name: title.clone(),
            artist,
            img,
        };

        json_data.lyrics.push(write_data);
        file.seek(SeekFrom::Start(0))?;
        serde_json::to_writer(&file, &json_data)?;
        self.save_lrc(title, lyrics).await
    }

    async fn save_lrc(&self, title: String, lyrics: String) -> Result<(), Box<dyn Error>> {
        let file_name = Self::hash_utf8(&title);
        let file_path = format!("{}/lyrics/{}.lrc", self.path, file_name);
        let mut file = self
            .create_data_folder()?
            .create_folder_if_not_exists("lyrics")?
            .create_file(file_path)?;
        file.write(lyrics.as_bytes())?;

        Ok(())
    }

    pub fn save_authentication(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<(), Box<dyn Error>> {
        let file_path = format!("{}/client/auth.json", self.path);
        let file = self
            .create_data_folder()?
            .create_folder_if_not_exists("client")?
            .create_file(file_path)?;
        let auth = json!({"client_id": client_id, "client_secret": client_secret});
        serde_json::to_writer(&file, &auth)?;

        Ok(())
    }

    pub async fn get_authentication(&self) -> Result<(String, String), Box<dyn Error>> {
        let file_path = format!("{}/client/auth.json", self.path);
        let file = File::open(file_path)?;
        let auth: serde_json::Value = serde_json::from_reader(file)?;
        let client_id = auth["client_id"].as_str().unwrap().to_string();
        let client_secret = auth["client_secret"].as_str().unwrap().to_string();

        Ok((client_id, client_secret))
    }

    pub async fn get_lyrics_list(&self) -> Result<SavedLyrics, Box<dyn Error>> {
        let file_path = format!("{}/lyrics.json", self.path);
        let file = File::open(file_path)?;
        let data: SavedLyrics = serde_json::from_reader(file)?;

        Ok(data)
    }

    pub async fn check_lyrics(&self, title: String) -> Result<String, Box<dyn Error>> {
        let path = format!("{}/lyrics", self.path);

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    let file_name = file_name.to_str().unwrap();
                    let file_name = Path::new(file_name).file_stem().unwrap().to_str().unwrap();
                    if Self::hash_utf8(title.as_str()) == file_name.to_string() {
                        let mut file = File::open(path)?;
                        let mut data = String::new();
                        file.read_to_string(&mut data)?;

                        return Ok(data);
                    }
                }
            }
        }

        Err("No lyrics found!".into())
    }
}
