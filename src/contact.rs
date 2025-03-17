use serde::{ Deserialize, Serialize };
use std::fmt;
use std::collections::HashSet;
use dialoguer::{Input, Confirm};
use uuid::Uuid;

use crate::unwrap::UnwrapString;
use crate::models::*;
use crate::Group;

#[derive(Deserialize, Serialize, Debug)]
pub struct Contact {
    pub identifier: Uuid,
    pub identity: Identity,
    pub address: Option<Address>,
    pub email: Option<Vec<Email>>,
    pub phone: Option<Vec<PhoneNumber>>,
    pub social: Option<Vec<Social>>,
    pub groups: Option<HashSet<Group>>
}

impl Contact {
    pub fn format_name(&self, pattern: &str) -> String {
        let title = self.identity.title.unwrap_string();
        let first_name = self.identity.first_name.unwrap_string();
        let middle_name = self.identity.middle_name.unwrap_string();
        let last_name = self.identity.last_name.unwrap_string();
        let post_nominal = self.identity.post_nominal.unwrap_string();

        return pattern
            .replace("TITLE", &title)
            .replace("FIRST", &first_name)
            .replace("MIDDLE", &middle_name)
            .replace("LAST", &last_name)
            .replace("POST", &post_nominal);
    }

    // pub fn add_to_group(&self, group: Group) -> Result<String,String> {

    // }

    pub fn new_from_input() -> Self {

        let title: Option<String> = if Confirm::new()
            .with_prompt("Do you want to enter a title?")
            .default(false)
            .interact().unwrap() {
                Some(
                    Input::new()
                     .with_prompt("Title")
                     .interact_text().unwrap()
                )
            } else { None };

        let first_name: String = Input::new()
            .with_prompt("First Name")
            .interact_text().unwrap();

        let middle_name: Option<String> = if Confirm::new()
            .with_prompt("Do you want to enter a middle name?")
            .default(false)
            .interact().unwrap() {
                Some(
                    Input::new()
                     .with_prompt("Middle name")
                     .interact_text().unwrap()
                )
            } else { None };

        let last_name: String = Input::new()
            .with_prompt("Last Name")
            .interact_text().unwrap();

        let post_nominal: Option<String> = if Confirm::new()
            .with_prompt("Do you want to enter a post-nominal title?")
            .default(false)
            .interact().unwrap() {
                Some(
                    Input::new()
                     .with_prompt("Post-nominal")
                     .interact_text().unwrap()
                )
            } else { None };

        // let birth_date: Option<String> = if Confirm::new()
        //     .with_prompt("Do you want to enter a birth date?")
        //     .interact().unwrap() {
        //         Some(
        //             Input::new()
        //              .with_prompt("Birth Date (YYYY-MM-DD)")
        //              .interact_text().unwrap()
        //         )
        //     } else { None };

        // let birth_location: Option<String> = if Confirm::new()
        //     .with_prompt("Do you want to enter a birth location?")
        //     .interact().unwrap() {
        //         Some(
        //             Input::new()
        //             .with_prompt("Birth Location")
        //             .interact_text().unwrap()
        //         )
        //     } else { None };

        let email: Option<Vec<Email>> = if Confirm::new()
            .with_prompt("Do you want to enter an email?")
            .default(false)
            .interact().unwrap() {
                Some(vec![
                    Email {
                        label: if Confirm::new()
                        .with_prompt("Do you want to enter a label?")
                        .default(false)
                        .interact().unwrap() {
                            Some(
                                Input::new()
                                .with_prompt("Label")
                                .interact_text().unwrap()
                            )
                        } else { None },
                        address: Some(
                            Input::new()
                            .with_prompt("Email Address")
                            .interact_text().unwrap()
                        ),
                    }
                ])
            } else { None };

        let phone: Option<Vec<PhoneNumber>> = if Confirm::new()
            .with_prompt("Do you want to enter a phone number?")
            .default(true)
            .interact().unwrap() {
                Some(vec![
                    PhoneNumber {
                        label: if Confirm::new()
                        .with_prompt("Do you want to enter a label ?")
                        .default(false)
                        .interact().unwrap() {
                            Some(
                                Input::new()
                                .with_prompt("Label")
                                .interact_text().unwrap()
                            )
                        } else { Some("default".to_string()) },
                        country_code: Input::<u16>::new()
                            .with_prompt("Country Code")
                            .interact_text()
                            .unwrap(),
                        number: Input::<u32>::new()
                            .with_prompt("Phone Number")
                            .interact_text()
                            .unwrap(),
                    }
                ])
            } else { None };


        Contact {
            identifier: Uuid::new_v4(),
            identity: Identity {
                title,
                first_name: Some(first_name),
                middle_name,
                last_name: Some(last_name),
                post_nominal,
                birth_date: None,
                birth_location: None,
            },
            email,
            phone,
            address: None,
            social: None,
            groups: None,
        }
    }
}

impl fmt::Display for Contact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\tName: {} {}",
            self.identity.first_name.unwrap_string(),
            self.identity.last_name.unwrap_string()
        )?;
        if let Some(date) = &self.identity.birth_date {
            writeln!(
                f,
                "\tBirth Date: {}",
                date
            )?;
        }

        if let Some(birth_location) = &self.identity.birth_location {
            writeln!(
                f,
                "\tBirth Location: {}",
                birth_location
            )?;
        }

        if let Some(address) = &self.address {
            writeln!(
                f,
                "\tAddress: {}",
                address
            )?;
        }

        if let Some(emails) = &self.email {
            writeln!(f, "\tEmails:")?;
            for email in emails {
                writeln!(
                    f,
                    "\t  {}",
                    email
                )?;
            }
        }

        if let Some(phones) = &self.phone {
            writeln!(f, "\tPhone Numbers:")?;
            for phone in phones {
                writeln!(
                    f,
                    "\t  {}",
                    phone
                )?;
            }
        }

        if let Some(socials) = &self.social {
            writeln!(f, "\tSocial Networks:")?;
            for social in socials {
                writeln!(
                    f,
                    "\t  {}",
                    social
                )?;
            }
        }

        Ok(())
    }
}
