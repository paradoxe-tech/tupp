use dialoguer::{Input, Confirm};
use crate::models::{Date, Address};

pub fn create_death_date_interactive() -> Date {
    let year: i32 = Input::new()
        .with_prompt("Death Year")
        .interact_text()
        .unwrap();

    let month: u8 = Input::new()
        .with_prompt("Death Month (1-12)")
        .interact_text()
        .unwrap();

    let day: u8 = Input::new()
        .with_prompt("Death Day (1-31)")
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

pub fn add_death_to_contact(contact: &mut crate::contact::Contact) {
    // Mark as deceased
    contact.identity.is_alive = false;

    if Confirm::new()
        .with_prompt("Do you want to add a death date?")
        .default(true)
        .interact()
        .unwrap()
    {
        contact.identity.death_date = Some(create_death_date_interactive());
    }

    if Confirm::new()
        .with_prompt("Do you want to add a death location?")
        .default(false)
        .interact()
        .unwrap()
    {
        contact.identity.death_location = Some(crate::interactions::address::create_address_interactive());
    }
}
