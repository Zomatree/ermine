use jiff::{Span, Unit};


pub fn format_duration(duration: Span) -> String {
    let hours = duration.total(Unit::Hour).unwrap_or(0.).floor() as u64;

    if hours > 0 {
        if hours == 1 {
            "an hour".to_string()
        } else {
            format!("{hours} hours")
        }
    } else {
        let mins = duration.total(Unit::Minute).unwrap_or(0.).round() as u64;

        if mins > 0 {
            if mins == 1 {
                "a minute".to_string()
            } else {
                format!("{mins} minutes")
            }
        } else {
            "a few seconds".to_string()
        }
    }
}