use dialoguer::{Input, Confirm};
use crate::models::Address;

pub fn create_address_interactive() -> Address {
    let label = if Confirm::new()
        .with_prompt("Do you want to enter a label for this address?")
        .default(false)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Address Label")
                .interact_text()
                .unwrap(),
        )
    } else {
        Some("default".to_string()) // Replace null with "default"
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
        create_address_interactive()
    }
}
