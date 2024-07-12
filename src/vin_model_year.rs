use chrono::{Datelike, Utc};
use regex::bytes::Regex;

pub fn vin_model_year(vin: &str) -> Option<i32> {
    let vin = vin.to_uppercase();
    if vin.len() >= 10 {
        let pos_ten = &vin[9..10];
        let mut model_year = match pos_ten {
            my if matches_regex(my, "[A-H]")? => offset_from(my, 'A').map(|o| 2010 + o),
            my if matches_regex(my, "[J-N]")? => offset_from(my, 'A').map(|o| 2009 + o),
            my if matches_regex(my, "[R-T]")? => offset_from(my, 'A').map(|o| 2007 + o),
            my if matches_regex(my, "[V-Y]")? => offset_from(my, 'A').map(|o| 2006 + o),
            my if matches_regex(my, "[1-9]")? => offset_from(my, '1').map(|o| 2031 + o),
            my if matches_regex(my, "P")? => Some(2023),
            _ => None,
        }?;

        if model_year > Utc::now().year() + 1 {
            model_year -= 30;
        }

        Some(model_year)
    } else {
        None
    }
}

fn offset_from(my: &str, val: char) -> Option<i32> {
    let val: i32 = u32::from(val).try_into().ok()?;
    let my: i32 = (*my.as_bytes().first()?).into();
    Some(my - val)
}

fn matches_regex<'a>(my: &'a str, patt: &'a str) -> Option<bool> {
    Some(Regex::new(patt).ok()?.find(my.as_bytes()).is_some())
}
