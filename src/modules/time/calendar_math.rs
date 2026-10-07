//! Calendar math: periods, weekdays, business days, month grids. Weeks start Monday.

use super::{e, end_of, is_weekend, start_of, unknown_weekday, weekday};
use crate::interp::Interp;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;
use jiff::{Span, Zoned, civil::Date};

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
            ("ISO week number", "today.iso_week"),
        ]),
    ],
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("fields", &["weekday_num", "day_of_year", "iso_week", "quarter"]),
        ("year", &["leap_year", "days_in_month", "days_in_year", "calendar"]),
        ("moving", &["start_of", "end_of", "next", "prev", "nth_weekday", "add_workdays", "workdays"]),
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
    doc("add_workdays", "add_workdays(d: date, n: int)", "move n Monday-Friday days (no holidays); negative goes back", &["today.add_workdays(10)"], &["workdays", "is_weekend"]),
    doc("workdays", "workdays(a: date, b: date)", "Monday-Friday days from a up to (not including) b", &[r#"workdays(today, date("2026-12-25"))"#], &["add_workdays"]),
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
        ("add_workdays", [Date(z), Int(n, _)]) => Value::date(add_workdays(z, *n)?),
        ("workdays", [Date(a), Date(b)]) => Value::int(workdays(a, b)?),
        ("calendar", [Date(z)]) => Value::str(calendar(z.year() as i64, z.month() as i64)?),
        ("calendar", [Int(y, _), Int(m, _)]) => Value::str(calendar(*y, *m)?),
        _ => return Err(Fail::BadArgs),
    })
}

// ponytail: business-day math walks day by day; fine for calendar-sized spans, closed form if ever needed for centuries.
/// Move `n` Monday-Friday days (negative goes back); keeps the time of day.
pub fn add_workdays(z: &Zoned, n: i64) -> Result<Zoned, String> {
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
pub fn workdays(a: &Zoned, b: &Zoned) -> Result<i64, String> {
    let (mut d, end, sign) = if a.date() <= b.date() { (a.date(), b.date(), 1) } else { (b.date(), a.date(), -1) };
    let mut n = 0;
    while d < end {
        n += i64::from(!is_weekend(d));
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
