use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::TuppError;
use crate::storage::get_config_dir;

const AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const SCOPE: &str = "https://www.googleapis.com/auth/contacts";
// Nothing actually listens on this port: the user is expected to paste the
// resulting (possibly failed-to-load) callback URL back into the terminal.
const REDIRECT_URI: &str = "http://127.0.0.1:8721/";
// Refresh a bit before actual expiry to avoid racing a call that's mid-flight.
const EXPIRY_BUFFER_SECS: i64 = 60;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SyncConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub default_region_prefix: Option<u16>,
}

fn sync_config_path() -> Result<PathBuf, TuppError> {
    let mut path = get_config_dir()?;
    path.push("sync.json");
    Ok(path)
}

pub fn load_sync_config() -> Result<SyncConfig, TuppError> {
    let path = sync_config_path()?;
    if !path.exists() {
        return Ok(SyncConfig::default());
    }
    let data = fs::read_to_string(&path).map_err(TuppError::Io)?;
    serde_json::from_str(&data).map_err(TuppError::Serialization)
}

pub fn save_sync_config(config: &SyncConfig) -> Result<(), TuppError> {
    let path = sync_config_path()?;
    let json = serde_json::to_string_pretty(config).map_err(TuppError::Serialization)?;
    fs::write(&path, json).map_err(TuppError::Io)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms).map_err(TuppError::Io)?;
    }

    Ok(())
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 3 <= bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn build_auth_url(client_id: &str) -> String {
    format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&access_type=offline&prompt=consent",
        AUTH_ENDPOINT,
        percent_encode(client_id),
        percent_encode(REDIRECT_URI),
        percent_encode(SCOPE),
    )
}

/// Accepts either a bare authorization code or the full (possibly
/// connection-refused) callback URL pasted from the browser's address bar.
pub fn extract_code(pasted: &str) -> String {
    let pasted = pasted.trim();
    let raw = if let Some(pos) = pasted.find("code=") {
        let after = &pasted[pos + "code=".len()..];
        match after.find('&') {
            Some(end) => &after[..end],
            None => after,
        }
    } else {
        pasted
    };
    percent_decode(raw)
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    #[serde(default)]
    refresh_token: Option<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    #[serde(default)]
    error: String,
    #[serde(default)]
    error_description: String,
}

fn describe_ureq_error(e: ureq::Error) -> TuppError {
    match e {
        ureq::Error::Status(code, response) => {
            let body = response.into_string().unwrap_or_default();
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&body) {
                TuppError::Other(format!(
                    "Google API error ({}): {} {}",
                    code, err.error, err.error_description
                ))
            } else {
                TuppError::Other(format!("Google API error ({}): {}", code, body))
            }
        }
        other => TuppError::Other(format!("Request to Google failed: {}", other)),
    }
}

pub fn exchange_code_for_tokens(
    config: &mut SyncConfig,
    code: &str,
) -> Result<(), TuppError> {
    let client_id = config
        .client_id
        .clone()
        .ok_or_else(|| TuppError::Other("Missing client id".to_string()))?;
    let client_secret = config
        .client_secret
        .clone()
        .ok_or_else(|| TuppError::Other("Missing client secret".to_string()))?;

    let response = ureq::post(TOKEN_ENDPOINT)
        .send_form(&[
            ("code", code),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("redirect_uri", REDIRECT_URI),
            ("grant_type", "authorization_code"),
        ])
        .map_err(describe_ureq_error)?;

    let token: TokenResponse = response
        .into_json()
        .map_err(|e| TuppError::Other(format!("Invalid token response: {}", e)))?;

    config.access_token = Some(token.access_token);
    config.expires_at = Some(now() + token.expires_in);
    if let Some(refresh_token) = token.refresh_token {
        config.refresh_token = Some(refresh_token);
    }

    save_sync_config(config)
}

fn refresh_access_token(config: &mut SyncConfig) -> Result<(), TuppError> {
    let client_id = config
        .client_id
        .clone()
        .ok_or_else(|| TuppError::Other("Missing client id".to_string()))?;
    let client_secret = config
        .client_secret
        .clone()
        .ok_or_else(|| TuppError::Other("Missing client secret".to_string()))?;
    let refresh_token = config
        .refresh_token
        .clone()
        .ok_or_else(|| TuppError::Other("No refresh token available".to_string()))?;

    let response = ureq::post(TOKEN_ENDPOINT)
        .send_form(&[
            ("refresh_token", refresh_token.as_str()),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("grant_type", "refresh_token"),
        ])
        .map_err(describe_ureq_error)?;

    let token: TokenResponse = response
        .into_json()
        .map_err(|e| TuppError::Other(format!("Invalid token response: {}", e)))?;

    config.access_token = Some(token.access_token);
    config.expires_at = Some(now() + token.expires_in);
    if let Some(refresh_token) = token.refresh_token {
        config.refresh_token = Some(refresh_token);
    }

    save_sync_config(config)
}

/// Returns a valid access token, silently refreshing it if needed. Meant for
/// `link`/`update`, which should not have to bother the user about auth.
pub fn ensure_valid_access_token(config: &mut SyncConfig) -> Result<String, TuppError> {
    if let (Some(token), Some(expires_at)) = (&config.access_token, config.expires_at) {
        if now() < expires_at - EXPIRY_BUFFER_SECS {
            return Ok(token.clone());
        }
    }

    if config.refresh_token.is_some() {
        refresh_access_token(config)?;
        if let Some(token) = &config.access_token {
            return Ok(token.clone());
        }
    }

    Err(TuppError::Other(
        "Not authenticated with Google, or the session expired. Run `tupp sync setup`."
            .to_string(),
    ))
}

pub fn is_configured(config: &SyncConfig) -> bool {
    config.client_id.is_some() && config.client_secret.is_some()
}

pub fn is_token_valid(config: &SyncConfig) -> bool {
    match (&config.access_token, config.expires_at) {
        (Some(_), Some(expires_at)) => now() < expires_at - EXPIRY_BUFFER_SECS,
        _ => false,
    }
}
