//! Runtime values: display, equality, ordering, number formatting.

use crate::ast::Radix;
use crate::interp::Closure;
use crate::modules::{self, Module, units::Unit};
use indexmap::IndexMap;
use jiff::{Zoned, tz::TimeZone};
use regex::Regex;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    /// Integer and how it displays (`255 to hex` remembers base 16).
    Int(i64, Radix),
    Float(f64),
    Qty(f64, Unit),
    Str(Rc<str>),
    Regex(Rc<Regex>),
    Date(Rc<Zoned>),
    List(Rc<RefCell<Vec<Value>>>),
    Map(Rc<RefCell<IndexMap<String, Value>>>),
    Fn(Rc<Closure>),
    Builtin(&'static Module, &'static str),
}

impl Value {
    pub fn int(n: i64) -> Value {
        Value::Int(n, Radix::DEC)
    }

    pub fn list(v: Vec<Value>) -> Value {
        Value::List(Rc::new(RefCell::new(v)))
    }

    pub fn map(m: IndexMap<String, Value>) -> Value {
        Value::Map(Rc::new(RefCell::new(m)))
    }

    pub fn str(s: impl Into<Rc<str>>) -> Value {
        Value::Str(s.into())
    }

    pub fn date(z: Zoned) -> Value {
        Value::Date(Rc::new(z))
    }

    /// Quantity, collapsing to a plain number when the units cancel (km/m, kWh/J).
    pub fn qty(v: f64, u: Unit) -> Value {
        if u.0.is_empty() {
            Value::Float(v)
        } else if u.dim() == [0; 7] {
            Value::Float(v * u.scale())
        } else {
            Value::Qty(v, u)
        }
    }

    pub fn truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Bool(_) => "bool",
            Value::Int(..) => "int",
            Value::Float(_) => "float",
            Value::Qty(..) => "quantity",
            Value::Str(_) => "str",
            Value::Regex(_) => "regex",
            Value::Date(_) => "date",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Fn(_) | Value::Builtin(..) => "fn",
        }
    }

    fn write(&self, f: &mut fmt::Formatter, top: bool) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(n, base) => write!(f, "{}", fmt_int(*n, *base)),
            Value::Float(n) => write!(f, "{}", fmt_float(*n)),
            Value::Qty(n, u) => write!(f, "{} {u}", fmt_float(*n)),
            Value::Str(s) if top => write!(f, "{s}"),
            Value::Str(s) => write!(f, "{s:?}"),
            Value::Regex(r) => write!(f, "r\"{}\"", r.as_str()),
            // Local midnight is just a date.
            Value::Date(z) if z.time() == jiff::civil::Time::midnight() && z.time_zone().iana_name() == TimeZone::system().iana_name() => {
                write!(f, "{}", z.date())
            }
            Value::Date(z) => write!(f, "{}", z.strftime("%Y-%m-%d %H:%M:%S %:z")),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.borrow().iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    v.write(f, false)?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.borrow().iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: ")?;
                    v.write(f, false)?;
                }
                write!(f, "}}")
            }
            Value::Fn(_) => write!(f, "<fn>"),
            Value::Builtin(_, name) => write!(f, "<builtin {name}>"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write(f, true)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write(f, false)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Bool(a), Bool(b)) => a == b,
            (Int(a, _), Int(b, _)) => a == b,
            (Int(..) | Float(_), Int(..) | Float(_)) => num(self) == num(other),
            (Qty(..), Qty(..)) => compare(self, other) == Some(Ordering::Equal),
            (Str(a), Str(b)) => a == b,
            (Regex(a), Regex(b)) => a.as_str() == b.as_str(),
            (Date(a), Date(b)) => a.timestamp() == b.timestamp(),
            (List(a), List(b)) => *a.borrow() == *b.borrow(),
            (Map(a), Map(b)) => *a.borrow() == *b.borrow(),
            (Fn(a), Fn(b)) => Rc::ptr_eq(a, b),
            (Builtin(_, a), Builtin(_, b)) => a == b,
            _ => false,
        }
    }
}

pub fn fmt_int(n: i64, r: Radix) -> String {
    let fixed = r.width > 0 && fits(n, r.width);
    let (sign, m) = match n {
        _ if fixed => ("", n as u64 & mask(r.width)),
        ..0 => ("-", n.unsigned_abs()),
        _ => ("", n as u64),
    };
    let mut s = digits(m, r.base);
    if fixed && r.base != 10 {
        let w = digits(mask(r.width), r.base).len();
        s = format!("{s:0>w$}");
    }
    match r.base {
        10 => format!("{sign}{s}"),
        16 => format!("{sign}0x{s}"),
        2 => format!("{sign}0b{s}"),
        8 => format!("{sign}0o{s}"),
        b => format!("{sign}{b}#{s}"),
    }
}

fn digits(mut m: u64, base: u32) -> String {
    let mut d = Vec::new();
    loop {
        d.push(std::char::from_digit((m % base as u64) as u32, base).unwrap());
        m /= base as u64;
        if m == 0 {
            break;
        }
    }
    d.iter().rev().collect()
}

/// Whether `n` is representable in `w` bits, signed or unsigned.
pub fn fits(n: i64, w: u32) -> bool {
    let n = n as i128;
    -(1i128 << (w - 1)) <= n && n < (1i128 << w)
}

fn mask(w: u32) -> u64 {
    u64::MAX >> (64 - w)
}

/// 6 significant digits, trailing zeros trimmed; scientific for very large/small.
pub fn fmt_float(x: f64) -> String {
    if !x.is_finite() || x == 0.0 {
        return if x == 0.0 { "0".into() } else { x.to_string() };
    }
    let trim = |s: String| {
        if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s }
    };
    let exp = x.abs().log10().floor() as i32;
    if !(-6..15).contains(&exp) {
        let s = format!("{x:.5e}");
        let (mantissa, e) = s.split_once('e').unwrap();
        return format!("{}e{e}", trim(mantissa.to_string()));
    }
    trim(format!("{:.*}", (5 - exp).max(0) as usize, x))
}

pub fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Int(n, _) => Some(*n as f64),
        Value::Float(n) => Some(*n),
        _ => None,
    }
}

pub fn compare(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::Int(a, _), Value::Int(b, _)) => Some(a.cmp(b)),
        (Value::Str(a), Value::Str(b)) => Some(a.cmp(b)),
        _ => modules::compare(a, b).or_else(|| num(a)?.partial_cmp(&num(b)?)),
    }
}
