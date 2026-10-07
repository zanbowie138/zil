//! Calendar math: periods, weekdays, business days, month grids. Weeks start Monday.

use super::{e, end_of, is_weekend, start_of, unknown_weekday, weekday};
use crate::interp::Interp;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;
use jiff::civil::{Date, Weekday};
use jiff::{Span, Zoned};

pub const MODULE: Module = Module {
    name: "calendar_math",
    about: "periods, weekdays, business days, week numbers, month grids; weeks start Monday",
    #[rustfmt::skip]
    examples: &[
        ("calendar_math", &[
            ("Thanksgiving: 4th Thursday", r#"date(2026, 11, 1).nth_weekday(4, "thu")"#),
            ("Memorial Day: last Monday", r#"date(2026, 5, 1).nth_weekday(-1, "mon")"#),
            ("3 business days later", r#"date("2026-12-24").add_workdays(3)"#),
            ("workdays in December", r#"workdays(date("2026-12-01"), date("2026-12-31"))"#),
            ("skipping US holidays", r#"date("2026-12-24").add_workdays(3, "US")"#),
            ("ISO week number", "today.iso_week"),
        ]),
    ],
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("fields", &["weekday_num", "day_of_year", "iso_week", "quarter"]),
        ("year", &["leap_year", "days_in_month", "days_in_year", "calendar"]),
        ("moving", &["start_of", "end_of", "next", "prev", "nth_weekday", "add_workdays", "workdays", "holidays"]),
    ],
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("weekday_num", "weekday_num(d: date)", "weekday as a number, Monday = 1 ... Sunday = 7", &["today.weekday_num"], &["weekday"]),
    doc("day_of_year", "day_of_year(d: date)", "day of the year, 1-366", &[r#"date("2026-12-31").day_of_year"#], &["iso_week"]),
    doc("iso_week", "iso_week(d: date)", "ISO 8601 week number (weeks start Monday)", &[r#"date("2026-12-31").iso_week"#], &["day_of_year", "quarter"]),
    doc("quarter", "quarter(d: date)", "quarter of the year, 1-4", &["today.quarter"], &["iso_week"]),
    doc("leap_year", "leap_year(d: date|int)", "whether the year is a leap year", &["leap_year(2028)", "today.leap_year"], &["days_in_year"]),
    doc("days_in_month", "days_in_month(d: date)", "number of days in the date's month", &[r#"date("2028-02-10").days_in_month"#], &["days_in_year"]),
    doc("days_in_year", "days_in_year(d: date)", "365 or 366", &["today.days_in_year"], &["leap_year"]),
    doc("start_of", "start_of(d: date, period: str)", "start of the second/minute/hour/day/week/month/quarter/year", &[r#"now.start_of("week")"#, r#"now.start_of("quarter")"#], &["end_of", "with"]),
    doc("end_of", "end_of(d: date, period: str)", "last moment of the period", &[r#"now.end_of("month")"#, r#"(today.end_of("year") - now).parts"#], &["start_of"]),
    doc("next", "next(d: date, weekday: str)", "the next given weekday strictly after d", &[r#"today.next("friday")"#, r#"now.next("mon")"#], &["prev", "nth_weekday"]),
    doc("prev", "prev(d: date, weekday: str)", "the last given weekday strictly before d", &[r#"today.prev("sunday")"#], &["next"]),
    doc("nth_weekday", "nth_weekday(d: date, n: int, weekday: str)", "nth weekday of d's month; negative counts from the end", &[r#"date(2026, 11, 1).nth_weekday(4, "thu")"#, r#"date(2026, 5, 1).nth_weekday(-1, "mon")"#], &["next"]),
    doc("add_workdays", "add_workdays(d: date, n: int, holidays?: list|str)", "move n Monday-Friday days, skipping holidays (a list of dates, or \"US\"/\"UK\"); negative goes back",
        &["today.add_workdays(10)", r#"date("2026-12-24").add_workdays(1, "US")"#, r#"date("2026-12-24").add_workdays(1, [date("2026-12-28")])"#], &["workdays", "holidays", "is_weekend"]),
    doc("workdays", "workdays(a: date, b: date, holidays?: list|str)", "Monday-Friday days from a up to (not including) b, minus holidays",
        &[r#"workdays(today, date("2026-12-25"))"#, r#"workdays(date("2026-12-01"), date("2027-01-01"), "UK")"#], &["add_workdays", "holidays"]),
    doc("holidays", "holidays(year: int, country: str)", "public holidays as observed (moved off weekends): \"US\" federal or \"UK\" England bank holidays",
        &[r#"holidays(2026, "US")"#, r#"holidays(2026, "UK").len"#], &["add_workdays", "workdays"]),
    doc("calendar", "calendar(d: date) / calendar(year: int, month: int)", "month grid, weeks starting Monday", &["calendar(2026, 12)"], &["date"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &crate::lexer::Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("weekday_num", [Date(z)]) => Value::int(z.weekday().to_monday_one_offset() as i64),
        ("day_of_year", [Date(z)]) => Value::int(z.day_of_year() as i64),
        ("iso_week", [Date(z)]) => Value::int(z.date().iso_week_date().week() as i64),
        ("quarter", [Date(z)]) => Value::int((z.month() as i64 - 1) / 3 + 1),
        ("leap_year", [Date(z)]) => Bool(z.in_leap_year()),
        ("leap_year", [Int(y, _)]) => Bool(y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)),
        ("days_in_month", [Date(z)]) => Value::int(z.days_in_month() as i64),
        ("days_in_year", [Date(z)]) => Value::int(z.days_in_year() as i64),
        ("start_of", [Date(z), Str(p)]) => Value::date(start_of(z, p).map_err(|m| Fail::Arg(1, m))?),
        ("end_of", [Date(z), Str(p)]) => Value::date(end_of(z, p).map_err(|m| Fail::Arg(1, m))?),
        ("next" | "prev", [Date(z), Str(wd)]) => {
            let wd = weekday(wd).ok_or_else(|| Fail::Arg(1, unknown_weekday(wd)))?;
            let nth = if name == "next" { 1 } else { -1 };
            Value::date(z.nth_weekday(nth, wd).map_err(|e| e.to_string())?)
        }
        ("nth_weekday", [Date(z), Int(n, _), Str(wd)]) => {
            let wd = weekday(wd).ok_or_else(|| Fail::Arg(2, unknown_weekday(wd)))?;
            let n = i8::try_from(*n)
                .map_err(|_| Fail::Arg(1, format!("{n} is out of range\nnote: use 1 to 5 from the start of the month, or -1 to -5 from the end")))?;
            Value::date(z.nth_weekday_of_month(n, wd).map_err(|e| e.to_string())?)
        }
        ("add_workdays", [Date(z), Int(n, _), rest @ ..]) if rest.len() <= 1 => Value::date(add_workdays(z, *n, &off(rest, 2)?)?),
        ("workdays", [Date(a), Date(b), rest @ ..]) if rest.len() <= 1 => Value::int(workdays(a, b, &off(rest, 2)?)?),
        ("holidays", [Int(y, _), Str(c)]) => {
            let y = i16::try_from(*y).map_err(|_| Fail::Arg(0, format!("year {y} out of range")))?;
            let c = Off::Country(country(c).ok_or_else(|| Fail::Arg(1, unknown_country(c)))?);
            Value::list(c.dates(y).into_iter().map(|d| Value::date(d.to_zoned(jiff::tz::TimeZone::system()).expect("holiday in range"))).collect())
        }
        ("calendar", [Date(z)]) => Value::str(calendar(z.year() as i64, z.month() as i64)?),
        ("calendar", [Int(y, _), Int(m, _)]) => Value::str(calendar(*y, *m)?),
        _ => return Err(Fail::BadArgs),
    })
}

/// Days off besides weekends: given dates, or a country's public holidays.
pub enum Off {
    Dates(Vec<Date>),
    Country(&'static str),
}

impl Off {
    // ponytail: recomputes a year's holidays per day checked; cache by year if long spans get slow.
    fn has(&self, d: Date) -> bool {
        is_weekend(d)
            || match self {
                Off::Dates(ds) => ds.contains(&d),
                Off::Country(_) => self.dates(d.year()).contains(&d),
            }
    }

    /// The holidays in `year`, as observed.
    fn dates(&self, year: i16) -> Vec<Date> {
        let Off::Country(c) = self else { return vec![] };
        let day = |m, d| Date::new(year, m, d).expect("valid holiday");
        let nth = |m, n, wd| day(m, 1).nth_weekday_of_month(n, wd).expect("valid holiday");
        let mut out = Vec::new();
        if *c == "US" {
            // Saturday holidays are observed Friday, Sunday ones Monday.
            for d in [day(1, 1), day(6, 19), day(7, 4), day(11, 11), day(12, 25)] {
                out.push(match d.weekday() {
                    Weekday::Saturday => d.yesterday().expect("in range"),
                    Weekday::Sunday => d.tomorrow().expect("in range"),
                    _ => d,
                });
            }
            out.extend([nth(1, 3, Weekday::Monday), nth(2, 3, Weekday::Monday), nth(5, -1, Weekday::Monday), nth(9, 1, Weekday::Monday)]);
            out.extend([nth(10, 2, Weekday::Monday), nth(11, 4, Weekday::Thursday)]);
        } else {
            // A weekend holiday moves to the next weekday not already a holiday (Christmas and Boxing Day).
            for mut d in [day(1, 1), day(12, 25), day(12, 26)] {
                while is_weekend(d) || out.contains(&d) {
                    d = d.tomorrow().expect("in range");
                }
                out.push(d);
            }
            let easter = easter(year);
            let shift = |n| easter.checked_add(Span::new().days(n)).expect("in range");
            out.extend([shift(-2), shift(1), nth(5, 1, Weekday::Monday), nth(5, -1, Weekday::Monday), nth(8, -1, Weekday::Monday)]);
        }
        out.sort();
        out
    }
}

/// Easter Sunday (Gregorian), by the anonymous algorithm.
fn easter(year: i16) -> Date {
    let y = year as i32;
    let (a, b, c) = (y % 19, y / 100, y % 100);
    let (d, e) = (b / 4, b % 4);
    let g = (8 * b + 13) / 25;
    let h = (19 * a + b - d - g + 15) % 30;
    let (i, k) = (c / 4, c % 4);
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 19 * l) / 433;
    let month = (h + l - 7 * m + 90) / 25;
    let day = (h + l - 7 * m + 33 * month + 19) % 32;
    Date::new(year, month as i8, day as i8).expect("valid Easter")
}

const COUNTRIES: [&str; 2] = ["US", "UK"];

fn country(c: &str) -> Option<&'static str> {
    COUNTRIES.into_iter().find(|n| n.eq_ignore_ascii_case(c))
}

fn unknown_country(c: &str) -> String {
    format!("unknown holiday set {c:?}\nnote: built-in sets are {}; or pass a list of dates", COUNTRIES.join(", "))
}

/// The optional holidays argument at position `i`: a list of dates or a country.
fn off(rest: &[Value], i: usize) -> Result<Off, Fail> {
    Ok(match rest {
        [] => Off::Dates(vec![]),
        [Value::Str(c)] => Off::Country(country(c).ok_or_else(|| Fail::Arg(i, unknown_country(c)))?),
        [Value::List(l)] => Off::Dates(
            l.borrow()
                .iter()
                .map(|v| match v {
                    Value::Date(z) => Ok(z.date()),
                    v => Err(Fail::Arg(i, format!("holidays must be dates, got {}", v.type_name()))),
                })
                .collect::<Result<_, _>>()?,
        ),
        _ => return Err(Fail::BadArgs),
    })
}

// ponytail: business-day math walks day by day; fine for calendar-sized spans, closed form if ever needed for centuries.
/// Move `n` Monday-Friday days that aren't holidays (negative goes back); keeps the time of day.
pub fn add_workdays(z: &Zoned, n: i64, off: &Off) -> Result<Zoned, String> {
    let step = if n < 0 { -1 } else { 1 };
    let mut d = z.date();
    for _ in 0..n.abs() {
        d = d.checked_add(Span::new().days(step)).map_err(e)?;
        while off.has(d) {
            d = d.checked_add(Span::new().days(step)).map_err(e)?;
        }
    }
    z.with().date(d).build().map_err(e)
}

/// Monday-Friday days that aren't holidays from `a` up to (not including) `b`; negative if `b` is earlier.
pub fn workdays(a: &Zoned, b: &Zoned, off: &Off) -> Result<i64, String> {
    let (mut d, end, sign) = if a.date() <= b.date() { (a.date(), b.date(), 1) } else { (b.date(), a.date(), -1) };
    let mut n = 0;
    while d < end {
        n += i64::from(!off.has(d));
        d = d.tomorrow().map_err(e)?;
    }
    Ok(n * sign)
}

/// Month grid like `cal`, Monday first.
pub fn calendar(year: i64, month: i64) -> Result<String, String> {
    let y = i16::try_from(year).map_err(|_| format!("year {year} out of range"))?;
    let m = i8::try_from(month).map_err(|_| format!("month {month} out of range"))?;
    let first = Date::new(y, m, 1).map_err(e)?;
    let mut cells = vec!["  ".to_string(); first.weekday().to_monday_zero_offset() as usize];
    cells.extend((1..=first.days_in_month()).map(|d| format!("{d:>2}")));
    let rows: Vec<String> = cells.chunks(7).map(|c| c.join(" ")).collect();
    let title = first.strftime("%B %Y").to_string();
    Ok(format!("{title:^20}\nMo Tu We Th Fr Sa Su\n{}", rows.join("\n")))
}
