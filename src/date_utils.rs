use chrono::*;
use chrono::{DateTime, Datelike, Utc};
use regex::Regex;

pub fn date_display(date: &DateTime<Utc>) -> DateTime<Local> {
    let converted: DateTime<Local> = date.with_timezone(&chrono::Local);
    return converted;
}

pub fn is_8601(line: &str) -> bool {
    return line.chars().nth(4) == Some('-')
        && line.chars().nth(7) == Some('-')
        && line.chars().nth(10) == Some('T')
        && line.chars().nth(13) == Some(':')
        && line.chars().nth(16) == Some(':');
}

pub fn is_8601_spaced(line: &str) -> bool {
    return line.chars().nth(4) == Some('-')
        && line.chars().nth(7) == Some('-')
        && line.chars().nth(10) == Some(' ')
        && line.chars().nth(13) == Some(':')
        && line.chars().nth(16) == Some(':');
}

pub fn is_8601_spaced_tz(line: &str) -> bool {
    return is_8601_spaced(line)
        && (line.chars().nth(19) == Some('+') || line.chars().nth(19) == Some('-'));
}

fn is_syslog_date_format(line: &str) -> bool {
    if line.len() < 16 {
        return false;
    }

    let input = &line[0..15];
    let re = Regex::new(r"^[Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec]").unwrap();
    re.is_match(input)
}

fn parse_datetime(datetime_str: &str) -> Option<DateTime<FixedOffset>> {
    let fmt = "%Y-%m-%d %H:%M:%S%z";

    match DateTime::parse_from_str(datetime_str, fmt) {
        Ok(dt) => Some(dt),
        Err(_) => {
            return None;
        }
    }
}

fn parse_8601_spaced_tz(line: &str) -> Option<DateTime<FixedOffset>> {
    let date_string = &line[0..22];
    let full_date_string = format!("{}00", date_string);
    let time = parse_datetime(&full_date_string);

    match time {
        Some(time) => return Some(time),
        None => return None,
    }
}

pub fn date_from_line(line: &str) -> Option<DateTime<Utc>> {
    if is_8601(line) {
        let date_string = &line[0..19];
        let time = Utc.datetime_from_str(&date_string, "%Y-%m-%dT%H:%M:%S");
        match time {
            Ok(time) => return Some(time),
            Err(_) => return None,
        };
    }

    if is_8601_spaced_tz(line) {
        let date = parse_8601_spaced_tz(line);
        match date {
            Some(date) => return Some(date.with_timezone(&Utc)),
            None => return None,
        };
    }

    if is_syslog_date_format(line) {
        let date_string = &line[0..15];
        let year = Utc::now().year();
        let year_string = year.to_string();
        let date_string_with_year = format!("{} {}", year_string, date_string);
        let time = Utc.datetime_from_str(&date_string_with_year, "%Y %b %d %H:%M:%S");
        match time {
            Ok(time) => return Some(time),
            Err(_) => return None,
        };
    }

    return None;
}

pub fn round_to_nearest_seconds(time: DateTime<Utc>, seconds: i64) -> DateTime<Utc> {
    let total_seconds =
        time.hour() as i64 * 3600 + time.minute() as i64 * 60 + time.second() as i64;
    let remainder = total_seconds % seconds;
    let round_down_seconds = total_seconds - remainder;
    let round_up_seconds = if remainder > seconds / 2 {
        round_down_seconds + seconds
    } else {
        round_down_seconds
    };

    time.date().and_hms(0, 0, 0) + Duration::seconds(round_up_seconds)
}

#[cfg(test)]
mod tests {
    #[test]
    fn _test_is_8601() {
        assert_eq!(super::is_8601("2021-01-01T00:00:00"), true);
        assert_eq!(super::is_8601(" 2021-01-01T00:00:00"), false);
        assert_eq!(super::is_8601("2021:01:01T00-00-00"), false);
    }

    #[test]
    fn _test_is_8601_spaced() {
        assert_eq!(super::is_8601_spaced("2021-01-01 00:00:00"), true);
        assert_eq!(super::is_8601_spaced(" 2021-01-01 00:00:00"), false);
        assert_eq!(super::is_8601_spaced("2021:01:01 00-00-00"), false);
    }

    #[test]
    fn _test_is_8601_spaced_tz() {
        assert_eq!(super::is_8601_spaced_tz("2021-01-01 00:00:00+00:00"), true);
        assert_eq!(super::is_8601_spaced_tz(" 2021-01-01 00:00:00"), false);
        assert_eq!(super::is_8601_spaced_tz("2021:01:01 00-00-00"), false);

        assert_eq!(super::is_8601_spaced_tz("2024-04-22 11:02:53+01"), true);
        assert_eq!(super::is_8601_spaced_tz("2024-04-22 11:02:53+01"), true);
        assert_eq!(super::is_8601_spaced_tz("2023-01-23 08:30:36-08"), true);
    }

    #[test]
    fn test_parse_8601_spaced_tz() {
        assert_eq!(
            super::date_from_line("2024-04-22 11:02:53+01").unwrap(),
            chrono::DateTime::parse_from_rfc3339("2024-04-22T10:02:53+00:00").unwrap()
        );

        assert_eq!(
            super::date_from_line("2023-01-23 08:30:36-08").unwrap(),
            chrono::DateTime::parse_from_rfc3339("2023-01-23T16:30:36+00:00").unwrap()
        );
    }
}
