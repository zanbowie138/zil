use crate::Error;
use crate::ast::{BinOp, Expr, ExprKind, Radix, Target, UnOp};
use crate::dates;
use crate::help::help;
use crate::interp::{Interp, Value, binary, compare, convert, num, range};
use crate::lexer::Span;
use base64::Engine;
use jiff::{Timestamp, Zoned, civil, tz::TimeZone};
use sha2::Digest;
use std::cmp::Ordering;

pub const NAMES: &[&str] = &[
    // general
    "print", "type", "str", "int", "float", "bool", "list", "hex", "bin", "oct", "dec", "base", "parse", "len", "read_file", "write_file", "help",
    // strings
    "upper", "lower", "trim", "capitalize", "reverse", "split", "lines", "chars", "join", "replace",
    "contains", "starts_with", "ends_with", "find", "count", "match", "find_all", "repeat",
    "base64", "encode", "decode", "sha256", "md5", "ord", "chr", "bytes", "from_bytes", "nums",
    // lists & maps
    "range", "push", "map", "filter", "reduce", "sum", "avg", "min", "max", "sort", "unique",
    "first", "last", "keys", "values",
    // math
    "digits", "from_digits", "sqrt", "abs", "round", "floor", "ceil", "ln", "log", "sin", "cos", "tan", "asin", "acos", "atan",
    // dates
    "date", "year", "month", "day", "hour", "minute", "second", "weekday", "format", "with",
    "weekday_num", "day_of_year", "iso_week", "quarter", "leap_year", "days_in_month", "days_in_year",
    "is_weekend", "is_weekday", "is_today", "is_past", "is_future", "start_of", "end_of", "next", "prev",
    "nth_weekday", "add_workdays", "workdays", "age", "diff", "relative", "parts", "calendar",
    // random
    "rand", "choice", "shuffle", "uuid",
];

pub fn call(it: &mut Interp, name: &str, args: Vec<Value>, span: &Span) -> Result<Value, Error> {
    let err = |msg: String| Error::new(format!("{name}: {msg}"), span.clone());
    let bad_args = || {
        let types: Vec<_> = args.iter().map(Value::type_name).collect();
        err(format!("unsupported arguments ({})", types.join(", ")))
    };
    use Value::*;
    Ok(match (name, args.as_slice()) {
        // general
        ("print", vs) => {
            let parts: Vec<String> = vs.iter().map(Value::to_string).collect();
            println!("{}", parts.join(" "));
            Nil
        }
        ("help", []) => help(None).map(|_| Nil).map_err(err)?,
        ("help", [Str(s)]) => help(Some(s)).map(|_| Nil).map_err(err)?,
        ("help", [Builtin(f)]) => help(Some(f)).map(|_| Nil).map_err(err)?,
        ("help", [Fn(_)]) => return Err(err("user-defined function; no help available".into())),
        ("type", [v]) => Value::str(v.type_name()),
        ("str", [Float(n)]) => Value::str(n.to_string()),
        ("str", [v]) => Value::str(v.to_string()),
        ("int", [Int(n, _)]) => Value::int(*n),
        ("int", [Float(n)]) => Value::int(*n as i64),
        ("int", [Str(s)]) => parse_int(s, None).ok_or_else(|| err(format!("cannot parse {s:?}")))?,
        ("int", [Str(s), Int(b, _)]) if (2..=36).contains(b) => {
            parse_int(s, Some(*b as u32)).ok_or_else(|| err(format!("cannot parse {s:?} in base {b}")))?
        }
        ("float", [v @ (Int(..) | Float(_))]) => Float(num(v).unwrap()),
        ("float", [Qty(n, _)]) => Float(*n),
        ("float", [Str(s)]) => Float(s.trim().parse().map_err(|_| err(format!("cannot parse {s:?}")))?),
        ("bool", [v]) => Bool(v.truthy()),
        // Same as `v to hex`, `v to bin(8)`, `v to base(36)`.
        ("hex" | "bin" | "oct" | "dec" | "base", [v, rest @ ..]) if rest.len() <= 1 => {
            let base = match name {
                "hex" => 16,
                "bin" => 2,
                "oct" => 8,
                "dec" => 10,
                _ => match rest {
                    [Int(b, _)] if (2..=36).contains(b) => *b as u32,
                    _ => return Err(err("expected base(v, 2-36)".into())),
                },
            };
            let width = match (name, rest) {
                ("base", _) | (_, []) => 0,
                (_, [Int(w, _)]) if (1..=64).contains(w) => *w as u32,
                _ => return Err(err("width must be 1-64 bits".into())),
            };
            convert(v.clone(), &Target::Base(Radix { base, width })).map_err(err)?
        }
        ("list", [Str(s)]) => Value::list(s.chars().map(|c| Value::str(c.to_string())).collect()),
        ("list", [List(l)]) => Value::list(l.borrow().clone()),
        ("list", [Map(m)]) => Value::list(m.borrow().iter().map(|(k, v)| Value::list(vec![Value::str(k.as_str()), v.clone()])).collect()),
        ("parse", [Str(s)]) => parse_literal(s).map_err(err)?,
        ("len", [Str(s)]) => Value::int(s.chars().count() as i64),
        ("len", [List(l)]) => Value::int(l.borrow().len() as i64),
        ("len", [Map(m)]) => Value::int(m.borrow().len() as i64),
        ("read_file", [Str(path)]) => Value::str(std::fs::read_to_string(&**path).map_err(|e| err(e.to_string()))?),
        ("write_file", [Str(path), v]) => {
            std::fs::write(&**path, v.to_string()).map_err(|e| err(e.to_string()))?;
            Nil
        }

        // strings
        ("upper", [Str(s)]) => Value::str(s.to_uppercase()),
        ("lower", [Str(s)]) => Value::str(s.to_lowercase()),
        ("trim", [Str(s)]) => Value::str(s.trim()),
        ("capitalize", [Str(s)]) => {
            let mut c = s.chars();
            Value::str(c.next().map(|f| f.to_uppercase().chain(c).collect::<String>()).unwrap_or_default())
        }
        ("reverse", [Str(s)]) => Value::str(s.chars().rev().collect::<String>()),
        ("reverse", [List(l)]) => Value::list(l.borrow().iter().rev().cloned().collect()),
        ("split", [Str(s)]) => strs(s.split_whitespace()),
        ("split", [Str(s), Str(sep)]) => strs(s.split(&**sep)),
        ("split", [Str(s), Regex(r)]) => strs(r.split(s)),
        ("lines", [Str(s)]) => strs(s.lines()),
        ("chars", [Str(s)]) => Value::list(s.chars().map(|c| Value::str(c.to_string())).collect()),
        ("join", [List(l)]) => Value::str(l.borrow().iter().map(Value::to_string).collect::<String>()),
        ("join", [List(l), Str(sep)]) => {
            Value::str(l.borrow().iter().map(Value::to_string).collect::<Vec<_>>().join(sep))
        }
        ("replace", [Str(s), Str(from), Str(to)]) => Value::str(s.replace(&**from, to)),
        ("replace", [Str(s), Regex(r), Str(to)]) => Value::str(r.replace_all(s, &**to)),
        ("contains", [Str(s), Str(sub)]) => Bool(s.contains(&**sub)),
        ("contains", [Str(s), Regex(r)]) => Bool(r.is_match(s)),
        ("contains", [List(l), v]) => Bool(l.borrow().contains(v)),
        ("contains", [Map(m), Str(k)]) => Bool(m.borrow().contains_key(&**k)),
        ("starts_with", [Str(s), Str(p)]) => Bool(s.starts_with(&**p)),
        ("ends_with", [Str(s), Str(p)]) => Bool(s.ends_with(&**p)),
        ("find", [Str(s), Str(sub)]) => s.find(&**sub).map_or(Nil, |i| char_index(s, i)),
        ("find", [Str(s), Regex(r)]) => r.find(s).map_or(Nil, |m| char_index(s, m.start())),
        ("find", [List(l), v]) => l.borrow().iter().position(|x| x == v).map_or(Nil, |i| Value::int(i as i64)),
        ("count", [Str(s), Str(sub)]) if !sub.is_empty() => Value::int(s.matches(&**sub).count() as i64),
        ("count", [Str(s), Regex(r)]) => Value::int(r.find_iter(s).count() as i64),
        ("count", [List(l), v]) => Value::int(l.borrow().iter().filter(|x| *x == v).count() as i64),
        // Whole match, or the list of groups when the regex has capture groups.
        ("match", [Str(s), Regex(r)]) => match r.captures(s) {
            None => Nil,
            Some(c) if c.len() == 1 => Value::str(&c[0]),
            Some(c) => Value::list(c.iter().skip(1).map(|g| g.map_or(Nil, |g| Value::str(g.as_str()))).collect()),
        },
        ("find_all", [Str(s), Regex(r)]) => strs(r.find_iter(s).map(|m| m.as_str())),
        ("find_all", [Str(s), Str(sub)]) if !sub.is_empty() => strs(s.matches(&**sub)),
        ("repeat", [Str(s), Int(n, _)]) => Value::str(s.repeat((*n).max(0) as usize)),
        ("base64", [Str(s)]) => Value::str(base64::engine::general_purpose::STANDARD.encode(s.as_bytes())),
        ("encode", [Str(s), Str(fmt)]) => Value::str(match &**fmt {
            "base64" => base64::engine::general_purpose::STANDARD.encode(s.as_bytes()),
            "url" => url_encode(s),
            "hex" => hex(s.as_bytes()),
            _ => return Err(err(format!("unknown encoding {fmt:?} (base64, url, hex)"))),
        }),
        ("decode", [Str(s), Str(fmt)]) => {
            let bytes = match &**fmt {
                "base64" => base64::engine::general_purpose::STANDARD.decode(s.trim()).map_err(|e| err(e.to_string()))?,
                "url" => url_decode(s).ok_or_else(|| err("invalid percent-encoding".into()))?,
                "hex" => unhex(s.trim()).ok_or_else(|| err("invalid hex".into()))?,
                _ => return Err(err(format!("unknown encoding {fmt:?} (base64, url, hex)"))),
            };
            Value::str(String::from_utf8(bytes).map_err(|_| err("decoded bytes are not UTF-8".into()))?)
        }
        ("sha256", [Str(s)]) => Value::str(hex(&sha2::Sha256::digest(s.as_bytes()))),
        ("md5", [Str(s)]) => Value::str(hex(&md5::Md5::digest(s.as_bytes()))),
        ("ord", [Str(s)]) => match s.chars().collect::<Vec<_>>()[..] {
            [c] => Value::int(c as i64),
            _ => return Err(err("expected a single character".into())),
        },
        ("chr", [Int(n, _)]) => {
            let c = u32::try_from(*n).ok().and_then(char::from_u32);
            Value::str(c.ok_or_else(|| err(format!("{n} is not a valid code point")))?.to_string())
        }
        // Every number in the text; a `-` glued to a word or number (2026-10-06) is a separator, not a sign.
        ("nums", [Str(s)]) => {
            let re = regex::Regex::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?").unwrap();
            let num = |m: regex::Match| {
                let glued = s[..m.start()].ends_with(|c: char| c.is_alphanumeric());
                let t = if glued { m.as_str().trim_start_matches('-') } else { m.as_str() };
                t.parse().map_or_else(|_| Float(t.parse().unwrap_or(f64::NAN)), Value::int)
            };
            Value::list(re.find_iter(s).map(num).collect())
        }
        ("bytes", [Str(s)]) => Value::list(s.bytes().map(|b| Value::int(b as i64)).collect()),
        ("from_bytes", [List(l)]) => {
            let byte = |v: &Value| match v {
                Int(n, _) => u8::try_from(*n).ok(),
                _ => None,
            };
            let bytes: Option<Vec<u8>> = l.borrow().iter().map(byte).collect();
            let bytes = bytes.ok_or_else(|| err("expected a list of integers 0-255".into()))?;
            Value::str(String::from_utf8(bytes).map_err(|_| err("bytes are not UTF-8".into()))?)
        }

        // lists & maps
        ("range", [Int(n, _)]) => range(0, *n).map_err(err)?,
        ("range", [Int(a, _), Int(b, _)]) => range(*a, *b).map_err(err)?,
        ("push", [List(l), v]) => {
            l.borrow_mut().push(v.clone());
            List(l.clone())
        }
        ("map", [List(l), f]) => {
            let items = l.borrow().clone();
            let mut out = Vec::with_capacity(items.len());
            for v in items {
                out.push(it.call(f, vec![v], span)?);
            }
            Value::list(out)
        }
        ("filter", [List(l), f]) => {
            let items = l.borrow().clone();
            let mut out = Vec::new();
            for v in items {
                if it.call(f, vec![v.clone()], span)?.truthy() {
                    out.push(v);
                }
            }
            Value::list(out)
        }
        ("reduce", [List(l), init, f]) => {
            let items = l.borrow().clone();
            let mut acc = init.clone();
            for v in items {
                acc = it.call(f, vec![acc, v], span)?;
            }
            acc
        }
        ("sum", [List(l)]) => sum(&l.borrow()).map_err(err)?,
        ("avg", [List(l)]) => {
            let l = l.borrow();
            if l.is_empty() {
                return Err(err("empty list".into()));
            }
            binary(BinOp::Div, &sum(&l).map_err(err)?, &Float(l.len() as f64)).map_err(err)?
        }
        ("min" | "max", [List(l)]) => extreme(name, &l.borrow()).map_err(err)?,
        ("min" | "max", vs) if vs.len() >= 2 => extreme(name, vs).map_err(err)?,
        ("sort", [List(l)]) => {
            let mut keyed: Vec<_> = l.borrow().iter().map(|v| (v.clone(), v.clone())).collect();
            sort_keyed(&mut keyed).map_err(err)?
        }
        ("sort", [List(l), f]) => {
            let mut keyed = Vec::new();
            for v in l.borrow().clone() {
                keyed.push((it.call(f, vec![v.clone()], span)?, v));
            }
            sort_keyed(&mut keyed).map_err(err)?
        }
        ("unique", [List(l)]) => {
            let mut out: Vec<Value> = Vec::new();
            for v in l.borrow().iter() {
                if !out.contains(v) {
                    out.push(v.clone());
                }
            }
            Value::list(out)
        }
        ("first", [List(l)]) => l.borrow().first().cloned().unwrap_or(Nil),
        ("last", [List(l)]) => l.borrow().last().cloned().unwrap_or(Nil),
        ("keys", [Map(m)]) => strs(m.borrow().keys().map(String::as_str)),
        ("values", [Map(m)]) => Value::list(m.borrow().values().cloned().collect()),

        // math
        ("digits", [Int(n, r)]) => {
            let (mut m, b) = (n.unsigned_abs(), r.base as u64);
            let mut d = vec![Value::int((m % b) as i64)];
            while m >= b {
                m /= b;
                d.push(Value::int((m % b) as i64));
            }
            d.reverse();
            Value::list(d)
        }
        ("from_digits", [List(l)]) => from_digits(&l.borrow(), 10).map_err(err)?,
        ("from_digits", [List(l), Int(b, _)]) if (2..=36).contains(b) => from_digits(&l.borrow(), *b as u32).map_err(err)?,
        ("sqrt", [v]) if num(v).is_some() => Float(num(v).unwrap().sqrt()),
        ("abs" | "round" | "floor" | "ceil", [v]) => round_like(name, v, 0).ok_or_else(bad_args)?,
        ("round", [v, Int(n, _)]) => round_like(name, v, *n).ok_or_else(bad_args)?,
        ("ln", [v]) if num(v).is_some() => Float(num(v).unwrap().ln()),
        ("log", [v]) if num(v).is_some() => Float(num(v).unwrap().log10()),
        ("log", [v, b]) if num(v).is_some() && num(b).is_some() => Float(num(v).unwrap().log(num(b).unwrap())),
        ("sin" | "cos" | "tan", [v]) => {
            let rad = match v {
                Qty(x, u) if u.dim() == crate::units::unit("rad").unwrap().dim() => u.to_si(*x),
                _ => num(v).ok_or_else(bad_args)?,
            };
            Float(match name {
                "sin" => rad.sin(),
                "cos" => rad.cos(),
                _ => rad.tan(),
            })
        }
        ("asin" | "acos" | "atan", [v]) if num(v).is_some() => {
            let x = num(v).unwrap();
            let rad = match name {
                "asin" => x.asin(),
                "acos" => x.acos(),
                _ => x.atan(),
            };
            Qty(rad, crate::units::unit("rad").unwrap())
        }

        // dates
        ("date", [Str(s)]) => Value::date(dates::parse(s, &Zoned::now()).ok_or_else(|| err(format!("cannot parse date {s:?}")))?),
        ("date", [Str(s), Str(f)]) => {
            let z = dates::with_format(s, f, &TimeZone::system());
            Value::date(z.ok_or_else(|| err(format!("{s:?} does not match {f:?}")))?)
        }
        ("date", [Int(y, _), Int(m, _), Int(d, _), rest @ ..]) if rest.len() <= 3 => {
            let mut t = [0i64; 3];
            for (slot, v) in t.iter_mut().zip(rest) {
                let Int(n, _) = v else { return Err(bad_args()) };
                *slot = *n;
            }
            let small = |n: i64| i8::try_from(n).map_err(|_| err(format!("{n} out of range")));
            let year = i16::try_from(*y).map_err(|_| err(format!("year {y} out of range")))?;
            let dt = civil::DateTime::new(year, small(*m)?, small(*d)?, small(t[0])?, small(t[1])?, small(t[2])?, 0);
            Value::date(dt.and_then(|dt| dt.to_zoned(TimeZone::system())).map_err(|e| err(e.to_string()))?)
        }
        ("date", [Int(n, _)]) => {
            let ts = Timestamp::from_second(*n).map_err(|e| err(e.to_string()))?;
            Value::date(ts.to_zoned(TimeZone::system()))
        }
        ("year", [Date(z)]) => Value::int(z.year() as i64),
        ("month", [Date(z)]) => Value::int(z.month() as i64),
        ("day", [Date(z)]) => Value::int(z.day() as i64),
        ("hour", [Date(z)]) => Value::int(z.hour() as i64),
        ("minute", [Date(z)]) => Value::int(z.minute() as i64),
        ("second", [Date(z)]) => Value::int(z.second() as i64),
        ("weekday", [Date(z)]) => Value::str(z.strftime("%A").to_string()),
        ("weekday_num", [Date(z)]) => Value::int(z.weekday().to_monday_one_offset() as i64),
        ("day_of_year", [Date(z)]) => Value::int(z.day_of_year() as i64),
        ("iso_week", [Date(z)]) => Value::int(z.date().iso_week_date().week() as i64),
        ("quarter", [Date(z)]) => Value::int((z.month() as i64 - 1) / 3 + 1),
        ("leap_year", [Date(z)]) => Bool(z.in_leap_year()),
        ("leap_year", [Int(y, _)]) => Bool(y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)),
        ("days_in_month", [Date(z)]) => Value::int(z.days_in_month() as i64),
        ("days_in_year", [Date(z)]) => Value::int(z.days_in_year() as i64),
        ("is_weekend", [Date(z)]) => Bool(dates::is_weekend(z.date())),
        ("is_weekday", [Date(z)]) => Bool(!dates::is_weekend(z.date())),
        ("is_today", [Date(z)]) => Bool(z.date() == Zoned::now().with_time_zone(z.time_zone().clone()).date()),
        ("is_past", [Date(z)]) => Bool(z.timestamp() < Timestamp::now()),
        ("is_future", [Date(z)]) => Bool(z.timestamp() > Timestamp::now()),
        ("with", [Date(z), Map(m)]) => {
            let mut fields = Vec::new();
            for (k, v) in m.borrow().iter() {
                let Int(n, _) = v else { return Err(err(format!("{k} must be an integer"))) };
                fields.push((k.clone(), *n));
            }
            Value::date(dates::with(z, &fields).map_err(err)?)
        }
        ("start_of", [Date(z), Str(p)]) => Value::date(dates::start_of(z, p).map_err(err)?),
        ("end_of", [Date(z), Str(p)]) => Value::date(dates::end_of(z, p).map_err(err)?),
        ("next" | "prev", [Date(z), Str(wd)]) => {
            let wd = dates::weekday(wd).ok_or_else(|| err(format!("unknown weekday {wd:?}")))?;
            let nth = if name == "next" { 1 } else { -1 };
            Value::date(z.nth_weekday(nth, wd).map_err(|e| err(e.to_string()))?)
        }
        ("nth_weekday", [Date(z), Int(n, _), Str(wd)]) => {
            let wd = dates::weekday(wd).ok_or_else(|| err(format!("unknown weekday {wd:?}")))?;
            let n = i8::try_from(*n).map_err(|_| err(format!("{n} out of range")))?;
            Value::date(z.nth_weekday_of_month(n, wd).map_err(|e| err(e.to_string()))?)
        }
        ("add_workdays", [Date(z), Int(n, _)]) => Value::date(dates::add_workdays(z, *n).map_err(err)?),
        ("workdays", [Date(a), Date(b)]) => Value::int(dates::workdays(a, b).map_err(err)?),
        ("age", [Date(z)]) => Value::int(dates::age(z, &Zoned::now())),
        ("diff", [Date(a), Date(b)]) => Value::str(dates::diff(a, b).map_err(err)?),
        ("relative", [Date(z)]) => Value::str(dates::relative(z, &Zoned::now())),
        // Automatic mixed units for a duration: up to three of d, h, min, s, last one rounded.
        ("parts", [Qty(x, u)]) if u.dim() == crate::units::unit("s").unwrap().dim() => {
            let us: Vec<_> = ["d", "h", "min", "s"].iter().map(|n| crate::units::unit(n).unwrap()).collect();
            let si = u.to_si(*x);
            let start = us.iter().position(|v| si.abs() >= v.scale()).unwrap_or(3);
            let us = &us[start..(start + 3).min(4)];
            let last = us.last().unwrap().scale();
            Value::str(crate::units::split((si / last).round() * last, us))
        }
        ("calendar", [Date(z)]) => Value::str(dates::calendar(z.year() as i64, z.month() as i64).map_err(err)?),
        ("calendar", [Int(y, _), Int(m, _)]) => Value::str(dates::calendar(*y, *m).map_err(err)?),
        ("format", [Date(z), Str(f)]) => {
            Value::str(jiff::fmt::strtime::format(f.as_bytes(), &**z).map_err(|e| err(e.to_string()))?)
        }

        // random
        ("rand", []) => Float(fastrand::f64()),
        ("rand", [Int(a, _), Int(b, _)]) if a <= b => Value::int(fastrand::i64(*a..=*b)),
        ("rand", [a, b]) if num(a).is_some() && num(b).is_some() => {
            let (a, b) = (num(a).unwrap(), num(b).unwrap());
            Float(a + fastrand::f64() * (b - a))
        }
        ("choice", [List(l)]) => {
            let l = l.borrow();
            if l.is_empty() {
                return Err(err("empty list".into()));
            }
            l[fastrand::usize(..l.len())].clone()
        }
        ("shuffle", [List(l)]) => {
            let mut v = l.borrow().clone();
            fastrand::shuffle(&mut v);
            Value::list(v)
        }
        ("uuid", []) => {
            let mut b: [u8; 16] = std::array::from_fn(|_| fastrand::u8(..));
            b[6] = (b[6] & 0x0f) | 0x40; // version 4
            b[8] = (b[8] & 0x3f) | 0x80; // RFC 4122 variant
            let h = hex(&b);
            Value::str(format!("{}-{}-{}-{}-{}", &h[..8], &h[8..12], &h[12..16], &h[16..20], &h[20..]))
        }

        _ => return Err(bad_args()),
    })
}

fn strs<'a>(it: impl Iterator<Item = &'a str>) -> Value {
    Value::list(it.map(Value::str).collect())
}

fn char_index(s: &str, byte: usize) -> Value {
    Value::int(s[..byte].chars().count() as i64)
}

/// Accepts `0x`/`0b`/`0o`/`36#` prefixes and `_` separators when no base is given.
fn parse_int(s: &str, base: Option<u32>) -> Option<Value> {
    let s = s.trim().replace('_', "");
    let (neg, s) = match s.strip_prefix('-') {
        Some(rest) => (true, rest.to_string()),
        None => (false, s),
    };
    let (base, digits) = match base {
        Some(b) => (b, s.as_str()),
        None => match s.get(..2) {
            Some("0x" | "0X") => (16, &s[2..]),
            Some("0b" | "0B") => (2, &s[2..]),
            Some("0o" | "0O") => (8, &s[2..]),
            _ => match s.split_once('#') {
                Some((b, d)) => (b.parse().ok().filter(|b| (2..=36).contains(b))?, d),
                None => (10, s.as_str()),
            },
        },
    };
    let n = i64::from_str_radix(digits, base).ok()?;
    Some(Value::Int(if neg { -n } else { n }, Radix { base, width: 0 }))
}

fn sum(l: &[Value]) -> Result<Value, String> {
    let Some((first, rest)) = l.split_first() else { return Ok(Value::int(0)) };
    let mut acc = first.clone();
    for v in rest {
        acc = binary(BinOp::Add, &acc, v)?;
    }
    Ok(acc)
}

fn extreme(name: &str, vs: &[Value]) -> Result<Value, String> {
    let mut best: Option<&Value> = None;
    for v in vs {
        best = Some(match best {
            None => v,
            Some(b) => {
                let ord = compare(v, b).ok_or_else(|| format!("cannot compare {v:?} and {b:?}"))?;
                if (name == "min" && ord.is_lt()) || (name == "max" && ord.is_gt()) { v } else { b }
            }
        });
    }
    best.cloned().ok_or_else(|| "empty list".into())
}

/// Stable sort of (key, value) pairs by key; errors on incomparable keys.
fn sort_keyed(keyed: &mut [(Value, Value)]) -> Result<Value, String> {
    let mut bad = None;
    keyed.sort_by(|(a, _), (b, _)| {
        compare(a, b).unwrap_or_else(|| {
            bad = Some(format!("cannot compare {a:?} and {b:?}"));
            Ordering::Equal
        })
    });
    match bad {
        Some(e) => Err(e),
        None => Ok(Value::list(keyed.iter().map(|(_, v)| v.clone()).collect())),
    }
}

/// abs/round/floor/ceil on numbers and quantities (keeping the unit).
fn round_like(name: &str, v: &Value, digits: i64) -> Option<Value> {
    let f = |x: f64| -> f64 {
        let p = 10f64.powi(digits as i32);
        match name {
            "abs" => x.abs(),
            "round" => (x * p).round() / p,
            "floor" => x.floor(),
            _ => x.ceil(),
        }
    };
    Some(match v {
        Value::Int(n, b) if name == "abs" => Value::Int(n.checked_abs()?, *b),
        Value::Int(..) if digits >= 0 => v.clone(),
        Value::Float(x) if digits == 0 && name != "abs" && f(*x).abs() < 9.2e18 => Value::int(f(*x) as i64),
        Value::Float(x) => Value::Float(f(*x)),
        Value::Qty(x, u) => Value::Qty(f(*x), u.clone()),
        _ => return None,
    })
}


fn from_digits(l: &[Value], base: u32) -> Result<Value, String> {
    let mut acc: i64 = 0;
    for v in l {
        let d = match v {
            Value::Int(d, _) if (0..base as i64).contains(d) => *d,
            _ => return Err(format!("{v:?} is not a base-{base} digit")),
        };
        acc = acc.checked_mul(base as i64).and_then(|a| a.checked_add(d)).ok_or("integer overflow")?;
    }
    Ok(Value::Int(acc, Radix { base, width: 0 }))
}

/// A single zil literal (number, string, list, map, quantity); anything that could run code is refused.
fn parse_literal(s: &str) -> Result<Value, String> {
    fn literal(e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Nil | ExprKind::Bool(_) | ExprKind::Int(..) | ExprKind::Float(_) | ExprKind::Str(_) | ExprKind::Regex(_) => true,
            ExprKind::List(items) => items.iter().all(literal),
            ExprKind::Map(entries) => entries.iter().all(|(_, v)| literal(v)),
            ExprKind::Qty(n, _) | ExprKind::Unary(UnOp::Neg, n) => literal(n),
            _ => false,
        }
    }
    let ast = crate::lexer::lex(s).and_then(crate::parser::parse).map_err(|e| e.msg)?;
    match &ast[..] {
        [e] if literal(e) => Interp::new().run(&ast).map_err(|e| e.msg),
        _ => Err(format!("not a literal value: {s:?}")),
    }
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

fn url_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn url_decode(s: &str) -> Option<Vec<u8>> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' => {
                out.push(u8::from_str_radix(s.get(i + 1..i + 3)?, 16).ok()?);
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{eval, try_eval};

    fn show(src: &str) -> String {
        eval(src).to_string()
    }

    #[test]
    fn strings() {
        assert_eq!(show(r##""a1b22c333".replace(r"\d+", "#")"##), "a#b#c#");
        assert_eq!(show(r#""a.b.c".replace(".", "-")"#), "a-b-c");
        assert_eq!(show(r#""2026-10-06".match(r"(\d+)-(\d+)-(\d+)")"#), r#"["2026", "10", "06"]"#);
        assert_eq!(show(r#""x1 y22 z333".find_all(r"\d+")"#), r#"["1", "22", "333"]"#);
        assert_eq!(show(r#""  a  b ".split"#), r#"["a", "b"]"#);
        assert_eq!(show(r#""hello world".capitalize"#), "Hello world");
        assert_eq!(show(r#""ff".int(16)"#), "0xff");
        assert_eq!(show(r#""0b101".int"#), "0b101");
        assert_eq!(show(r#""z".int(36)"#), "36#z");
        assert_eq!(show(r#""-36#z".int to dec"#), "-35");
    }

    #[test]
    fn encodings() {
        assert_eq!(show(r#""hi there".encode("base64")"#), "aGkgdGhlcmU=");
        assert_eq!(show(r#""aGkgdGhlcmU=".decode("base64")"#), "hi there");
        assert_eq!(show(r#""a b&c".encode("url")"#), "a%20b%26c");
        assert_eq!(show(r#""a%20b%26c".decode("url")"#), "a b&c");
        assert_eq!(show(r#""hi".encode("hex").decode("hex")"#), "hi");
        assert_eq!(show(r#""A".ord"#), "65");
        assert_eq!(show("0x1f600.chr"), "😀");
        assert_eq!(show(r#""hé".bytes"#), "[104, 195, 169]");
        assert_eq!(show(r#""hé".bytes.from_bytes"#), "hé");
        assert!(try_eval(r#""ab".ord"#).is_err());
        assert!(try_eval("[256].from_bytes").is_err());
        assert_eq!(show(r#"list("ab")"#), r#"["a", "b"]"#);
        assert_eq!(show("{a: 1}.list"), r#"[["a", 1]]"#);
        assert_eq!(show("1234.digits"), "[1, 2, 3, 4]");
        assert_eq!(show("0b110.digits"), "[1, 1, 0]");
        assert_eq!(show("0.digits"), "[0]");
        assert_eq!(show("[1, 1, 0].from_digits(2)"), "0b110");
        assert_eq!(show("0xff.digits.from_digits(16)"), "0xff");
        assert!(try_eval("[2].from_digits(2)").is_err());
        assert_eq!(show(r#""x=3, y=-2.5 on 2026-10-06, 1e3".nums"#), "[3, -2.5, 2026, 10, 6, 1000]");
        assert_eq!(show(r#""[1, -2.5, \"a\", 0xff, 5 km, \{k: [nil, true]}]".parse"#), r#"[1, -2.5, "a", 0xff, 5 km, {k: [nil, true]}]"#);
        assert_eq!(show(r#"x = [1, "b"]
str(x).parse == x"#), "true");
        for bad in [r#""read_file(\"x\")".parse"#, r#""[x]".parse"#, r#""1 + 2".parse"#, r#""".parse"#] {
            assert!(try_eval(bad).is_err(), "{bad}");
        }
        assert_eq!(show(r#""abc".sha256"#), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(show(r#""abc".md5"#), "900150983cd24fb0d6963f7d28e17f72");
    }

    #[test]
    fn lists() {
        assert_eq!(show("range(5).reduce(0, \\acc, x -> acc + x)"), "10");
        assert_eq!(show("[3, 1, 2].sort"), "[1, 2, 3]");
        assert_eq!(show(r#"["bb", "a", "ccc"].sort(\s -> s.len)"#), r#"["a", "bb", "ccc"]"#);
        assert_eq!(show("[1, 2, 2, 3].unique"), "[1, 2, 3]");
        assert_eq!(show("[1, 2, 3, 4].avg"), "2.5");
        assert_eq!(show("[1 m, 50 cm].sum"), "1.5 m");
        assert_eq!(show("max(3, 9, 4)"), "9");
        assert_eq!(show("[1 km, 900 m].min"), "900 m");
    }

    #[test]
    fn math() {
        assert_eq!(show("sqrt(16)"), "4");
        assert_eq!(show("round(pi, 2)"), "3.14");
        assert_eq!(show("round(2.5 km)"), "3 km");
        assert_eq!(show("sin(30 deg)"), "0.5");
        assert_eq!(show("asin(1) to deg"), "90 deg");
        assert_eq!(show("log(1000)"), "3");
    }

    #[test]
    fn random() {
        assert!(matches!(eval("rand(1, 6)"), crate::interp::Value::Int(1..=6, _)));
        assert_eq!(show("uuid().len"), "36");
        assert_eq!(show("[1, 2, 3].shuffle.sort"), "[1, 2, 3]");
        assert!(try_eval("choice([])").is_err());
    }
}
