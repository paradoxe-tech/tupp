use dialoguer::{Input, Confirm, Select};
use uuid::Uuid;
use crate::contact::Contact;
use crate::models::{Identity, Gender, Email, PhoneNumber, Social};
use crate::interactions;
use crate::error::Result;

pub fn create_contact_interactive(
    title_f: Option<String>,
    first_name_f: Option<String>,
    middle_name_f: Option<String>,
    last_name_f: Option<String>,
    post_nominal_f: Option<String>,
    gender_f: Option<String>,
) -> Result<Contact> {
    let bypass = first_name_f.is_some() && last_name_f.is_some();

    let gender_val: Gender = if let Some(g_str) = &gender_f {
        match g_str.to_lowercase().as_str() {
            "male" => Gender::Male,
            "female" => Gender::Female,
            "non-binary" | "nonbinary" => Gender::NonBinary,
            _ => {
                if bypass {
                    Gender::Male
                } else {
                    println!("Invalid gender '{}', falling back to interactive selection.", g_str);
                    let selection = Select::new()
                        .with_prompt("Select gender")
                        .items(&["Male", "Female", "Non-binary"])
                        .default(0)
                        .interact()?;
                    
                    match selection {
                        0 => Gender::Male,
                        1 => Gender::Female,
                        2 => Gender::NonBinary,
                        _ => Gender::Male,
                    }
                }
            }
        }
    } else if bypass {
        Gender::Male
    } else {
        let selection = Select::new()
            .with_prompt("Select gender")
            .items(&["Male", "Female", "Non-binary"])
            .default(0)
            .interact()?;
        
        match selection {
            0 => Gender::Male,
            1 => Gender::Female,
            2 => Gender::NonBinary,
            _ => Gender::Male, // Default fallback
        }
    };

    let title: Option<String> = if title_f.is_some() {
        title_f
    } else if bypass {
        None
    } else if Confirm::new()
        .with_prompt("Do you want to enter a title?")
        .default(false)
        .interact()? {
            Some(
                Input::new()
                    .with_prompt("Title")
                    .interact_text()?
            )
        } else { None };

    let first_name_val: String = if let Some(fn_val) = first_name_f {
        fn_val
    } else {
        Input::new()
            .with_prompt("First Name")
            .interact_text()?
    };

    let middle_name: Option<String> = if middle_name_f.is_some() {
        middle_name_f
    } else if bypass {
        None
    } else if Confirm::new()
        .with_prompt("Do you want to enter a middle name?")
        .default(false)
        .interact()? {
            Some(
                Input::new()
                    .with_prompt("Middle name")
                    .interact_text()?
            )
        } else { None };

    let last_name_val: String = if let Some(ln_val) = last_name_f {
        ln_val
    } else {
        Input::new()
            .with_prompt("Last Name")
            .interact_text()?
    };

    let post_nominal: Option<String> = if post_nominal_f.is_some() {
        post_nominal_f
    } else if bypass {
        None
    } else if Confirm::new()
        .with_prompt("Do you want to enter a post-nominal title?")
        .default(false)
        .interact()? {
            Some(
                Input::new()
                    .with_prompt("Post-nominal")
                    .interact_text()?
            )
        } else { None };

    let emails: Option<Vec<Email>> = if bypass {
        None
    } else if Confirm::new()
        .with_prompt("Do you want to enter an email?")
        .default(false)
        .interact()? {
            Some(vec![interactions::email::create_email_interactive(&[])])
        } else { None };

    let phones: Option<Vec<PhoneNumber>> = if bypass {
        None
    } else if Confirm::new()
        .with_prompt("Do you want to enter a phone number?")
        .default(true)
        .interact()? {
            Some(vec![interactions::phone::create_phone_interactive(&[])])
        } else { None };

    let socials: Option<Vec<Social>> = if bypass {
        None
    } else if Confirm::new()
        .with_prompt("Do you want to enter social media information?")
        .default(false)
        .interact()? {
            Some(vec![interactions::social::create_social_interactive()])
        } else { None };

    Ok(Contact {
        identifier: Uuid::new_v4(),
        identity: Identity {
            title,
            first_name: Some(first_name_val.clone()),
            middle_name: middle_name.clone(),
            last_name: Some(last_name_val.clone()),
            post_nominal,
            gender: Some(gender_val),
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
    })
}
