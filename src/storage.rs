use dirs;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use crate::models::TuppData;
use crate::contact::Contact;

pub fn get_config_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("Failed to get home directory");
    path.push(".config");
    path.push("tupp");
    path
}

pub fn ensure_config_file() -> io::Result<PathBuf> {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    let contacts_file = config_dir.join("contacts.json");
    if !contacts_file.exists() {
        let mut file = File::create(&contacts_file)?;
        writeln!(file, "{{\"contacts\": [], \"groups\": []}}")?;
    }

    Ok(contacts_file)
}

pub fn load_data(path: &PathBuf) -> io::Result<TuppData> {
    let data = fs::read_to_string(path)?;
    match serde_json::from_str::<TuppData>(&data) {
        Ok(tupp_data) => Ok(tupp_data),
        Err(_) => {
            // Try to load as 1.1.0 format (array of contacts)
            match serde_json::from_str::<Vec<Contact>>(&data) {
                Ok(contacts) => Ok(TuppData {
                    contacts,
                    groups: Vec::new(),
                }),
                Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            }
        }
    }
}

pub fn save_data(path: &PathBuf, data: &TuppData) -> io::Result<()> {
    let json_data = serde_json::to_string_pretty(data)?;
    fs::write(path, json_data)
}