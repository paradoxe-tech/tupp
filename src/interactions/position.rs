use uuid::Uuid;
use crate::contact::{Contact, Position, Tenure};
use crate::models::Date;

pub fn add_position_to_contact(
    contact: &mut Contact,
    institution: Uuid,
    title: String,
    start_year: Option<i32>,
    start_month: Option<u8>,
    start_day: Option<u8>,
    end_year: Option<i32>,
    end_month: Option<u8>,
    end_day: Option<u8>,
) {
    let start = if start_year.is_some() || start_month.is_some() || start_day.is_some() {
        Some(Date { year: start_year, month: start_month, day: start_day, hour: None, minute: None, second: None })
    } else {
        None
    };

    let tenure = if end_year.is_some() || end_month.is_some() || end_day.is_some() {
        Tenure::Ended(Date { year: end_year, month: end_month, day: end_day, hour: None, minute: None, second: None })
    } else {
        Tenure::Present
    };

    let new_position = Position { institution, title, start, tenure };

    if let Some(ref mut positions) = contact.positions {
        positions.push(new_position);
    } else {
        contact.positions = Some(vec![new_position]);
    }
}
