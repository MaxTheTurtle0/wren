//! Timestamps: Graph's ISO 8601 strings in, short local labels out.

use jiff::{Timestamp, Zoned, civil, tz::TimeZone};

use crate::model::Millis;

pub fn now() -> Millis {
    Timestamp::now().as_millisecond()
}

/// Parses `2024-02-01T10:00:00.123Z` (with any offset or fraction).
pub fn parse_iso(value: &str) -> Option<Millis> {
    value
        .parse::<Timestamp>()
        .ok()
        .map(Timestamp::as_millisecond)
}

/// Parses a calendar `dateTime` without an offset (Graph's
/// `dateTimeTimeZone`), in the zone it names. Wren asks for UTC.
pub fn parse_civil(value: &str, zone: Option<&str>) -> Option<Millis> {
    // Graph writes seven fractional digits; jiff takes up to nine.
    let civil: civil::DateTime = value.parse().ok()?;
    let tz = match zone {
        None | Some("UTC") | Some("") => TimeZone::UTC,
        Some(name) => TimeZone::get(name).unwrap_or(TimeZone::UTC),
    };
    civil
        .to_zoned(tz)
        .ok()
        .map(|zoned| zoned.timestamp().as_millisecond())
}

pub fn to_iso(millis: Millis) -> String {
    Timestamp::from_millisecond(millis)
        .map(|t| t.strftime("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
        .unwrap_or_default()
}

fn zoned(millis: Millis, tz: &TimeZone) -> Option<Zoned> {
    Timestamp::from_millisecond(millis)
        .ok()
        .map(|t| t.to_zoned(tz.clone()))
}

thread_local! {
    static LOCAL: TimeZone = TimeZone::system();
}

fn local(millis: Millis) -> Option<Zoned> {
    LOCAL.with(|tz| zoned(millis, tz))
}

/// `14:05`.
pub fn clock(millis: Millis) -> String {
    local(millis)
        .map(|z| z.strftime("%H:%M").to_string())
        .unwrap_or_default()
}

/// The chat list's label: the time today, `Yesterday`, the weekday within a
/// week, else the date.
pub fn short_label(millis: Millis, now: Millis) -> String {
    LOCAL.with(|tz| short_label_in(millis, now, tz))
}

pub fn short_label_in(millis: Millis, now: Millis, tz: &TimeZone) -> String {
    let (Some(at), Some(today)) = (zoned(millis, tz), zoned(now, tz)) else {
        return String::new();
    };
    let days = (today.date() - at.date()).get_days();
    if days <= 0 {
        at.strftime("%H:%M").to_string()
    } else if days == 1 {
        "Yesterday".into()
    } else if days < 7 {
        at.strftime("%a").to_string()
    } else if at.year() == today.year() {
        at.strftime("%-d %b").to_string()
    } else {
        at.strftime("%-d %b %Y").to_string()
    }
}

/// The divider between days in a conversation: `Today`, `Yesterday`,
/// `Monday 6 October`.
pub fn day_label(millis: Millis, now: Millis) -> String {
    LOCAL.with(|tz| {
        let (Some(at), Some(today)) = (zoned(millis, tz), zoned(now, tz)) else {
            return String::new();
        };
        match (today.date() - at.date()).get_days() {
            0 => "Today".into(),
            1 => "Yesterday".into(),
            _ if at.year() == today.year() => at.strftime("%A %-d %B").to_string(),
            _ => at.strftime("%A %-d %B %Y").to_string(),
        }
    })
}

/// A key that changes when the local day changes.
pub fn day_key(millis: Millis) -> i64 {
    local(millis)
        .map(|z| i64::from(z.year()) * 400 + i64::from(z.day_of_year()))
        .unwrap_or_default()
}

/// `9:00 – 9:30`, or `All day`.
pub fn span_label(start: Millis, end: Millis, all_day: bool) -> String {
    if all_day {
        return "All day".into();
    }
    format!("{} – {}", clock(start), clock(end))
}

/// The local midnight starting the day `millis` falls in.
pub fn start_of_day(millis: Millis) -> Millis {
    local(millis)
        .and_then(|z| z.start_of_day().ok())
        .map_or(millis, |z| z.timestamp().as_millisecond())
}

/// How long until `millis`, in words: `in 5 min`, `now`, `in 2 h`.
pub fn until_label(millis: Millis, now: Millis) -> String {
    let minutes = (millis - now) / 60_000;
    match minutes {
        m if m <= 0 => "now".into(),
        m if m < 60 => format!("in {m} min"),
        m if m < 24 * 60 => format!("in {} h", m / 60),
        m => format!("in {} d", m / (24 * 60)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_round_trip() {
        let at = parse_iso("2024-02-01T10:00:00.123Z").unwrap();
        assert_eq!(to_iso(at), "2024-02-01T10:00:00.123Z");
        assert_eq!(
            parse_iso("2024-02-01T11:00:00+01:00"),
            parse_iso("2024-02-01T10:00:00Z")
        );
        assert_eq!(parse_iso("nonsense"), None);
    }

    #[test]
    fn civil_times_with_seven_fraction_digits() {
        let at = parse_civil("2026-10-11T09:00:00.0000000", Some("UTC")).unwrap();
        assert_eq!(to_iso(at), "2026-10-11T09:00:00.000Z");
    }

    #[test]
    fn labels_by_distance() {
        let tz = TimeZone::UTC;
        let now = parse_iso("2026-10-11T15:00:00Z").unwrap();
        assert_eq!(
            short_label_in(parse_iso("2026-10-11T09:05:00Z").unwrap(), now, &tz),
            "09:05"
        );
        assert_eq!(
            short_label_in(parse_iso("2026-10-10T09:05:00Z").unwrap(), now, &tz),
            "Yesterday"
        );
        assert_eq!(
            short_label_in(parse_iso("2026-10-07T09:05:00Z").unwrap(), now, &tz),
            "Wed"
        );
        assert_eq!(
            short_label_in(parse_iso("2026-03-02T09:05:00Z").unwrap(), now, &tz),
            "2 Mar"
        );
        assert_eq!(
            short_label_in(parse_iso("2025-03-02T09:05:00Z").unwrap(), now, &tz),
            "2 Mar 2025"
        );
    }

    #[test]
    fn until() {
        assert_eq!(until_label(10 * 60_000, 0), "in 10 min");
        assert_eq!(until_label(0, 10), "now");
        assert_eq!(until_label(3 * 3_600_000, 0), "in 3 h");
    }
}
