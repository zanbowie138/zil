//! Reading dates: ISO 8601 / RFC 9557 / RFC 2822, US and written-out formats, and natural language like "next friday at 5pm".

use super::{add, add_period, end_of, start_of, weekday};
use crate::modules::units::{self, Unit};
use jiff::civil::{self, Date, Time};
use jiff::{Timestamp, Zoned, tz::TimeZone};

/// Any supported date string: ISO 8601 / RFC 9557 / RFC 2822, US and written-out dates, or natural language.
pub fn parse(s: &str, now: &Zoned) -> Option<Zoned> {
    strict(s.trim(), now.time_zone()).or_else(|| natural(s, now))
}

const DATE_FORMATS: &[&str] = &[
    "%m/%d/%y",
    "%d/%m/%y",
    "%m/%d/%Y",
    "%d/%m/%Y", // US first; day/month only when month would be > 12
    "%b %d %Y",
    "%b %d, %Y",
    "%d %b %Y",
    "%d %b, %Y",
    "%a %b %d %Y",
    "%a, %b %d, %Y", // abbreviated month
    "%B %d %Y",
    "%B %d, %Y",
    "%d %B %Y",
    "%d %B, %Y",
    "%A %B %d %Y",
    "%A, %B %d, %Y", // full month
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
    DATE_FORMATS.iter().flat_map(|d| TIME_FORMATS.iter().map(move |t| format!("{d}{t}"))).find_map(|f| with_format(s, &f, tz))
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
        ["next" | "last", wd] if weekday(wd).is_some() => at(today.nth_weekday(if w[0] == "next" { 1 } else { -1 }, weekday(wd)?).ok()?)?,
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
pub fn time(t: &str) -> Option<Time> {
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
