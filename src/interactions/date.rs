use dialoguer::{Input, Confirm};
use crate::models::Date;

pub fn create_date_interactive() -> Date {
    let year: Option<i32> = if Confirm::new()
        .with_prompt("Do you want to add a year?")
        .default(true)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Year")
                .interact_text()
                .unwrap()
        )
    } else {
        None
    };

    let month: Option<u8> = if Confirm::new()
        .with_prompt("Do you want to add a month?")
        .default(true)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Month (1-12)")
                .interact_text()
                .unwrap()
        )
    } else {
        None
    };

    let day: Option<u8> = if Confirm::new()
        .with_prompt("Do you want to add a day?")
        .default(true)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Day (1-31)")
                .interact_text()
                .unwrap()
        )
    } else {
        None
    };

    let hour: Option<u8> = if Confirm::new()
        .with_prompt("Do you want to add an hour?")
        .default(false)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Hour (0-23)")
                .interact_text()
                .unwrap()
        )
    } else {
        None
    };

    let minute: Option<u8> = if Confirm::new()
        .with_prompt("Do you want to add minutes?")
        .default(false)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Minutes (0-59)")
                .interact_text()
                .unwrap()
        )
    } else {
        None
    };

    let second: Option<u8> = if Confirm::new()
        .with_prompt("Do you want to add seconds?")
        .default(false)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Seconds (0-59)")
                .interact_text()
                .unwrap()
        )
    } else {
        None
    };

    Date {
        year,
        month,
        day,
        hour,
        minute,
        second,
    }
}
