use serde::{ Deserialize, Serialize };
use std::fmt;
use std::collections::HashSet;
use dialoguer::{Input, Confirm, Select};
use uuid::Uuid;

use crate::unwrap::UnwrapString;
use crate::models::*;
use crate::Group;
use crate::interactions;

#[derive(Deserialize, Serialize, Debug)]
pub struct Contact {
    pub identifier: Uuid,
    pub identity: Identity,
    pub address: Option<Address>,
    pub emails: Option<Vec<Email>>,
    pub phones: Option<Vec<PhoneNumber>>,
    pub socials: Option<Vec<Social>>,
    pub groups: Option<HashSet<Uuid>>,
    pub links: Option<Vec<Link>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Link {
    pub target: Uuid,
    pub relation: Relation,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Relation {
    Friend,
    Child,
    Parent,
    Boss,
    Employee,
    Colleague,
    Partner,
    Spouse,
    Ex
}

impl Contact {
    pub fn format_name(&self, pattern: &str) -> String {
        let title = self.identity.title.unwrap_string();
        let first_name = self.identity.first_name.unwrap_string();
        let middle_name = self.identity.middle_name.unwrap_string();
        let last_name = self.identity.last_name.unwrap_string();
        let post_nominal = self.identity.post_nominal.unwrap_string();

        return crate::sanitize::trim_extra_spaces(
            &pattern
                .replace("TITLE", &title)
                .replace("FIRST", &first_name)
                .replace("MIDDLE", &middle_name)
                .replace("LAST", &last_name)
                .replace("POST", &post_nominal)
        );
    }

    pub fn new_from_input() -> Self {

        let gender: Gender = {
            let selection = Select::new()
                .with_prompt("Select gender")
                .items(&["Male", "Female", "Non-binary"])
                .default(0)
                .interact()
                .unwrap();
            
            match selection {
                0 => Gender::Male,
                1 => Gender::Female,
                2 => Gender::NonBinary,
                _ => Gender::Male, // Default fallback
            }
        };

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

        let emails: Option<Vec<Email>> = if Confirm::new()
            .with_prompt("Do you want to enter an email?")
            .default(false)
            .interact().unwrap() {
                Some(vec![interactions::create_email_interactive()])
            } else { None };

        let phones: Option<Vec<PhoneNumber>> = if Confirm::new()
            .with_prompt("Do you want to enter a phone number?")
            .default(true)
            .interact().unwrap() {
                Some(vec![interactions::create_phone_interactive()])
            } else { None };

        let socials: Option<Vec<Social>> = if Confirm::new()
            .with_prompt("Do you want to enter social media information?")
            .default(false)
            .interact().unwrap() {
                Some(vec![interactions::create_social_interactive()])
            } else { None };

        Contact {
            identifier: Uuid::new_v4(),
            identity: Identity {
                title,
                first_name: Some(first_name.clone()),
                middle_name: middle_name.clone(),
                last_name: Some(last_name.clone()),
                post_nominal,
                gender: Some(gender),
                birth_date: None,
                birth_location: None,
                birth_first_name: None,
                birth_middle_name: None,
                birth_last_name: None,
                is_alive: true,
                death_date: None,
                death_location: None,
            },
            emails,
            phones,
            address: None,
            socials,
            groups: None,
            links: None,
        }
    }

    pub fn add_social_interactive(&mut self) {
        interactions::add_social_to_contact(self);
    }

    pub fn add_birth_interactive(&mut self) {
        interactions::add_birth_to_contact(self);
    }

    pub fn add_death_interactive(&mut self) {
        interactions::add_death_to_contact(self);
    }

    pub fn add_gender_interactive(&mut self) {
        interactions::add_gender_to_contact(self);
    }

    pub fn add_email_interactive(&mut self) {
        interactions::add_email_to_contact(self);
    }

    pub fn add_phone_interactive(&mut self) {
        interactions::add_phone_to_contact(self);
    }

    pub fn add_link_interactive(&mut self, other_id: Uuid, relation_type: String) {
        if let Some(relation) = Self::parse_relation(&relation_type) {
            let new_link = Link {
                target: other_id,
                relation,
            };

            if let Some(ref mut links_vec) = self.links {
                links_vec.push(new_link);
            } else {
                self.links = Some(vec![new_link]);
            }
        } else {
            println!("Invalid relation type: {}. Valid types are: friend, child, parent, boss, employee, colleague, partner, spouse", relation_type);
        }
    }

    pub fn create_bidirectional_link(
        contact_a: &mut Contact,
        contact_b: &mut Contact,
        relation_type: String,
    ) -> Result<(), String> {
        if let Some(relation) = Self::parse_relation(&relation_type) {
            let reciprocal_relation = Self::get_reciprocal_relation(&relation);

            // Add link from A to B
            let link_a_to_b = Link {
                target: contact_b.identifier,
                relation,
            };

            if let Some(ref mut links_vec) = contact_a.links {
                links_vec.push(link_a_to_b);
            } else {
                contact_a.links = Some(vec![link_a_to_b]);
            }

            // Add reciprocal link from B to A
            let link_b_to_a = Link {
                target: contact_a.identifier,
                relation: reciprocal_relation,
            };

            if let Some(ref mut links_vec) = contact_b.links {
                links_vec.push(link_b_to_a);
            } else {
                contact_b.links = Some(vec![link_b_to_a]);
            }

            Ok(())
        } else {
            Err(format!(
                "Invalid relation type: {}. Valid types are: friend, child, parent, boss, employee, colleague, partner, spouse",
                relation_type
            ))
        }
    }

    fn parse_relation(relation_str: &str) -> Option<Relation> {
        match relation_str.to_lowercase().as_str() {
            "friend" => Some(Relation::Friend),
            "child" => Some(Relation::Child),
            "parent" => Some(Relation::Parent),
            "boss" => Some(Relation::Boss),
            "employee" => Some(Relation::Employee),
            "colleague" => Some(Relation::Colleague),
            "partner" => Some(Relation::Partner),
            "spouse" => Some(Relation::Spouse),
            "ex" => Some(Relation::Ex),
            _ => None,
        }
    }

    fn get_reciprocal_relation(relation: &Relation) -> Relation {
        match relation {
            Relation::Friend => Relation::Friend,
            Relation::Child => Relation::Parent,
            Relation::Parent => Relation::Child,
            Relation::Boss => Relation::Employee,
            Relation::Employee => Relation::Boss,
            Relation::Colleague => Relation::Colleague,
            Relation::Partner => Relation::Partner,
            Relation::Spouse => Relation::Spouse,
            Relation::Ex => Relation::Ex,
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

        if let Some(gender) = &self.identity.gender {
            writeln!(
                f,
                "\tGender: {}",
                match gender {
                    crate::models::Gender::Male => "Male",
                    crate::models::Gender::Female => "Female",
                    crate::models::Gender::NonBinary => "Non-binary",
                }
            )?;
        }

        writeln!(
            f,
            "\tAlive: {}",
            if self.identity.is_alive { "Yes" } else { "No" }
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

        if let Some(bfn) = &self.identity.birth_first_name {
            writeln!(
                f,
                "\tBirth First Name: {}",
                bfn
            )?;
        }

        if let Some(bmn) = &self.identity.birth_middle_name {
            writeln!(
                f,
                "\tBirth Middle Name: {}",
                bmn
            )?;
        }

        if let Some(bln) = &self.identity.birth_last_name {
            writeln!(
                f,
                "\tBirth Last Name: {}",
                bln
            )?;
        }

        if !self.identity.is_alive {
            if let Some(death_date) = &self.identity.death_date {
                writeln!(
                    f,
                    "\tDeath Date: {}",
                    death_date
                )?;
            }

            if let Some(death_location) = &self.identity.death_location {
                writeln!(
                    f,
                    "\tDeath Location: {}",
                    death_location
                )?;
            }
        }

        if let Some(address) = &self.address {
            writeln!(
                f,
                "\tAddress: {}",
                address
            )?;
        }

        if let Some(emails) = &self.emails {
            writeln!(f, "\tEmails:")?;
            for email in emails {
                writeln!(
                    f,
                    "\t  {}",
                    email
                )?;
            }
        }

        if let Some(phones) = &self.phones {
            writeln!(f, "\tPhone Numbers:")?;
            for phone in phones {
                writeln!(
                    f,
                    "\t  {}",
                    phone
                )?;
            }
        }

        if let Some(socials) = &self.socials {
            writeln!(f, "\tSocial Networks:")?;
            for social in socials {
                writeln!(
                    f,
                    "\t  {}",
                    social
                )?;
            }
        }

        if let Some(links) = &self.links {
            writeln!(f, "\tRelationships:")?;
            for link in links {
                writeln!(
                    f,
                    "\t  {}",
                    link
                )?;
            }
        }

        if let Some(groups) = &self.groups {
            writeln!(f, "\tGroups:")?;
            for group_id in groups {
                writeln!(f, "\t  {}", group_id)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Relation::Friend => write!(f, "Friend"),
            Relation::Child => write!(f, "Child"),
            Relation::Parent => write!(f, "Parent"),
            Relation::Boss => write!(f, "Boss"),
            Relation::Employee => write!(f, "Employee"),
            Relation::Colleague => write!(f, "Colleague"),
            Relation::Partner => write!(f, "Partner"),
            Relation::Spouse => write!(f, "Spouse"),
            Relation::Ex => write!(f, "Ex-partner"),
        }
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.relation, self.target)
    }
}
