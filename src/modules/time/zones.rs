//! Time zones: the same moment on another clock. Zones are IANA names or city names (`"Tokyo"`, `"new york"`).

use super::e;
use crate::ast::Target;
use crate::interp::Interp;
use crate::modules::{Call, Claim, Doc, Fail, Module, doc};
use crate::value::Value;
use jiff::Zoned;
use jiff::tz::TimeZone;

pub const MODULE: Module = Module {
    name: "zones",
    about: "the same moment in UTC, local time, any IANA zone or a city; world clocks",
    #[rustfmt::skip]
    examples: &[
        ("zones", &[
            ("time zones", r#"date("2026-12-25 18:30") to "Asia/Tokyo""#),
            ("meeting time abroad", r#"5pm to "London""#),
            ("world clock", r#"clock(["Tokyo", "London", "New York"])"#),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[
        ("conversions", &[
            ("to UTC, to local", r#"date("2026-12-25 18:30") to UTC"#),
            ("to \"Zone/Name\"", r#"date("2026-12-25 18:30") to "Asia/Tokyo""#),
            ("to \"City\"", r#"date("2026-12-25 18:30") to "Sao Paulo""#),
        ]),
    ],
    fns: FNS,
    call,
    targets: &[("UTC", "utc"), ("utc", "utc"), ("local", "local")],
    convert: Some(convert),
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("utc", "utc(d: date)", "the same moment in UTC; same as `d to UTC` (`d to \"Asia/Tokyo\"` for any zone)", &["date(0) to UTC", "date(0).utc.year"], &["local", "date"]),
    doc("local", "local(d: date)", "the same moment in the system time zone; same as `d to local`", &["(date(0) to UTC).local.year"], &["utc"]),
    doc("clock", "clock(places: list) / clock(place: str)", "the time now in each zone or city", &[r#"clock(["Tokyo", "London"])"#, r#"clock("Asia/Kolkata").hour >= 0"#], &["utc", "local"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &crate::lexer::Span) -> Call {
    Ok(match (name, args) {
        ("utc", [Value::Date(z)]) => Value::date(z.with_time_zone(TimeZone::UTC)),
        ("local", [Value::Date(z)]) => Value::date(z.with_time_zone(TimeZone::system())),
        ("clock", [Value::Str(p)]) => Value::date(Zoned::now().with_time_zone(zone(p).map_err(|m| Fail::Arg(0, m))?)),
        ("clock", [Value::List(l)]) => {
            let mut m = indexmap::IndexMap::new();
            for (i, p) in l.borrow().iter().enumerate() {
                let Value::Str(p) = p else {
                    return Err(Fail::Arg(0, format!("item {i} must be a str, got {}", p.type_name())));
                };
                m.insert(p.to_string(), Value::date(Zoned::now().with_time_zone(zone(p).map_err(|m| Fail::Arg(0, m))?)));
            }
            Value::map(m)
        }
        _ => return Err(Fail::BadArgs),
    })
}

/// `d to "Europe/Paris"`.
fn convert(v: &Value, t: &Target) -> Claim {
    let (Value::Date(z), Target::Str(name)) = (v, t) else {
        return None;
    };
    Some(match name.as_str() {
        "local" => Ok(Value::date(z.with_time_zone(TimeZone::system()))),
        _ => zone(name).map(|tz| Value::date(z.with_time_zone(tz))),
    })
}

/// An IANA name (`"Asia/Tokyo"`) or the city part of one, any case, spaces for underscores (`"new york"`).
pub fn zone(name: &str) -> Result<TimeZone, String> {
    let db = jiff::tz::db();
    if let Ok(tz) = db.get(name) {
        return Ok(tz);
    }
    let city = name.trim().replace(' ', "_");
    let names: Vec<_> = db.available().collect();
    let full = names.iter().map(|n| n.as_str()).find(|n| n.rsplit('/').next().is_some_and(|c| c.eq_ignore_ascii_case(&city)));
    match full {
        Some(n) => db.get(n).map_err(e),
        None => {
            let cities: Vec<_> = names.iter().filter_map(|n| n.as_str().rsplit_once('/').map(|(_, c)| c.replace('_', " ").to_lowercase())).collect();
            let hint = crate::error::did_you_mean(&name.to_lowercase(), cities.iter().map(|c| c.as_str()));
            Err(format!("unknown time zone or city {name:?}{hint}"))
        }
    }
}
