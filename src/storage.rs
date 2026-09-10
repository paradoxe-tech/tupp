use dirs;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use crate::models::TuppData;
use crate::contact::Contact;
use crate::error::TuppError;
use uuid::Uuid;

pub fn get_config_dir() -> Result<PathBuf, TuppError> {
    let mut path = dirs::home_dir().ok_or(TuppError::ConfigDirNotFound)?;
    path.push(".config");
    path.push("tupp");
    Ok(path)
}

/// Extensions we know how to save/serve a contact photo as, paired with
/// their HTTP content type.
const PHOTO_EXTENSIONS: &[(&str, &str)] = &[
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("png", "image/png"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
];

pub fn extension_for_content_type(content_type: &str) -> &'static str {
    PHOTO_EXTENSIONS
        .iter()
        .find(|(_, ct)| *ct == content_type)
        .map(|(ext, _)| *ext)
        .unwrap_or("jpg")
}

pub fn photos_dir() -> Result<PathBuf, TuppError> {
    let mut path = get_config_dir()?;
    path.push("photos");
    Ok(path)
}

fn ensure_photos_dir() -> Result<PathBuf, TuppError> {
    let dir = photos_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(TuppError::Io)?;
    }
    Ok(dir)
}

pub fn has_photo(id: &Uuid) -> bool {
    let Ok(dir) = photos_dir() else { return false };
    PHOTO_EXTENSIONS
        .iter()
        .any(|(ext, _)| dir.join(format!("{}.{}", id, ext)).exists())
}

/// Reads a contact's photo from disk, trying every known extension. Returns
/// the raw bytes and their content type, for serving directly over HTTP.
pub fn find_photo(id: &Uuid) -> Option<(Vec<u8>, &'static str)> {
    let dir = photos_dir().ok()?;
    for (ext, content_type) in PHOTO_EXTENSIONS {
        let path = dir.join(format!("{}.{}", id, ext));
        if let Ok(bytes) = fs::read(&path) {
            return Some((bytes, content_type));
        }
    }
    None
}

/// Saves a contact's photo, named after its content type. Any previously
/// saved photo for this contact (possibly under a different extension, if
/// the content type changed between syncs) is removed first.
pub fn save_photo(id: &Uuid, bytes: &[u8], content_type: &str) -> Result<(), TuppError> {
    let dir = ensure_photos_dir()?;
    for (ext, _) in PHOTO_EXTENSIONS {
        let _ = fs::remove_file(dir.join(format!("{}.{}", id, ext)));
    }
    let ext = extension_for_content_type(content_type);
    fs::write(dir.join(format!("{}.{}", id, ext)), bytes).map_err(TuppError::Io)
}

pub fn ensure_config_file() -> Result<PathBuf, TuppError> {
    let config_dir = get_config_dir()?;
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(TuppError::Io)?;
    }

    let contacts_file = config_dir.join("contacts.json");
    if !contacts_file.exists() {
        let mut file = File::create(&contacts_file).map_err(TuppError::Io)?;
        writeln!(file, "{{ \"contacts\": [], \"groups\": [], \"institutions\": [] }}").map_err(TuppError::Io)?;
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
                    institutions: Vec::new(),
                }),
                Err(e) => Err(TuppError::Serialization(e)),
            }
        }
    }
}

use crate::validation;

pub fn save_data(path: &PathBuf, data: &TuppData) -> Result<(), TuppError> {
    // Validate data structure against schema
    let json_value = serde_json::to_value(data).map_err(TuppError::Serialization)?;

    if let Err(e) = validation::validate_json(&json_value) {
        return Err(e);
    }
    
    let json_data = serde_json::to_string_pretty(&json_value).map_err(TuppError::Serialization)?;
    fs::write(path, json_data).map_err(TuppError::Io)
}
