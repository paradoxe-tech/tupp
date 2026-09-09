use std::collections::HashSet;

use uuid::Uuid;

use crate::contact::Contact;
use crate::institution::Institution;
use crate::models::{Address, Date, Email, PhoneNumber};

use super::people::{GAddress, GDate, GoogleContact};

/* ---------------------------------------------------------------------- */
/* Normalization                                                          */
/* ---------------------------------------------------------------------- */

pub fn normalize_email(raw: &str) -> String {
    raw.trim().to_lowercase()
}

/// Digits-only, country-code-prefixed representation, used only to compare
/// two phone numbers for equality (never to recover the split between
/// country code and national number — see `split_phone_for_tupp`).
pub fn normalize_phone(raw: &str, default_region_prefix: Option<u16>) -> Option<String> {
    let digits: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();
    if digits.is_empty() {
        return None;
    }

    if let Some(rest) = digits.strip_prefix('+') {
        if rest.is_empty() {
            return None;
        }
        Some(rest.to_string())
    } else {
        let prefix = default_region_prefix?;
        let national = digits.strip_prefix('0').unwrap_or(&digits);
        if national.is_empty() {
            return None;
        }
        Some(format!("{}{}", prefix, national))
    }
}

pub fn normalize_tupp_phone(country_code: u16, number: u32) -> String {
    format!("{}{}", country_code, number)
}

/// Splits a free-text Google phone number into (country_code, number) so it
/// can be stored as a tupp `PhoneNumber`. Only numbers in the configured
/// default region, or already in `+<code>` form matching it, can be split
/// unambiguously without a full calling-code table — anything else returns
/// `None` and is left for the user to add manually.
pub fn split_phone_for_tupp(raw: &str, default_region_prefix: Option<u16>) -> Option<(u16, u32)> {
    let prefix = default_region_prefix?;
    let digits: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();

    let national = if let Some(rest) = digits.strip_prefix('+') {
        rest.strip_prefix(&prefix.to_string())?
    } else {
        digits.strip_prefix('0').unwrap_or(&digits)
    };

    if national.is_empty() {
        return None;
    }
    national.parse::<u32>().ok().map(|number| (prefix, number))
}

/* ---------------------------------------------------------------------- */
/* Social network URL <-> (network, username) mapping                     */
/* ---------------------------------------------------------------------- */

const SOCIAL_DOMAINS: &[(&str, &str)] = &[
    ("twitter.com", "Twitter"),
    ("x.com", "Twitter"),
    ("instagram.com", "Instagram"),
    ("facebook.com", "Facebook"),
    ("linkedin.com", "LinkedIn"),
    ("github.com", "GitHub"),
];

/// Best-effort parse of a Google contact URL into a (network, username) pair.
pub fn guess_social_from_url(url: &str) -> Option<(String, String)> {
    let without_scheme = url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");

    let (host, path) = match without_scheme.split_once('/') {
        Some((h, p)) => (h, p),
        None => (without_scheme, ""),
    };

    let network = SOCIAL_DOMAINS
        .iter()
        .find(|(domain, _)| host.eq_ignore_ascii_case(domain))
        .map(|(_, name)| *name)?;

    let username = path
        .trim_start_matches("in/")
        .trim_matches('/')
        .split(&['/', '?'][..])
        .next()
        .unwrap_or("")
        .to_string();

    if username.is_empty() {
        None
    } else {
        Some((network.to_string(), username))
    }
}

/* ---------------------------------------------------------------------- */
/* Exact matching (email / phone)                                         */
/* ---------------------------------------------------------------------- */

/// Returns the single tupp contact whose email or phone exactly matches one
/// of `google`'s, if there is exactly one such contact among `candidates`.
pub fn find_exact_match(
    google: &GoogleContact,
    candidates: &[&Contact],
    default_region_prefix: Option<u16>,
) -> Option<Uuid> {
    let google_emails: HashSet<String> = google
        .email_addresses
        .iter()
        .filter_map(|e| e.value.as_deref())
        .map(normalize_email)
        .collect();
    let google_phones: HashSet<String> = google
        .phone_numbers
        .iter()
        .filter_map(|p| p.value.as_deref())
        .filter_map(|raw| normalize_phone(raw, default_region_prefix))
        .collect();

    let mut matches: HashSet<Uuid> = HashSet::new();
    for contact in candidates {
        let has_email_match = contact
            .emails
            .iter()
            .flatten()
            .filter_map(|e| e.address.as_deref())
            .map(normalize_email)
            .any(|e| google_emails.contains(&e));

        let has_phone_match = contact.phones.iter().flatten().any(|p| {
            google_phones.contains(&normalize_tupp_phone(p.country_code, p.number))
        });

        if has_email_match || has_phone_match {
            matches.insert(contact.identifier);
        }
    }

    if matches.len() == 1 {
        matches.into_iter().next()
    } else {
        None
    }
}

/* ---------------------------------------------------------------------- */
/* Heuristic scoring                                                      */
/* ---------------------------------------------------------------------- */

pub struct MatchCandidate {
    pub tupp_id: Uuid,
    pub score: f64,
    pub reasons: Vec<String>,
}

fn best_similarity<'a>(a: impl Iterator<Item = String>, b: impl Iterator<Item = String> + Clone) -> f64 {
    let mut best = 0.0f64;
    for x in a {
        for y in b.clone() {
            let s = strsim::jaro_winkler(&x, &y);
            if s > best {
                best = s;
            }
        }
    }
    best
}

pub fn score_candidate(
    google: &GoogleContact,
    tupp: &Contact,
    institutions: &[Institution],
) -> MatchCandidate {
    let mut reasons = Vec::new();

    let google_name = google.display_name().to_lowercase();
    let tupp_name = tupp.format_name("FIRST MIDDLE LAST").to_lowercase();
    let name_score = if google_name.is_empty() || tupp_name.is_empty() {
        0.0
    } else {
        strsim::jaro_winkler(&google_name, &tupp_name)
    };
    if name_score > 0.0 {
        reasons.push(format!("name similarity {:.0}%", name_score * 100.0));
    }

    let google_addresses = google.addresses.iter().map(normalize_address_google);
    let tupp_addresses: Vec<String> = tupp
        .addresses
        .iter()
        .flatten()
        .map(normalize_address_tupp)
        .collect();
    let address_score = if tupp_addresses.is_empty() {
        0.0
    } else {
        best_similarity(google_addresses, tupp_addresses.into_iter())
    };
    if address_score > 0.0 {
        reasons.push(format!("address similarity {:.0}%", address_score * 100.0));
    }

    let google_socials: Vec<String> = google
        .urls
        .iter()
        .filter_map(|u| u.value.as_deref())
        .filter_map(guess_social_from_url)
        .map(|(network, username)| format!("{}:{}", network.to_lowercase(), username.to_lowercase()))
        .collect();
    let tupp_socials: HashSet<String> = tupp
        .socials
        .iter()
        .flatten()
        .filter_map(|s| s.username.as_ref().map(|u| (s.network.clone(), u.clone())))
        .map(|(network, username)| format!("{}:{}", network.to_lowercase(), username.to_lowercase()))
        .collect();
    let social_score = if google_socials.iter().any(|s| tupp_socials.contains(s)) {
        1.0
    } else {
        0.0
    };
    if social_score > 0.0 {
        reasons.push("matching social profile".to_string());
    }

    let tupp_institution_names: Vec<String> = tupp
        .positions
        .iter()
        .flatten()
        .filter_map(|p| Institution::find_institution_by_id_recursive(institutions, &p.institution))
        .map(|i| i.name.to_lowercase())
        .collect();
    let google_orgs = google
        .organizations
        .iter()
        .filter_map(|o| o.name.as_deref())
        .map(|s| s.to_lowercase());
    let org_score = if tupp_institution_names.is_empty() {
        0.0
    } else {
        best_similarity(google_orgs, tupp_institution_names.into_iter())
    };
    if org_score > 0.0 {
        reasons.push(format!("organization similarity {:.0}%", org_score * 100.0));
    }

    let score = 0.5 * name_score + 0.2 * address_score + 0.15 * social_score + 0.15 * org_score;

    MatchCandidate {
        tupp_id: tupp.identifier,
        score,
        reasons,
    }
}

fn normalize_address_tupp(a: &Address) -> String {
    [&a.number, &a.street, &a.post_code, &a.city, &a.region, &a.country]
        .into_iter()
        .filter_map(|f| f.as_deref())
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_address_google(a: &GAddress) -> String {
    [&a.street_address, &a.postal_code, &a.city, &a.region, &a.country]
        .into_iter()
        .filter_map(|f| f.as_deref())
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/* ---------------------------------------------------------------------- */
/* Update diff                                                            */
/* ---------------------------------------------------------------------- */

pub enum TuppUpdate {
    BirthDate(Date),
    Email(Email),
    Phone(PhoneNumber),
}

pub enum GoogleUpdate {
    Birthday(GDate),
    Email(String),
    Phone(String),
}

pub struct ContactDiff {
    pub tupp_id: Uuid,
    pub resource_name: String,
    pub label: String,
    pub tupp_updates: Vec<TuppUpdate>,
    pub google_updates: Vec<GoogleUpdate>,
    pub conflicts: Vec<String>,
}

impl ContactDiff {
    pub fn is_empty(&self) -> bool {
        self.tupp_updates.is_empty() && self.google_updates.is_empty()
    }
}

pub fn diff(google: &GoogleContact, tupp: &Contact, default_region_prefix: Option<u16>) -> ContactDiff {
    let mut tupp_updates = Vec::new();
    let mut google_updates = Vec::new();
    let mut conflicts = Vec::new();

    let t_date = tupp.identity.birth_date.as_ref().map(|d| (d.year, d.month, d.day));
    let g_date = google
        .birthdays
        .first()
        .and_then(|b| b.date.as_ref())
        .map(|d| (d.year, d.month, d.day));
    match (t_date, g_date) {
        (Some(tv), Some(gv)) if tv != gv => conflicts.push(format!(
            "birth date: tupp has {:?}, google has {:?} — left untouched",
            tv, gv
        )),
        (Some(tv), None) => google_updates.push(GoogleUpdate::Birthday(GDate {
            year: tv.0,
            month: tv.1,
            day: tv.2,
        })),
        (None, Some(gv)) => tupp_updates.push(TuppUpdate::BirthDate(Date {
            year: gv.0,
            month: gv.1,
            day: gv.2,
            hour: None,
            minute: None,
            second: None,
        })),
        _ => {}
    }

    // Emails: plain union, no contradiction concept for list fields.
    let tupp_emails: HashSet<String> = tupp
        .emails
        .iter()
        .flatten()
        .filter_map(|e| e.address.as_deref())
        .map(normalize_email)
        .collect();
    for ge in &google.email_addresses {
        if let Some(addr) = &ge.value {
            if !normalize_email(addr).is_empty() && !tupp_emails.contains(&normalize_email(addr)) {
                tupp_updates.push(TuppUpdate::Email(Email {
                    label: ge.r#type.clone(),
                    address: Some(addr.clone()),
                }));
            }
        }
    }
    let google_emails: HashSet<String> = google
        .email_addresses
        .iter()
        .filter_map(|e| e.value.as_deref())
        .map(normalize_email)
        .collect();
    for te in tupp.emails.iter().flatten() {
        if let Some(addr) = &te.address {
            if !normalize_email(addr).is_empty() && !google_emails.contains(&normalize_email(addr)) {
                google_updates.push(GoogleUpdate::Email(addr.clone()));
            }
        }
    }

    // Phones: plain union.
    let tupp_phones: HashSet<String> = tupp
        .phones
        .iter()
        .flatten()
        .map(|p| normalize_tupp_phone(p.country_code, p.number))
        .collect();
    for gp in &google.phone_numbers {
        if let Some(raw) = &gp.value {
            if let Some(norm) = normalize_phone(raw, default_region_prefix) {
                if !tupp_phones.contains(&norm) {
                    if let Some((country_code, number)) = split_phone_for_tupp(raw, default_region_prefix) {
                        tupp_updates.push(TuppUpdate::Phone(PhoneNumber {
                            label: gp.r#type.clone(),
                            country_code,
                            number,
                        }));
                    }
                }
            }
        }
    }
    let google_phones: HashSet<String> = google
        .phone_numbers
        .iter()
        .filter_map(|p| p.value.as_deref())
        .filter_map(|raw| normalize_phone(raw, default_region_prefix))
        .collect();
    for tp in tupp.phones.iter().flatten() {
        let norm = normalize_tupp_phone(tp.country_code, tp.number);
        if !google_phones.contains(&norm) {
            google_updates.push(GoogleUpdate::Phone(format!(
                "+{}{}",
                tp.country_code, tp.number
            )));
        }
    }

    ContactDiff {
        tupp_id: tupp.identifier,
        resource_name: google.resource_name.clone(),
        label: tupp.format_name("FIRST LAST"),
        tupp_updates,
        google_updates,
        conflicts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_phone_with_plus() {
        assert_eq!(normalize_phone("+33 6 12 34 56 78", None), Some("33612345678".to_string()));
    }

    #[test]
    fn normalize_phone_without_plus_uses_default_region() {
        assert_eq!(normalize_phone("06 12 34 56 78", Some(33)), Some("33612345678".to_string()));
    }

    #[test]
    fn normalize_phone_without_plus_and_no_default_fails() {
        assert_eq!(normalize_phone("06 12 34 56 78", None), None);
    }

    #[test]
    fn split_phone_for_tupp_national_number() {
        assert_eq!(split_phone_for_tupp("06 12 34 56 78", Some(33)), Some((33, 612345678)));
    }

    #[test]
    fn split_phone_for_tupp_matching_intl_prefix() {
        assert_eq!(split_phone_for_tupp("+33612345678", Some(33)), Some((33, 612345678)));
    }

    #[test]
    fn split_phone_for_tupp_foreign_number_unresolved() {
        assert_eq!(split_phone_for_tupp("+14155552671", Some(33)), None);
    }

    #[test]
    fn guess_social_from_url_recognizes_known_domains() {
        assert_eq!(
            guess_social_from_url("https://www.linkedin.com/in/janedoe"),
            Some(("LinkedIn".to_string(), "janedoe".to_string()))
        );
        assert_eq!(
            guess_social_from_url("https://example.com/janedoe"),
            None
        );
    }
}
