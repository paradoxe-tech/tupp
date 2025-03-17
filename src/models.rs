use crate::unwrap::UnwrapString;
use serde::{ Deserialize, Serialize };
use std::fmt;

/* BASIC STRUCTURES */

#[derive(Deserialize, Serialize, Debug)]
pub struct Company {
    pub name: Option<String>,
    pub position: Option<String>,
    pub address: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Email {
    pub label: Option<String>,
    pub address: Option<String>,
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} : {}",
            self.label.as_deref().unwrap_or_default(),
            self.address.as_deref().unwrap_or_default()
        )?;
        return Ok(());
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PhoneNumber {
    pub label: Option<String>,
    pub country_code: u16,
    pub number: u32,
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: +{} {}",
            self.label.as_deref().unwrap_or("N/A"),
            self.country_code,
            self.number
        )?;
        return Ok(());
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Social {
    pub network: String,
    pub username: Option<String>,
}

impl fmt::Display for Social {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.network,
            self.username.as_deref().unwrap_or("N/A")
        )?;
        return Ok(());
    }
}

/* ADDRESS DEF, DISPLAY & DEFAULT */

#[derive(Deserialize, Serialize, Debug)]
pub struct Address {
    pub label: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub post_code: Option<String>,
    pub street: Option<String>,
    pub number: Option<String>,
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} ; {} {}, {}",
            self.number.unwrap_string(),
            self.street.unwrap_string(),
            self.post_code.unwrap_string(),
            self.city.unwrap_string(),
            self.country.unwrap_string()
        )?;

        return Ok(());
    }
}

impl Default for Address {
    fn default() -> Self {
        Self {
            label: Some("Unknown".to_string()),
            country: Some("France".to_string()),
            region: Some("Poitou-Charentes".to_string()),
            city: Some("Poitiers".to_string()),
            post_code: Some("98400".to_string()),
            street: Some("Rue de la Coupe du Monde".to_string()),
            number: Some("1998".to_string()),
        }
    }
}

/* DATE DEF, DISPLAY & DEFAULT */

#[derive(Deserialize, Serialize, Debug)]
pub struct Date {
    pub year: Option<i32>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{}-{}",
            self.year.unwrap_or_default(),
            self.month.unwrap_or_default(),
            self.day.unwrap_or_default()
        )?;

        return Ok(());
    }
}

impl Default for Date {
    fn default() -> Self {
        Self {
            year: Some(1944),
            month: Some(6),
            day: Some(18),
            hour: Some(6),
            minute: Some(9),
            second: Some(0),
        }
    }
}

/* HOLDER STRUCTURE : Identity */

#[derive(Deserialize, Serialize, Debug)]
pub struct Identity {
    pub title: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub first_name: Option<String>,
    pub post_nominal: Option<String>,
    pub birth_date: Option<Date>,
    pub birth_location: Option<Address>,
}
