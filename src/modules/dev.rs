//! Developer tools: JWT and UUID inspection, and the parent of net, binary and colors.

pub mod binary;
pub mod colors;
pub mod net;

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;
use base64::Engine;
use indexmap::IndexMap;
use jiff::{Timestamp, tz::TimeZone};

pub const MODULE: Module = Module {
    name: "dev",
    about: "developer tools: JWTs, UUIDs, IP addresses and subnets, raw bytes, colors",
    #[rustfmt::skip]
    examples: &[
        ("dev", &[
            ("what's in this token?", r#"jwt("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZGEiLCJpYXQiOjE1MTYyMzkwMjJ9.c2ln").payload"#),
            ("when was this UUID made?", r#"uuid_info("01890a5d-ac96-774b-bcce-b302099a8057").timestamp"#),
        ]),
    ],
    fns: FNS,
    call,
    children: &[net::MODULE, binary::MODULE, colors::MODULE],
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("jwt", "jwt(s: str)", "decode a JWT into {header, payload, signature} without verifying it; exp, iat and nbf become dates",
        &[r#"jwt("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZGEiLCJleHAiOjE5MjQ5OTIwMDB9.c2ln").payload.exp"#], &["decode", "hmac"]),
    doc("uuid_info", "uuid_info(s: str)", "version and variant of a UUID, plus its timestamp for v1 and v7",
        &[r#"uuid_info(uuid())"#, r#"uuid_info("c232ab00-9414-11ec-b3c8-9f6bdeced846")"#], &["uuid"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    Ok(match (name, args) {
        ("jwt", [Value::Str(s)]) => jwt(s).map_err(|e| Fail::Arg(0, e.into()))?,
        ("uuid_info", [Value::Str(s)]) => uuid_info(s).ok_or_else(|| Fail::Arg(0, format!("`{s}` is not a UUID")))?,
        _ => return Err(Fail::BadArgs),
    })
}

fn jwt(s: &str) -> Result<Value, &'static str> {
    let [header, payload, signature] = s.trim().split('.').collect::<Vec<_>>()[..] else {
        return Err("expected a JWT: three base64url parts separated by `.`");
    };
    let part = |p: &str| -> Result<Value, &'static str> {
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(p.trim_end_matches('=')).map_err(|_| "invalid base64url in JWT")?;
        Ok(from_json(serde_json::from_slice(&bytes).map_err(|_| "JWT part is not JSON")?))
    };
    let payload = part(payload)?;
    if let Value::Map(m) = &payload {
        for (k, v) in m.borrow_mut().iter_mut() {
            if let ("exp" | "iat" | "nbf", Value::Int(n, _)) = (k.as_str(), &*v)
                && let Ok(ts) = Timestamp::from_second(*n)
            {
                *v = Value::date(ts.to_zoned(TimeZone::system()));
            }
        }
    }
    let fields = [("header", part(header)?), ("payload", payload), ("signature", Value::str(signature))];
    Ok(Value::map(fields.into_iter().map(|(k, v)| (k.to_string(), v)).collect()))
}

fn from_json(j: serde_json::Value) -> Value {
    use serde_json::Value as J;
    match j {
        J::Null => Value::Nil,
        J::Bool(b) => Value::Bool(b),
        J::Number(n) => n.as_i64().map_or_else(|| Value::Float(n.as_f64().unwrap_or(f64::NAN)), Value::int),
        J::String(s) => Value::str(s),
        J::Array(a) => Value::list(a.into_iter().map(from_json).collect()),
        J::Object(o) => Value::map(o.into_iter().map(|(k, v)| (k, from_json(v))).collect::<IndexMap<_, _>>()),
    }
}

fn uuid_info(s: &str) -> Option<Value> {
    let h: String = s.trim().trim_start_matches("urn:uuid:").chars().filter(|&c| !"{}-".contains(c)).collect();
    let n = u128::from_str_radix(&h, 16).ok().filter(|_| h.len() == 32)?;
    let b = n.to_be_bytes();
    let version = b[6] >> 4;
    let variant = match b[8] >> 5 {
        0..=3 => "NCS",
        4 | 5 => "RFC 9562",
        6 => "Microsoft",
        _ => "reserved",
    };
    let time = match version {
        // 100 ns ticks since 1582-10-15, split low/mid/high across the first 8 bytes.
        1 => {
            let ticks = (n >> 64 & 0xfff) << 48 | (n >> 80 & 0xffff) << 32 | n >> 96;
            Timestamp::from_microsecond((ticks as i64 - 0x01B2_1DD2_1381_4000) / 10).ok()
        }
        7 => Timestamp::from_millisecond((n >> 80) as i64).ok(),
        _ => None,
    };
    let mut m = IndexMap::from([("version".to_string(), Value::int(version as i64)), ("variant".to_string(), Value::str(variant))]);
    if let Some(t) = time {
        m.insert("timestamp".to_string(), Value::date(t.to_zoned(TimeZone::system())));
    }
    Some(Value::map(m))
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn jwt() {
        let t = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert_eq!(show(&format!(r#"jwt("{t}").header"#)), r#"{alg: "HS256", typ: "JWT"}"#);
        assert_eq!(show(&format!(r#"jwt("{t}").payload.name"#)), "John Doe");
        assert_eq!(show(&format!(r#"jwt("{t}").payload.iat.unix"#)), "1516239022");
        assert_eq!(show(&format!(r#"jwt("{t}").signature"#)), "SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c");
        assert!(try_eval(r#"jwt("abc")"#).is_err());
    }

    #[test]
    fn uuid_info() {
        // RFC 9562 examples: both made at 2022-02-22 19:22:22 UTC.
        assert_eq!(show(r#"uuid_info("C232AB00-9414-11EC-B3C8-9F6BDECED846").timestamp.unix"#), "1645557742");
        assert_eq!(show(r#"uuid_info("017F22E2-79B0-7CC3-98C4-DC0C0C07398F").timestamp.unix"#), "1645557742");
        assert_eq!(show(r#"uuid_info(uuid())"#), r#"{version: 4, variant: "RFC 9562"}"#);
        assert!(try_eval(r#"uuid_info("nope")"#).is_err());
    }
}
