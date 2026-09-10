use std::collections::HashSet;
use std::path::PathBuf;

use dialoguer::{Confirm, Input, Password, Select};
use uuid::Uuid;

use crate::cli::SyncCommand;
use crate::contact::Contact;
use crate::error::TuppError;
use crate::google::matching::{self, ContactDiff, GoogleUpdate, MatchCandidate, TuppUpdate};
use crate::google::people::{self, GBirthday, GEmail, GPhone, GoogleContact};
use crate::google::{auth, SyncConfig};
use crate::models::TuppData;
use crate::storage::save_data;

pub fn handle_sync_command(
    command: SyncCommand,
    data: &mut TuppData,
    file_path: &PathBuf,
) -> Result<(), TuppError> {
    match command {
        SyncCommand::Setup => setup(),
        SyncCommand::Link { heuristic } => link(data, heuristic),
        SyncCommand::Update => update(data, file_path),
    }
}

fn prompt_credentials(config: &mut SyncConfig) {
    let client_id: String = Input::new()
        .with_prompt("Google OAuth client ID")
        .interact_text()
        .unwrap();
    let client_secret: String = Password::new()
        .with_prompt("Google OAuth client secret")
        .interact()
        .unwrap();
    config.client_id = Some(client_id);
    config.client_secret = Some(client_secret);
}

fn setup() -> Result<(), TuppError> {
    let mut config = auth::load_sync_config()?;

    if !auth::is_configured(&config) {
        prompt_credentials(&mut config);
    } else if auth::is_token_valid(&config) {
        println!("Already set up, and the access token is still valid.");
        return Ok(());
    } else {
        let choice = Select::new()
            .with_prompt("Your Google session has expired or was never established. What do you want to do?")
            .items(&["Change credentials", "Just log in again"])
            .default(1)
            .interact()
            .unwrap();

        if choice == 0 {
            prompt_credentials(&mut config);
        }
    }

    let client_id = config
        .client_id
        .clone()
        .ok_or_else(|| TuppError::Other("Missing client id".to_string()))?;
    let auth_url = auth::build_auth_url(&client_id);

    println!(
        "\nOpen this URL in your browser, log in, and grant access to tupp:\n\n{}\n",
        auth_url
    );
    println!(
        "The page will likely fail to load once you grant access — that's expected. \
         Paste the resulting URL from your browser's address bar (or just the `code` value) below."
    );

    let pasted: String = Input::new()
        .with_prompt("Callback URL or code")
        .interact_text()
        .unwrap();

    let code = auth::extract_code(&pasted);
    auth::exchange_code_for_tokens(&mut config, &code)?;

    println!("tupp is now linked to your Google account.");
    Ok(())
}

fn link(data: &mut TuppData, heuristic: bool) -> Result<(), TuppError> {
    let mut config = auth::load_sync_config()?;
    let token = auth::ensure_valid_access_token(&mut config)?;

    if config.default_region_prefix.is_none() {
        let prefix: u16 = Input::new()
            .with_prompt("Default country calling code for phone numbers without one (e.g. 33 for France)")
            .interact_text()
            .unwrap();
        config.default_region_prefix = Some(prefix);
        auth::save_sync_config(&config)?;
    }
    let default_region_prefix = config.default_region_prefix;

    println!("Fetching Google Contacts...");
    let google_contacts = people::list_contacts(&token)?;

    let mut linked_google: HashSet<String> = HashSet::new();
    let mut linked_tupp: HashSet<Uuid> = HashSet::new();
    for gc in &google_contacts {
        if let Some(id) = gc.tuppsync_id() {
            linked_google.insert(gc.resource_name.clone());
            linked_tupp.insert(id);
        }
    }

    let mut exact_count = 0;
    for gc in &google_contacts {
        if linked_google.contains(&gc.resource_name) {
            continue;
        }
        let candidates: Vec<&Contact> = data
            .contacts
            .iter()
            .filter(|c| !linked_tupp.contains(&c.identifier))
            .collect();

        if let Some(tupp_id) = matching::find_exact_match(gc, &candidates, default_region_prefix) {
            link_pair(&token, gc, tupp_id)?;
            linked_google.insert(gc.resource_name.clone());
            linked_tupp.insert(tupp_id);
            exact_count += 1;
            println!("Linked '{}' <-> tupp contact {}", gc.display_name(), tupp_id);
        }
    }
    println!("Exact linking done: {} contact(s) linked.", exact_count);

    if heuristic {
        let mut heuristic_count = 0;
        for gc in &google_contacts {
            if linked_google.contains(&gc.resource_name) {
                continue;
            }
            let candidates: Vec<&Contact> = data
                .contacts
                .iter()
                .filter(|c| !linked_tupp.contains(&c.identifier))
                .collect();
            if candidates.is_empty() {
                break;
            }

            let mut scored: Vec<MatchCandidate> = candidates
                .iter()
                .map(|c| matching::score_candidate(gc, c, &data.institutions))
                .collect();
            scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

            if let Some(best) = scored.first() {
                const MIN_SCORE: f64 = 0.35;
                if best.score >= MIN_SCORE {
                    let tupp_contact = data
                        .contacts
                        .iter()
                        .find(|c| c.identifier == best.tupp_id)
                        .unwrap();
                    println!("\nGoogle contact: {}", gc.display_name());
                    println!(
                        "Tupp contact:   {}",
                        tupp_contact.format_name("TITLE FIRST MIDDLE LAST POST")
                    );
                    println!(
                        "Match score: {:.0}% ({})",
                        best.score * 100.0,
                        if best.reasons.is_empty() {
                            "no specific signal".to_string()
                        } else {
                            best.reasons.join(", ")
                        }
                    );
                    let confirmed = Confirm::new()
                        .with_prompt("Link these two contacts?")
                        .default(false)
                        .interact()
                        .unwrap();
                    if confirmed {
                        link_pair(&token, gc, best.tupp_id)?;
                        linked_google.insert(gc.resource_name.clone());
                        linked_tupp.insert(best.tupp_id);
                        heuristic_count += 1;
                    }
                }
            }
        }
        println!("Heuristic linking done: {} contact(s) linked.", heuristic_count);
    }

    Ok(())
}

fn link_pair(token: &str, gc: &GoogleContact, tupp_id: Uuid) -> Result<(), TuppError> {
    let mut note = gc.note().unwrap_or("").to_string();
    if !note.trim().is_empty() {
        note.push('\n');
    }
    note.push_str(&format!("tuppsync-id: {}", tupp_id));

    let body = serde_json::json!({
        "etag": gc.etag,
        "biographies": [{ "value": note, "contentType": "TEXT_PLAIN" }],
    });
    people::patch_contact(token, &gc.resource_name, &["biographies"], body)
}

fn update(data: &mut TuppData, file_path: &PathBuf) -> Result<(), TuppError> {
    let mut config = auth::load_sync_config()?;
    let token = auth::ensure_valid_access_token(&mut config)?;
    let default_region_prefix = config.default_region_prefix;

    println!("Fetching Google Contacts...");
    let google_contacts = people::list_contacts(&token)?;

    let mut diffs: Vec<ContactDiff> = Vec::new();
    let mut photo_downloads: Vec<(Uuid, String, String)> = Vec::new(); // (tupp_id, url, contact label)
    for gc in &google_contacts {
        if let Some(tupp_id) = gc.tuppsync_id() {
            if let Some(tupp_contact) = data.contacts.iter().find(|c| c.identifier == tupp_id) {
                let d = matching::diff(gc, tupp_contact, default_region_prefix);
                if !d.is_empty() || !d.conflicts.is_empty() {
                    diffs.push(d);
                }

                if !crate::storage::has_photo(&tupp_id) {
                    if let Some(url) = gc.photo_url() {
                        photo_downloads.push((tupp_id, url.to_string(), gc.display_name()));
                    }
                }
            }
        }
    }

    if diffs.is_empty() && photo_downloads.is_empty() {
        println!("Everything is already in sync (no linked contacts had anything to add).");
        return Ok(());
    }

    let mut tupp_change_count = 0;
    let mut google_change_count = 0;
    println!("\nPlanned changes:\n");
    for d in &diffs {
        println!("== {} ==", d.label);
        for u in &d.tupp_updates {
            println!("  + tupp: {}", describe_tupp_update(u));
        }
        for u in &d.google_updates {
            println!("  + google: {}", describe_google_update(u));
        }
        for c in &d.conflicts {
            println!("  ! conflict (not applied): {}", c);
        }
        tupp_change_count += d.tupp_updates.len();
        google_change_count += d.google_updates.len();
    }

    if !photo_downloads.is_empty() {
        println!("Photos to download:");
        for (_, _, label) in &photo_downloads {
            println!("  + photo for {}", label);
        }
    }

    if tupp_change_count == 0 && google_change_count == 0 && photo_downloads.is_empty() {
        println!("\nNo non-conflicting changes to apply.");
        return Ok(());
    }

    println!();
    let confirmed = Confirm::new()
        .with_prompt(format!(
            "Apply {} change(s) to tupp, {} change(s) to Google Contacts, and download {} photo(s)?",
            tupp_change_count, google_change_count, photo_downloads.len()
        ))
        .default(false)
        .interact()
        .unwrap();

    if !confirmed {
        println!("Aborted, nothing changed.");
        return Ok(());
    }

    for d in diffs {
        if !d.tupp_updates.is_empty() {
            if let Some(contact) = data.contacts.iter_mut().find(|c| c.identifier == d.tupp_id) {
                apply_tupp_updates(contact, d.tupp_updates);
            }
        }
        if !d.google_updates.is_empty() {
            apply_google_updates(&token, &google_contacts, &d.resource_name, d.google_updates)?;
        }
    }

    for (tupp_id, url, label) in photo_downloads {
        match people::download_photo(&url).and_then(|(bytes, content_type)| {
            crate::storage::save_photo(&tupp_id, &bytes, &content_type)
        }) {
            Ok(_) => println!("Downloaded photo for {}.", label),
            Err(e) => println!("Failed to download photo for {}: {}", label, e),
        }
    }

    save_data(file_path, data)?;
    println!("Sync complete.");
    Ok(())
}

fn apply_tupp_updates(contact: &mut Contact, updates: Vec<TuppUpdate>) {
    for u in updates {
        match u {
            TuppUpdate::BirthDate(d) => contact.identity.birth_date = Some(d),
            TuppUpdate::Email(e) => contact.emails.get_or_insert_with(Vec::new).push(e),
            TuppUpdate::Phone(p) => contact.phones.get_or_insert_with(Vec::new).push(p),
        }
    }
}

fn apply_google_updates(
    token: &str,
    google_contacts: &[GoogleContact],
    resource_name: &str,
    updates: Vec<GoogleUpdate>,
) -> Result<(), TuppError> {
    let gc = google_contacts
        .iter()
        .find(|g| g.resource_name == resource_name)
        .ok_or_else(|| TuppError::Other(format!("Google contact {} disappeared mid-run", resource_name)))?;

    let mut emails = gc.email_addresses.clone();
    let mut phones = gc.phone_numbers.clone();
    let mut birthdays = gc.birthdays.clone();

    let mut fields: HashSet<&'static str> = HashSet::new();

    for u in updates {
        match u {
            GoogleUpdate::Birthday(d) => {
                birthdays = vec![GBirthday { date: Some(d) }];
                fields.insert("birthdays");
            }
            GoogleUpdate::Email(v) => {
                emails.push(GEmail {
                    value: Some(v),
                    r#type: None,
                });
                fields.insert("emailAddresses");
            }
            GoogleUpdate::Phone(v) => {
                phones.push(GPhone {
                    value: Some(v),
                    canonical_form: None,
                    r#type: None,
                });
                fields.insert("phoneNumbers");
            }
        }
    }

    let mut body = serde_json::json!({ "etag": gc.etag });
    if fields.contains("birthdays") {
        body["birthdays"] = serde_json::to_value(&birthdays).unwrap();
    }
    if fields.contains("emailAddresses") {
        body["emailAddresses"] = serde_json::to_value(&emails).unwrap();
    }
    if fields.contains("phoneNumbers") {
        body["phoneNumbers"] = serde_json::to_value(&phones).unwrap();
    }

    let field_list: Vec<&str> = fields.into_iter().collect();
    people::patch_contact(token, resource_name, &field_list, body)
}

fn describe_tupp_update(u: &TuppUpdate) -> String {
    match u {
        TuppUpdate::BirthDate(d) => format!("birth date = {}", d),
        TuppUpdate::Email(e) => format!("email {}", e.address.as_deref().unwrap_or("")),
        TuppUpdate::Phone(p) => format!("phone +{}{}", p.country_code, p.number),
    }
}

fn describe_google_update(u: &GoogleUpdate) -> String {
    match u {
        GoogleUpdate::Birthday(d) => format!(
            "birthday = {}-{}-{}",
            d.year.map(|y| y.to_string()).unwrap_or_default(),
            d.month.map(|m| m.to_string()).unwrap_or_default(),
            d.day.map(|day| day.to_string()).unwrap_or_default()
        ),
        GoogleUpdate::Email(v) => format!("email {}", v),
        GoogleUpdate::Phone(v) => format!("phone {}", v),
    }
}
