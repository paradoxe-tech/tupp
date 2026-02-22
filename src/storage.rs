use dirs;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use crate::models::TuppData;
use crate::contact::Contact;
use crate::error::TuppError;

pub fn get_config_dir() -> Result<PathBuf, TuppError> {
    let mut path = dirs::home_dir().ok_or(TuppError::ConfigDirNotFound)?;
    path.push(".config");
    path.push("tupp");
    Ok(path)
}

pub fn ensure_config_file() -> Result<PathBuf, TuppError> {
    let config_dir = get_config_dir()?;
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(TuppError::Io)?;
    }

    let contacts_file = config_dir.join("contacts.json");
    if !contacts_file.exists() {
        let mut file = File::create(&contacts_file).map_err(TuppError::Io)?;
        writeln!(file, "{{ \"contacts\": [], \"groups\": [] }}").map_err(TuppError::Io)?;
    }

    Ok(contacts_file)
}

pub fn load_data(path: &PathBuf) -> Result<TuppData, TuppError> {
    let data = fs::read_to_string(path).map_err(TuppError::Io)?;
    match serde_json::from_str::<TuppData>(&data) {
        Ok(tupp_data) => Ok(tupp_data),
        Err(_) => {
            // Try to load as 1.1.0 format (array of contacts)
            match serde_json::from_str::<Vec<Contact>>(&data) {
                Ok(contacts) => Ok(TuppData {
                    contacts,
                    groups: Vec::new(),
                }),
                Err(e) => Err(TuppError::Serialization(e)),
            }
        }
    }
}

pub fn save_data(path: &PathBuf, data: &TuppData) -> Result<(), TuppError> {
    let json_data = serde_json::to_string_pretty(data).map_err(TuppError::Serialization)?;
    fs::write(path, json_data).map_err(TuppError::Io)
}
