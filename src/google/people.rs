use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::error::TuppError;

const PEOPLE_API: &str = "https://people.googleapis.com/v1";
const PERSON_FIELDS: &str = "names,emailAddresses,phoneNumbers,addresses,biographies,urls,organizations,genders,birthdays,photos";
// Guards against an unexpectedly huge response when downloading a photo.
const MAX_PHOTO_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GName {
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub middle_name: Option<String>,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub honorific_prefix: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GEmail {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GPhone {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub canonical_form: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GAddress {
    #[serde(default)]
    pub street_address: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GBiography {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GUrl {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GOrganization {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GGender {
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GDate {
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub month: Option<u8>,
    #[serde(default)]
    pub day: Option<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GBirthday {
    #[serde(default)]
    pub date: Option<GDate>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GPhoto {
    #[serde(default)]
    pub url: Option<String>,
    /// True for Google's auto-generated monogram avatar rather than a photo
    /// the person actually uploaded.
    #[serde(default)]
    pub default: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleContact {
    #[serde(default)]
    pub resource_name: String,
    #[serde(default)]
    pub etag: String,
    #[serde(default)]
    pub names: Vec<GName>,
    #[serde(default)]
    pub email_addresses: Vec<GEmail>,
    #[serde(default)]
    pub phone_numbers: Vec<GPhone>,
    #[serde(default)]
    pub addresses: Vec<GAddress>,
    #[serde(default)]
    pub biographies: Vec<GBiography>,
    #[serde(default)]
    pub urls: Vec<GUrl>,
    #[serde(default)]
    pub organizations: Vec<GOrganization>,
    #[serde(default)]
    pub genders: Vec<GGender>,
    #[serde(default)]
    pub birthdays: Vec<GBirthday>,
    #[serde(default)]
    pub photos: Vec<GPhoto>,
}

impl GoogleContact {
    pub fn display_name(&self) -> String {
        if let Some(name) = self.names.first() {
            if let Some(display) = &name.display_name {
                if !display.trim().is_empty() {
                    return display.clone();
                }
            }
            let parts: Vec<&str> = [&name.given_name, &name.middle_name, &name.family_name]
                .into_iter()
                .filter_map(|p| p.as_deref())
                .filter(|p| !p.trim().is_empty())
                .collect();
            if !parts.is_empty() {
                return parts.join(" ");
            }
        }
        self.resource_name.clone()
    }

    pub fn note(&self) -> Option<&str> {
        self.biographies.first().and_then(|b| b.value.as_deref())
    }

    /// The tupp UUID this contact is linked to, if its note carries one.
    pub fn tuppsync_id(&self) -> Option<uuid::Uuid> {
        self.note()?.lines().find_map(|line| {
            line.trim()
                .strip_prefix("tuppsync-id:")
                .and_then(|rest| uuid::Uuid::parse_str(rest.trim()).ok())
        })
    }

    /// The URL of a real, user-set photo — skips Google's auto-generated
    /// monogram avatars, which aren't worth downloading.
    pub fn photo_url(&self) -> Option<&str> {
        self.photos
            .iter()
            .find(|p| p.default != Some(true))
            .and_then(|p| p.url.as_deref())
    }
}

#[derive(Deserialize)]
struct ListConnectionsResponse {
    #[serde(default)]
    connections: Vec<GoogleContact>,
    #[serde(default)]
    next_page_token: Option<String>,
}

fn describe_ureq_error(e: ureq::Error) -> TuppError {
    match e {
        ureq::Error::Status(code, response) => {
            let body = response.into_string().unwrap_or_default();
            TuppError::Other(format!("Google People API error ({}): {}", code, body))
        }
        other => TuppError::Other(format!("Request to Google People API failed: {}", other)),
    }
}

pub fn list_contacts(access_token: &str) -> Result<Vec<GoogleContact>, TuppError> {
    let mut contacts = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let url = format!("{}/people/me/connections", PEOPLE_API);
        let mut req = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", access_token))
            .query("personFields", PERSON_FIELDS)
            .query("pageSize", "1000");

        if let Some(token) = &page_token {
            req = req.query("pageToken", token);
        }

        let response = req.call().map_err(describe_ureq_error)?;
        let page: ListConnectionsResponse = response
            .into_json()
            .map_err(|e| TuppError::Other(format!("Invalid response from Google: {}", e)))?;

        contacts.extend(page.connections);

        match page.next_page_token {
            Some(token) => page_token = Some(token),
            None => break,
        }
    }

    Ok(contacts)
}

/// Sends a partial update for `resource_name`. `body` must contain the
/// `etag` plus the *complete* value for every field group named in
/// `update_person_fields` (the People API replaces those groups wholesale).
pub fn patch_contact(
    access_token: &str,
    resource_name: &str,
    update_person_fields: &[&str],
    body: serde_json::Value,
) -> Result<(), TuppError> {
    let url = format!("{}/{}:updateContact", PEOPLE_API, resource_name);
    ureq::request("PATCH", &url)
        .set("Authorization", &format!("Bearer {}", access_token))
        .query("updatePersonFields", &update_person_fields.join(","))
        .send_json(body)
        .map_err(describe_ureq_error)?;
    Ok(())
}

// Google's `googleusercontent.com` photo URLs carry a size directive after
// the last `=` (e.g. `...=s100-c`), and the People API defaults to a small
// ~100px thumbnail. Swapping it for a bigger one is the standard way to get
// a higher-resolution copy without any extra API call.
const PHOTO_SIZE_PX: u32 = 512;

fn upsized_photo_url(url: &str) -> String {
    let base = url.rsplit_once('=').map(|(base, _)| base).unwrap_or(url);
    format!("{}=s{}-c", base, PHOTO_SIZE_PX)
}

/// Downloads a contact photo. The URL comes straight from the People API
/// response and is already a directly-fetchable `googleusercontent.com`
/// link — no auth header needed, same as an `<img src>` would use.
pub fn download_photo(url: &str) -> Result<(Vec<u8>, String), TuppError> {
    let response = ureq::get(&upsized_photo_url(url))
        .call()
        .map_err(describe_ureq_error)?;
    let content_type = response.content_type().to_string();

    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(MAX_PHOTO_BYTES)
        .read_to_end(&mut bytes)
        .map_err(TuppError::Io)?;

    Ok((bytes, content_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsized_photo_url_replaces_existing_size_directive() {
        assert_eq!(
            upsized_photo_url("https://lh3.googleusercontent.com/a-/ABC123=s100-c"),
            "https://lh3.googleusercontent.com/a-/ABC123=s512-c"
        );
    }

    #[test]
    fn upsized_photo_url_appends_directive_when_absent() {
        assert_eq!(
            upsized_photo_url("https://lh3.googleusercontent.com/a-/ABC123"),
            "https://lh3.googleusercontent.com/a-/ABC123=s512-c"
        );
    }
}
