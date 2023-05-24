use chrono::{DateTime, NaiveDateTime, Utc};

pub mod io;

#[macro_export]
macro_rules! validate {
    ($val:expr) => {
        if $val {
            Some(())
        } else {
            None
        }
    };
}

pub fn parse_datetime(input: &str) -> Option<DateTime<Utc>> {
    if input.is_empty() || input == "0000:00:00 00:00:00" || input == ":  :     :  :  " {
        return None;
    }
    let popular_fmts = [
        "%Y:%m:%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%z",
        "%a %b %d %H:%M:%S %Y",
    ];
    for fmt in popular_fmts {
        match NaiveDateTime::parse_from_str(&input, &fmt) {
            Ok(dt) => return Some(dt.and_local_timezone(Utc).unwrap()),
            Err(_) => (),
        };
    }
    println!("Could not parse datetime ({})", input);
    return None;
}
