use dialoguer::{Input, Confirm};
use crate::models::Address;

pub fn create_address_interactive(existing_labels: &[String]) -> Address {
    let label = if Confirm::new()
        .with_prompt("Do you want to enter a label for this address?")
        .default(false)
        .interact()
        .unwrap()
    {
        loop {
            let input: String = Input::new()
                .with_prompt("Address Label")
                .interact_text()
                .unwrap();
            
            if existing_labels.contains(&input) {
                println!("Error: Label '{}' is already used for this contact. Please choose a different label.", input);
            } else {
                break Some(input);
            }
        }
    } else {
        if existing_labels.contains(&"default".to_string()) {
            println!("Error: The 'default' label is already used for this contact.");
            loop {
                let input: String = Input::new()
                    .with_prompt("Default label is taken. Please enter a different Address Label")
                    .interact_text()
                    .unwrap();
                
                if existing_labels.contains(&input) {
                    println!("Error: Label '{}' is already used. Please choose a different label.", input);
                } else {
                    break Some(input);
                }
            }
        } else {
            Some("default".to_string()) // Replace null with "default"
        }
    };

    let number = if Confirm::new()
        .with_prompt("Do you want to enter a house/building number?")
        .default(false)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Number")
                .interact_text()
                .unwrap(),
        )
    } else {
        None
    };

    let street = if Confirm::new()
        .with_prompt("Do you want to enter a street name?")
        .default(false)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Street")
                .interact_text()
                .unwrap(),
        )
    } else {
        None
    };

    let city = if Confirm::new()
        .with_prompt("Do you want to enter a city?")
        .default(true)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("City")
                .interact_text()
                .unwrap(),
        )
    } else {
        None
    };

    let post_code = if Confirm::new()
        .with_prompt("Do you want to enter a postal code?")
        .default(false)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Postal Code")
                .interact_text()
                .unwrap(),
        )
    } else {
        None
    };

    let region = if Confirm::new()
        .with_prompt("Do you want to enter a region/state?")
        .default(false)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Region/State")
                .interact_text()
                .unwrap(),
        )
    } else {
        None
    };

    let country = if Confirm::new()
        .with_prompt("Do you want to enter a country?")
        .default(true)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Country")
                .interact_text()
                .unwrap(),
        )
    } else {
        None
    };

    Address {
        label,
        country,
        region,
        city,
        post_code,
        street,
        number,
    }
}

pub fn create_address(
    label: Option<String>,
    country: Option<String>,
    region: Option<String>,
    city: Option<String>,
    post_code: Option<String>,
    street: Option<String>,
    number: Option<String>,
    existing_labels: &[String],
) -> Address {
    if label.is_some() || country.is_some() || region.is_some() || city.is_some() || post_code.is_some() || street.is_some() || number.is_some() {
        Address {
            label,
            country,
            region,
            city,
            post_code,
            street,
            number,
        }
    } else {
        create_address_interactive(existing_labels)
    }
}

pub fn add_address_to_contact(
    contact: &mut crate::contact::Contact,
    label: Option<String>,
    country: Option<String>,
    region: Option<String>,
    city: Option<String>,
    post_code: Option<String>,
    street: Option<String>,
    number: Option<String>,
) -> bool {
    let existing_labels: Vec<String> = contact
        .address
        .as_ref()
        .and_then(|a| a.label.clone())
        .map(|l| vec![l])
        .unwrap_or_default();

    let label_str = label.clone().unwrap_or_else(|| "default".to_string());
    if existing_labels.contains(&label_str) {
        println!("Error: Address label '{}' is already used for this contact.", label_str);
        return false;
    }

    let new_address = create_address(
        label,
        country,
        region,
        city,
        post_code,
        street,
        number,
        &existing_labels,
    );
    contact.address = Some(new_address);
    true
}
