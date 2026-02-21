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

pub fn create_date(
    year: Option<i32>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>
) -> Date {
    if year.is_some() || month.is_some() || day.is_some() || hour.is_some() || minute.is_some() || second.is_some() {
        Date {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    } else {
        create_date_interactive()
    }
}
