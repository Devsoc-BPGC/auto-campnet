use super::data_types::Credentials;
use std::io::{Error, Read, Write};
use std::path::PathBuf;
use std::{fs, fs::File};
use tauri::AppHandle;

#[derive(Clone)]
pub struct CredentialManager {
    app_handle: AppHandle,
}

impl CredentialManager {
    pub fn new(app_handle: AppHandle) -> CredentialManager {
        CredentialManager { app_handle }
    }

    fn get_save_file(&self) -> PathBuf {
        self.app_handle
            .path_resolver()
            .app_config_dir()
            .unwrap()
            .join("credentials.json")
    }

    pub fn save(&self, creds: Credentials) -> Result<usize, Error> {
        if let Some(config_dir) = self.app_handle.path_resolver().app_config_dir() {
            fs::create_dir_all(config_dir)?;
        }
        let mut file = File::create(&self.get_save_file())?;
        file.write(serde_json::to_string(&creds).unwrap().as_bytes())
    }

    pub fn load(&self) -> Result<Credentials, String> {
        let mut file = match File::open(self.get_save_file()) {
            Ok(file) => file,
            Err(_) => return Err("Credentials not saved".to_string()),
        };
        let mut creds_string = String::new();
        if file.read_to_string(&mut creds_string).is_ok() {
            if let Ok(creds) = serde_json::from_str(&creds_string) {
                return Ok(creds);
            }
        }
        Err("Credentials not saved".to_string())
    }

    pub fn clear(&self) -> Result<(), Error> {
        match fs::remove_file(self.get_save_file()) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }
}
