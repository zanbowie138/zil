//! Date helpers: parsing (fixed formats and natural language), calendar math, relative text, month grids.
//! Weeks start on Monday (ISO 8601).

use crate::units::{self, Unit};
use jiff::civil::{self, Date, Time, Weekday};
use jiff::{SignedDuration, Span, Timestamp, Zoned, tz::TimeZone};

type R<T> = Result<T, String>;

fn e(x: impl ToString) -> String {
    x.to_string()
}

pub const PERIODS: &str = "second, minute, hour, day, week, month, quarter, year";

/// `z + v u`; whole months/years follow the calendar (Jan 31 + 1 mo = Feb 28), not a fixed length.
pub fn add(z: &Zoned, v: f64, u: &Unit) -> R<Zoned> {
    if u.dim() != units::unit("s")?.dim() {
        return Err(format!("cannot add {u} to a date"));
    }
    let r = if (u.is("mo") || u.is("yr")) && v.fract() == 0.0 {
        let span = if u.is("mo") { Span::new().try_months(v as i64) } else { Span::new().try_years(v as i64) };
        z.checked_add(span.map_err(e)?)
    } else {
        z.checked_add(SignedDuration::try_from_secs_f64(u.to_si(v)).map_err(e)?)
    };
    r.map_err(e)
}

/// Any supported date string: ISO 8601 / RFC 9557 / RFC 2822, US and written-out dates, or natural language.
pub fn parse(s: &str, now: &Zoned) -> Option<Zoned> {
    strict(s.trim(), now.time_zone()).or_else(|| natural(s, now))
}

const DATE_FORMATS: &[&str] = &[
    "%m/%d/%y", "%d/%m/%y", "%m/%d/%Y", "%d/%m/%Y", // US first; day/month only when month would be > 12
    "%b %d %Y", "%b %d, %Y", "%d %b %Y", "%d %b, %Y", "%a %b %d %Y", "%a, %b %d, %Y", // abbreviated month
    "%B %d %Y", "%B %d, %Y", "%d %B %Y", "%d %B, %Y", "%A %B %d %Y", "%A, %B %d, %Y", // full month
];
const TIME_FORMATS: &[&str] = &["", " %H:%M", " %H:%M:%S", " %I:%M %p", " %I:%M%p", " %I%p", " %I %p"];

fn strict(s: &str, tz: &TimeZone) -> Option<Zoned> {
    if let Ok(z) = s.parse::<Zoned>() {
        return Some(z);
    }
    if let Ok(t) = s.parse::<Timestamp>() {
        return Some(t.to_zoned(tz.clone()));
    }
    if let Ok(dt) = s.replacen(' ', "T", 1).parse::<civil::DateTime>() {
        return dt.to_zoned(tz.clone()).ok();
    }
    if let Ok(d) = s.parse::<Date>() {
        return d.to_zoned(tz.clone()).ok();
    }
    if let Ok(z) = jiff::fmt::rfc2822::parse(s) {
        return Some(z);
    }
    DATE_FORMATS
        .iter()
        .flat_map(|d| TIME_FORMATS.iter().map(move |t| format!("{d}{t}")))
        .find_map(|f| with_format(s, &f, tz))
}

/// Parse with explicit strftime codes; missing time means midnight, missing zone means `tz`.
pub fn with_format(s: &str, fmt: &str, tz: &TimeZone) -> Option<Zoned> {
    let tm = jiff::fmt::strtime::parse(fmt, s).ok()?;
    if let Ok(z) = tm.to_zoned() {
        return Some(z);
    }
    if let Ok(dt) = tm.to_datetime() {
        return dt.to_zoned(tz.clone()).ok();
    }
    tm.to_date().ok()?.to_zoned(tz.clone()).ok()
}

/// "tomorrow at 5pm", "next friday", "3 days ago", "in 2 weeks", "end of month", "first day of next year".
fn natural(s: &str, now: &Zoned) -> Option<Zoned> {
    let s = s.trim().to_lowercase().replace(" am", "am").replace(" pm", "pm");
    let (day, time) = split_time(&s);
    let w: Vec<&str> = day.split_whitespace().filter(|w| *w != "the").collect();
    let tz = now.time_zone();
    let today = now.date();
    let at = |d: Date| d.to_zoned(tz.clone()).ok();
    let z = match w[..] {
        [] if time.is_some() => at(today)?,
        ["now"] => now.clone(),
        ["today"] => at(today)?,
        ["tomorrow"] => at(today.tomorrow().ok()?)?,
        ["yesterday"] => at(today.yesterday().ok()?)?,
        ["in", n, u] | [n, u, "from", "now"] => add(now, count(n)?, &time_unit(u)?).ok()?,
        [n, u, "ago"] => add(now, -count(n)?, &time_unit(u)?).ok()?,
        ["next" | "last", wd] if weekday(wd).is_some() => {
            at(today.nth_weekday(if w[0] == "next" { 1 } else { -1 }, weekday(wd)?).ok()?)?
        }
        ["this", wd] | [wd] if weekday(wd).is_some() => {
            let wd = weekday(wd)?;
            at(if today.weekday() == wd { today } else { today.nth_weekday(1, wd).ok()? })?
        }
        ["next" | "last" | "this", p] => at(shift(now, p, rel(w[0]))?.date())?,
        ["start" | "beginning" | "end", "of", ref rest @ ..] => {
            let (base, p) = period_base(rest, now)?;
            if w[0] == "end" { end_of(&base, p).ok()? } else { start_of(&base, p).ok()? }
        }
        ["first" | "last", "day", "of", ref rest @ ..] => {
            let (base, p) = period_base(rest, now)?;
            let z = if w[0] == "first" { start_of(&base, p) } else { end_of(&base, p) };
            z.ok()?.start_of_day().ok()?
        }
        _ => strict(day, tz)?,
    };
    match time {
        Some(t) => z.with().time(t).build().ok(),
        None => Some(z),
    }
}

/// `["month"]`, `["next", "month"]` → the moment to take the period from, and the period.
fn period_base<'a>(rest: &[&'a str], now: &Zoned) -> Option<(Zoned, &'a str)> {
    match *rest {
        [p] => Some((now.clone(), p)),
        [r @ ("next" | "last" | "this"), p] => Some((shift(now, p, rel(r))?, p)),
        _ => None,
    }
}

fn rel(word: &str) -> i64 {
    match word {
        "next" => 1,
        "last" => -1,
        _ => 0,
    }
}

fn shift(z: &Zoned, period: &str, n: i64) -> Option<Zoned> {
    add_period(z, period, n).ok()
}

fn count(n: &str) -> Option<f64> {
    match n {
        "a" | "an" | "one" => Some(1.0),
        _ => n.parse().ok(),
    }
}

fn time_unit(u: &str) -> Option<Unit> {
    let u = units::unit(u).ok()?;
    (u.dim() == units::unit("s").ok()?.dim()).then_some(u)
}

/// Split a trailing time off: "tomorrow at 5pm", "dec 25 17:30", "noon".
fn split_time(s: &str) -> (&str, Option<Time>) {
    if let Some((d, t)) = s.rsplit_once(" at ")
        && let Some(t) = time(t)
    {
        return (d, Some(t));
    }
    if let Some(t) = time(s) {
        return ("", Some(t));
    }
    match s.rsplit_once(' ').and_then(|(d, t)| Some((d, time(t)?))) {
        Some((d, t)) => (d, Some(t)),
        None => (s, None),
    }
}

/// "5pm", "5:30am", "17:30", "17:30:15", "noon", "midnight". A bare number is not a time.
fn time(t: &str) -> Option<Time> {
    match t {
        "noon" => return Some(Time::constant(12, 0, 0, 0)),
        "midnight" => return Some(Time::midnight()),
        _ => {}
    }
    let (t, half) = match (t.strip_suffix("pm"), t.strip_suffix("am")) {
        (Some(r), _) => (r, Some(12)),
        (_, Some(r)) => (r, Some(0)),
        _ => (t, None),
    };
    if half.is_none() && !t.contains(':') {
        return None;
    }
    let mut p = t.split(':').map(|x| x.parse::<i8>().ok());
    let h = p.next()??;
    let m = p.next().unwrap_or(Some(0))?;
    let s = p.next().unwrap_or(Some(0))?;
    if p.next().is_some() {
        return None;
    }
    let h = match half {
        Some(off) if (1..=12).contains(&h) => h % 12 + off,
        Some(_) => return None,
        None => h,
    };
    Time::new(h, m, s, 0).ok()
}

/// "monday", "Mon", "thurs" (any prefix of 3+ letters).
pub fn weekday(w: &str) -> Option<Weekday> {
    const NAMES: [&str; 7] = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"];
    let w = w.to_lowercase();
    let i = NAMES.iter().position(|n| w.len() >= 3 && n.starts_with(&w))?;
    Weekday::from_monday_one_offset(i as i8 + 1).ok()
}

pub fn start_of(z: &Zoned, period: &str) -> R<Zoned> {
    let d = z.date();
    let day = match period {
        "second" => return z.with().subsec_nanosecond(0).build().map_err(e),
        "minute" => return z.with().second(0).subsec_nanosecond(0).build().map_err(e),
        "hour" => return z.with().minute(0).second(0).subsec_nanosecond(0).build().map_err(e),
        "day" => d,
        "week" => d.checked_sub(Span::new().days(d.weekday().to_monday_zero_offset())).map_err(e)?,
        "month" => d.first_of_month(),
        "quarter" => Date::new(d.year(), (d.month() - 1) / 3 * 3 + 1, 1).map_err(e)?,
        "year" => d.first_of_year(),
        _ => return Err(format!("unknown period {period:?} ({PERIODS})")),
    };
    day.to_zoned(z.time_zone().clone()).map_err(e)
}

/// The last nanosecond of the period.
pub fn end_of(z: &Zoned, period: &str) -> R<Zoned> {
    let next = add_period(&start_of(z, period)?, period, 1)?;
    next.checked_sub(SignedDuration::from_nanos(1)).map_err(e)
}

fn add_period(z: &Zoned, period: &str, n: i64) -> R<Zoned> {
    let span = match period {
        "second" => Span::new().try_seconds(n),
        "minute" => Span::new().try_minutes(n),
        "hour" => Span::new().try_hours(n),
        "day" => Span::new().try_days(n),
        "week" => Span::new().try_weeks(n),
        "month" => Span::new().try_months(n),
        "quarter" => Span::new().try_months(3 * n),
        "year" => Span::new().try_years(n),
        _ => return Err(format!("unknown period {period:?} ({PERIODS})")),
    };
    z.checked_add(span.map_err(e)?).map_err(e)
}

/// Same date and time with some fields replaced: year, month, day, hour, minute, second.
pub fn with(z: &Zoned, fields: &[(String, i64)]) -> R<Zoned> {
    let mut w = z.with();
    for &(ref k, n) in fields {
        let small = |n: i64| i8::try_from(n).map_err(|_| format!("{k} {n} out of range"));
        w = match k.as_str() {
            "year" => w.year(i16::try_from(n).map_err(|_| format!("year {n} out of range"))?),
            "month" => w.month(small(n)?),
            "day" => w.day(small(n)?),
            "hour" => w.hour(small(n)?),
            "minute" => w.minute(small(n)?),
            "second" => w.second(small(n)?),
            _ => return Err(format!("unknown field {k:?} (year, month, day, hour, minute, second)")),
        };
    }
    w.build().map_err(e)
}

pub fn is_weekend(d: Date) -> bool {
    matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday)
}

// ponytail: business-day math walks day by day; fine for calendar-sized spans, closed form if ever needed for centuries.
/// Move `n` Monday-Friday days (negative goes back); keeps the time of day.
pub fn add_workdays(z: &Zoned, n: i64) -> R<Zoned> {
    let step = if n < 0 { -1 } else { 1 };
    let mut d = z.date();
    for _ in 0..n.abs() {
        d = d.checked_add(Span::new().days(step)).map_err(e)?;
        while is_weekend(d) {
            d = d.checked_add(Span::new().days(step)).map_err(e)?;
        }
    }
    z.with().date(d).build().map_err(e)
}

/// Monday-Friday days from `a` up to (not including) `b`; negative if `b` is earlier.
pub fn workdays(a: &Zoned, b: &Zoned) -> R<i64> {
    let (mut d, end, sign) = if a.date() <= b.date() { (a.date(), b.date(), 1) } else { (b.date(), a.date(), -1) };
    let mut n = 0;
    while d < end {
        n += i64::from(!is_weekend(d));
        d = d.tomorrow().map_err(e)?;
    }
    Ok(n * sign)
}

/// Whole years from `birth` to `now`.
pub fn age(birth: &Zoned, now: &Zoned) -> i64 {
    let (b, n) = (birth.date(), now.with_time_zone(birth.time_zone().clone()).date());
    let before_birthday = (n.month(), n.day()) < (b.month(), b.day());
    (n.year() - b.year()) as i64 - i64::from(before_birthday)
}

/// Calendar difference from `a` to `b`: "1 yr 2 mo 3 d 4 h".
pub fn diff(a: &Zoned, b: &Zoned) -> R<String> {
    let s = a.until((jiff::Unit::Year, b)).map_err(e)?;
    let fields = [
        (s.get_years() as i64, "yr"),
        (s.get_months() as i64, "mo"),
        (s.get_days() as i64, "d"),
        (s.get_hours() as i64, "h"),
        (s.get_minutes(), "min"),
        (s.get_seconds(), "s"),
    ];
    let parts: Vec<String> = fields.iter().filter(|f| f.0 != 0).map(|(n, u)| format!("{} {u}", n.abs())).collect();
    let sign = if s.is_negative() { "-" } else { "" };
    Ok(if parts.is_empty() { "0 s".into() } else { format!("{sign}{}", parts.join(" ")) })
}

/// "in 3 days", "2 hours ago", "just now".
pub fn relative(z: &Zoned, now: &Zoned) -> String {
    let secs = (z.timestamp().as_second() - now.timestamp().as_second()) as f64;
    let a = secs.abs();
    let (n, unit) = match a {
        _ if a < 45.0 => return "just now".into(),
        _ if a < 45.0 * 60.0 => ((a / 60.0).round(), "minute"),
        _ if a < 22.0 * 3600.0 => ((a / 3600.0).round(), "hour"),
        _ if a < 26.0 * 86400.0 => ((a / 86400.0).round(), "day"),
        _ if a < 320.0 * 86400.0 => ((a / 2629746.0).round().max(1.0), "month"),
        _ => ((a / 31556952.0).round().max(1.0), "year"),
    };
    let s = if n == 1.0 { "" } else { "s" };
    if secs > 0.0 { format!("in {n} {unit}{s}") } else { format!("{n} {unit}{s} ago") }
}

/// Month grid like `cal`, Monday first.
pub fn calendar(year: i64, month: i64) -> R<String> {
    let y = i16::try_from(year).map_err(|_| format!("year {year} out of range"))?;
    let m = i8::try_from(month).map_err(|_| format!("month {month} out of range"))?;
    let first = Date::new(y, m, 1).map_err(e)?;
    let mut cells = vec!["  ".to_string(); first.weekday().to_monday_zero_offset() as usize];
    cells.extend((1..=first.days_in_month()).map(|d| format!("{d:>2}")));
    let rows: Vec<String> = cells.chunks(7).map(|c| c.join(" ")).collect();
    let title = first.strftime("%B %Y").to_string();
    Ok(format!("{title:^20}\nMo Tu We Th Fr Sa Su\n{}", rows.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tuesday 2026-10-06 14:30 UTC.
    fn now() -> Zoned {
        "2026-10-06T14:30:00[UTC]".parse().unwrap()
    }

    fn p(s: &str) -> String {
        let z = parse(s, &now()).unwrap_or_else(|| panic!("cannot parse {s:?}"));
        z.strftime("%Y-%m-%d %H:%M").to_string()
    }

    #[test]
    fn formats() {
        assert_eq!(p("2026-12-25"), "2026-12-25 00:00");
        assert_eq!(p("2026-12-25 10:30"), "2026-12-25 10:30");
        assert_eq!(p("2026-12-25T10:00Z"), "2026-12-25 10:00");
        assert_eq!(p("12/25/2026"), "2026-12-25 00:00");
        assert_eq!(p("03/04/2026"), "2026-03-04 00:00");
        assert_eq!(p("25/12/2026"), "2026-12-25 00:00");
        assert_eq!(p("12/25/26"), "2026-12-25 00:00");
        assert_eq!(p("Dec 25 2026"), "2026-12-25 00:00");
        assert_eq!(p("December 25, 2026"), "2026-12-25 00:00");
        assert_eq!(p("25 Dec 2026 5pm"), "2026-12-25 17:00");
        assert_eq!(p("Fri, 25 Dec 2026 10:00:00 +0000"), "2026-12-25 10:00");
        assert!(parse("garbage", &now()).is_none());
        assert!(parse("", &now()).is_none());
    }

    #[test]
    fn natural_language() {
        assert_eq!(p("now"), "2026-10-06 14:30");
        assert_eq!(p("today"), "2026-10-06 00:00");
        assert_eq!(p("tomorrow at 5pm"), "2026-10-07 17:00");
        assert_eq!(p("yesterday noon"), "2026-10-05 12:00");
        assert_eq!(p("next friday"), "2026-10-09 00:00");
        assert_eq!(p("last friday"), "2026-10-02 00:00");
        assert_eq!(p("tuesday"), "2026-10-06 00:00");
        assert_eq!(p("next tue"), "2026-10-13 00:00");
        assert_eq!(p("3 days ago"), "2026-10-03 14:30");
        assert_eq!(p("in 2 weeks"), "2026-10-20 14:30");
        assert_eq!(p("an hour from now"), "2026-10-06 15:30");
        assert_eq!(p("in 1 month"), "2026-11-06 14:30");
        assert_eq!(p("next month"), "2026-11-06 00:00");
        assert_eq!(p("end of month"), "2026-10-31 23:59");
        assert_eq!(p("start of the week"), "2026-10-05 00:00");
        assert_eq!(p("first day of next month"), "2026-11-01 00:00");
        assert_eq!(p("last day of year"), "2026-12-31 00:00");
        assert_eq!(p("5:30 pm"), "2026-10-06 17:30");
        assert_eq!(p("dec 25 2026 at 9am"), "2026-12-25 09:00");
        assert!(parse("next blursday", &now()).is_none());
    }

    #[test]
    fn calendar_math() {
        let z = now();
        assert_eq!(start_of(&z, "quarter").unwrap().date().to_string(), "2026-10-01");
        assert_eq!(end_of(&z, "week").unwrap().strftime("%F %T").to_string(), "2026-10-11 23:59:59");
        assert_eq!(add_workdays(&z, 4).unwrap().date().to_string(), "2026-10-12");
        assert_eq!(add_workdays(&z, -2).unwrap().date().to_string(), "2026-10-02");
        let fri = parse("2026-10-09", &z).unwrap();
        let mon = parse("2026-10-19", &z).unwrap();
        assert_eq!(workdays(&fri, &mon).unwrap(), 6);
        assert_eq!(workdays(&mon, &fri).unwrap(), -6);
        let birth = parse("1990-10-07", &z).unwrap();
        assert_eq!(age(&birth, &z), 35);
        assert_eq!(diff(&parse("2025-08-03", &z).unwrap(), &parse("2026-10-06 04:00", &z).unwrap()).unwrap(), "1 yr 2 mo 3 d 4 h");
        assert_eq!(relative(&parse("in 3 days", &z).unwrap(), &z), "in 3 days");
        assert_eq!(relative(&parse("2 hours ago", &z).unwrap(), &z), "2 hours ago");
        assert_eq!(with(&z, &[("day".into(), 1)]).unwrap().date().to_string(), "2026-10-01");
        assert!(with(&z, &[("month".into(), 13)]).is_err());
        assert!(calendar(2026, 10).unwrap().ends_with("26 27 28 29 30 31"));
    }
}
