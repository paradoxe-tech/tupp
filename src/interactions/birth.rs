use dialoguer::{Input, Confirm};
use crate::models::{Date, Address};

pub fn create_birth_date_interactive() -> Date {
    let year: i32 = Input::new()
        .with_prompt("Birth Year")
        .interact_text()
        .unwrap();

    let month: u8 = Input::new()
        .with_prompt("Birth Month (1-12)")
        .interact_text()
        .unwrap();

    let day: u8 = Input::new()
        .with_prompt("Birth Day (1-31)")
        .interact_text()
        .unwrap();

    Date {
        year: Some(year),
        month: Some(month),
        day: Some(day),
        hour: None,
        minute: None,
        second: None,
    }
}

pub fn add_birth_to_contact(contact: &mut crate::contact::Contact) {
    if Confirm::new()
        .with_prompt("Do you want to add a birth date?")
        .default(true)
        .interact()
        .unwrap()
    {
        contact.identity.birth_date = Some(create_birth_date_interactive());
    }

    if Confirm::new()
        .with_prompt("Do you want to add a birth location?")
        .default(false)
        .interact()
        .unwrap()
    {
        contact.identity.birth_location = Some(crate::interactions::address::create_address_interactive());
    }
}
