use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::constants::{Dates, LOCAL_TYPE};

#[derive(Serialize, Deserialize)]
struct CaGCalendarsFormat {
    format: HashMap<String, HashMap<String, String>>,
}
#[derive(Serialize, Deserialize)]
struct CaGCalendarsII {
    months: CaGCalendarsFormat,
    days: CaGCalendarsFormat,
}
#[derive(Serialize, Deserialize)]
struct CaGCalendarsI {
    gregorian: CaGCalendarsII,
}

#[derive(Serialize, Deserialize)]
struct CaGCalendars {
    calendars: CaGCalendarsI,
}

#[derive(Serialize, Deserialize)]
struct CaGId {
    identity: Value,
    dates: CaGCalendars,
}

#[derive(Serialize, Deserialize)]
struct CaGregorian {
    main: HashMap<String, CaGId>,
}

pub fn get_dates_formatting(cldr_dir: &str, locale_id: &str) -> Result<Dates, &'static str> {
    let calendar_file = format!(
        "{}cldr-json/cldr-dates-{}/main/{}/ca-gregorian.json",
        cldr_dir, LOCAL_TYPE, locale_id
    );

    let contents =
        fs::read_to_string(calendar_file).or(Err("Failed reading 'ca-gregorian' file"))?;
    let ca_gregorian: CaGregorian =
        serde_json::from_str(&contents).or(Err("Failed parsing 'ca-gregorian' file"))?;
    let gregorian = &ca_gregorian.main[locale_id].dates.calendars.gregorian;
    // See: http://cldr.unicode.org/translation/date-time-1/date-time-patterns
    // for the difference between stand-alone and format. We will use only the format mode
    let months_format = &gregorian.months.format;
    let days_format = &gregorian.days.format;
    let mut day_names = vec![];
    let mut day_names_short = vec![];

    let mut months = vec![];
    let mut months_short = vec![];
    let mut months_letter = vec![];

    let month_index = vec![
        "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
    ];
    for index in month_index {
        months_letter.push(months_format["narrow"][index].to_owned());
        months_short.push(months_format["abbreviated"][index].to_owned());
        months.push(months_format["wide"][index].to_owned());
    }

    let day_index = vec!["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
    for day in day_index {
        day_names_short.push(days_format["abbreviated"][day].to_owned());
        day_names.push(days_format["wide"][day].to_owned());
    }

    Ok(Dates {
        day_names,
        day_names_short,
        months,
        months_short,
        months_letter,
    })
}
