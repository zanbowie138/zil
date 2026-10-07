//! Time zones: the same moment on another clock.

use super::e;
use crate::ast::Target;
use crate::interp::Interp;
use crate::modules::{Call, Claim, Doc, Fail, Module, doc};
use crate::value::Value;
use jiff::tz::TimeZone;

pub const MODULE: Module = Module {
    name: "zones",
    about: "the same moment in UTC, local time or any IANA zone",
    #[rustfmt::skip]
    examples: &[
        ("zones", &[
            ("time zones", r#"date("2026-12-25 18:30") to "Asia/Tokyo""#),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[
        ("conversions", &[
            ("to UTC, to local", r#"date("2026-12-25 18:30") to UTC"#),
            ("to \"Zone/Name\"", r#"date("2026-12-25 18:30") to "Asia/Tokyo""#),
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
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &crate::lexer::Span) -> Call {
    Ok(match (name, args) {
        ("utc", [Value::Date(z)]) => Value::date(z.with_time_zone(TimeZone::UTC)),
        ("local", [Value::Date(z)]) => Value::date(z.with_time_zone(TimeZone::system())),
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
        _ => z.in_tz(name).map(Value::date).map_err(e),
    })
}
