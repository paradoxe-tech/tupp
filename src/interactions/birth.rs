use dialoguer::Confirm;

pub fn add_birth_to_contact(contact: &mut crate::contact::Contact) {
    if Confirm::new()
        .with_prompt("Do you want to add a birth date?")
        .default(true)
        .interact()
        .unwrap()
    {
        contact.identity.birth_date = Some(crate::interactions::date::create_date_interactive());
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
