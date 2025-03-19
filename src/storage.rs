use dirs;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
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
        writeln!(file, "[]")?;
    }

    Ok(contacts_file)
}

pub fn load_contacts(path: &PathBuf) -> io::Result<Vec<Contact>> {
    let data = fs::read_to_string(path)?;
    let contacts: Vec<Contact> = serde_json::from_str(&data)?;
    Ok(contacts)
}

pub fn save_contacts(path: &PathBuf, contacts: &Vec<Contact>) -> io::Result<()> {
    let data = serde_json::to_string_pretty(contacts)?;
    fs::write(path, data)
}