use dialoguer::Confirm;

pub fn add_death_to_contact(contact: &mut crate::contact::Contact) {
    // Mark as deceased
    contact.identity.is_alive = false;

    if Confirm::new()
        .with_prompt("Do you want to add a death date?")
        .default(true)
        .interact()
        .unwrap()
    {
        contact.identity.death_date = Some(crate::interactions::date::create_date_interactive());
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
