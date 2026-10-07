use crate::Error;
use crate::ast::{BinOp, Expr, ExprKind, FnDef, Radix, Target, UnOp, UnitSpec};
use crate::builtins::hex;
use crate::lexer::Span;
use crate::units::{self, Unit};
use indexmap::IndexMap;
use jiff::{Zoned, tz::TimeZone};
use regex::Regex;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
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
    Builtin(&'static str),
}

// ponytail: closures hold their defining Env, so recursive fns form Rc cycles and leak; add a GC/arena if long-running scripts care.
pub struct Closure {
    def: Rc<FnDef>,
    env: Env,
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
            Value::Fn(_) | Value::Builtin(_) => "fn",
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
            Value::Builtin(name) => write!(f, "<builtin {name}>"),
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
            (Builtin(a), Builtin(b)) => a == b,
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
fn fits(n: i64, w: u32) -> bool {
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
    let trim = |s: String| if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
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
        (Value::Date(a), Value::Date(b)) => Some(a.timestamp().cmp(&b.timestamp())),
        (Value::Qty(x, u), Value::Qty(y, w)) if u.dim() == w.dim() => u.to_si(*x).partial_cmp(&w.to_si(*y)),
        _ => num(a)?.partial_cmp(&num(b)?),
    }
}

pub fn unit_of(spec: &UnitSpec) -> Result<Unit, String> {
    let mut u = Unit::default();
    for (name, p) in spec {
        u = u.mul(&units::unit(name)?.pow(*p), 1);
    }
    Ok(u)
}

pub type Env = Rc<RefCell<Scope>>;

#[derive(Default)]
pub struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Env>,
}

fn child(parent: &Env) -> Env {
    Rc::new(RefCell::new(Scope { vars: HashMap::new(), parent: Some(parent.clone()) }))
}

fn lookup(env: &Env, name: &str) -> Option<Value> {
    let s = env.borrow();
    match s.vars.get(name) {
        Some(v) => Some(v.clone()),
        None => lookup(s.parent.as_ref()?, name),
    }
}

/// Update an existing variable in the nearest scope that has it.
fn assign(env: &Env, name: &str, v: Value) -> Result<(), Value> {
    let mut s = env.borrow_mut();
    if let Some(slot) = s.vars.get_mut(name) {
        *slot = v;
        return Ok(());
    }
    match &s.parent {
        Some(p) => assign(p, name, v),
        None => Err(v),
    }
}

/// Non-local exits threaded through `?`.
enum Ctl {
    Err(Error),
    Return(Value),
}

impl From<Error> for Ctl {
    fn from(e: Error) -> Ctl {
        Ctl::Err(e)
    }
}

type EResult = Result<Value, Ctl>;

pub struct Interp {
    globals: Env,
    input: Option<Rc<str>>,
}

impl Interp {
    pub fn new() -> Interp {
        let globals: Env = Default::default();
        {
            let mut g = globals.borrow_mut();
            for name in crate::builtins::NAMES {
                g.vars.insert(name.to_string(), Value::Builtin(name));
            }
            g.vars.insert("pi".into(), Value::Float(std::f64::consts::PI));
            g.vars.insert("e".into(), Value::Float(std::f64::consts::E));
        }
        Interp { globals, input: None }
    }

    pub fn set_global(&mut self, name: &str, v: Value) {
        self.globals.borrow_mut().vars.insert(name.into(), v);
    }

    pub fn run(&mut self, prog: &[Expr]) -> Result<Value, Error> {
        let globals = self.globals.clone();
        let mut last = Value::Nil;
        for stmt in prog {
            last = match self.eval(stmt, &globals) {
                Ok(v) | Err(Ctl::Return(v)) => v,
                Err(Ctl::Err(e)) => return Err(e),
            };
        }
        Ok(last)
    }

    pub fn call(&mut self, f: &Value, args: Vec<Value>, span: &Span) -> Result<Value, Error> {
        match f {
            Value::Fn(c) => {
                if args.len() != c.def.params.len() {
                    return Err(Error::new(
                        format!("expected {} args, got {}", c.def.params.len(), args.len()),
                        span.clone(),
                    ));
                }
                let env = child(&c.env);
                for (p, a) in c.def.params.iter().zip(args) {
                    env.borrow_mut().vars.insert(p.clone(), a);
                }
                match self.eval(&c.def.body, &env) {
                    Ok(v) | Err(Ctl::Return(v)) => Ok(v),
                    Err(Ctl::Err(e)) => Err(e),
                }
            }
            Value::Builtin(name) => crate::builtins::call(self, name, args, span),
            v => Err(Error::new(format!("{} is not callable", v.type_name()), span.clone())),
        }
    }

    /// Special names that aren't variables: `now`, `today`, `input`, then units (`d * km`).
    fn ident_fallback(&mut self, name: &str) -> Result<Value, String> {
        match name {
            "now" => Ok(Value::date(Zoned::now())),
            "today" | "tomorrow" | "yesterday" => Ok(Value::date(crate::dates::parse(name, &Zoned::now()).unwrap())),
            "input" => {
                if self.input.is_none() {
                    let mut s = String::new();
                    std::io::Read::read_to_string(&mut std::io::stdin(), &mut s).map_err(|e| e.to_string())?;
                    self.input = Some(s.into());
                }
                Ok(Value::Str(self.input.clone().unwrap()))
            }
            _ => match units::unit(name) {
                Ok(u) => Ok(Value::Qty(1.0, u)),
                Err(e) if e.starts_with("unknown unit") => Err(format!("undefined variable `{name}`")),
                Err(e) => Err(e),
            },
        }
    }

    fn eval(&mut self, e: &Expr, env: &Env) -> EResult {
        let err = |msg: String| Ctl::Err(Error::new(msg, e.span.clone()));
        Ok(match &e.kind {
            ExprKind::Nil => Value::Nil,
            ExprKind::Bool(b) => Value::Bool(*b),
            ExprKind::Int(n, b) => Value::Int(*n, Radix { base: *b, width: 0 }),
            ExprKind::Float(n) => Value::Float(*n),
            ExprKind::Str(s) => Value::Str(s.clone()),
            ExprKind::Regex(r) => Value::Regex(r.clone()),
            ExprKind::Ident(name) => match lookup(env, name) {
                Some(v) => v,
                None => self.ident_fallback(name).map_err(err)?,
            },
            ExprKind::List(items) => {
                let mut v = Vec::with_capacity(items.len());
                for it in items {
                    v.push(self.eval(it, env)?);
                }
                Value::list(v)
            }
            ExprKind::Map(entries) => {
                let mut m = IndexMap::new();
                for (k, v) in entries {
                    m.insert(k.clone(), self.eval(v, env)?);
                }
                Value::map(m)
            }
            ExprKind::Qty(n, spec) => {
                let v = self.eval(n, env)?;
                let n = num(&v).ok_or_else(|| err(format!("cannot attach a unit to a {}", v.type_name())))?;
                Value::Qty(n, unit_of(spec).map_err(err)?)
            }
            ExprKind::To(v, Target::Type(t)) => {
                let v = self.eval(v, env)?;
                crate::builtins::call(self, t, vec![v], &e.span)?
            }
            ExprKind::To(v, target) => {
                let v = self.eval(v, env)?;
                convert(v, target).map_err(err)?
            }
            ExprKind::Unary(op, x) => match (op, self.eval(x, env)?) {
                (UnOp::Not, v) => Value::Bool(!v.truthy()),
                (UnOp::Neg, Value::Int(n, b)) => Value::Int(n.checked_neg().ok_or_else(|| err("integer overflow".into()))?, b),
                (UnOp::Neg, Value::Float(n)) => Value::Float(-n),
                (UnOp::Neg, Value::Qty(n, u)) => Value::Qty(-n, u),
                (UnOp::Neg, v) => return Err(err(format!("cannot negate {}", v.type_name()))),
                (UnOp::BitNot, Value::Int(n, b)) => Value::Int(!n, b),
                (UnOp::BitNot, v) => return Err(err(format!("cannot bit-invert {}", v.type_name()))),
            },
            ExprKind::Binary(BinOp::And, a, b) => {
                let a = self.eval(a, env)?;
                if a.truthy() { self.eval(b, env)? } else { a }
            }
            ExprKind::Binary(BinOp::Or, a, b) => {
                let a = self.eval(a, env)?;
                if a.truthy() { a } else { self.eval(b, env)? }
            }
            ExprKind::Binary(op, a, b) => {
                let (a, b) = (self.eval(a, env)?, self.eval(b, env)?);
                binary(*op, &a, &b).map_err(err)?
            }
            ExprKind::Assign(target, v) => {
                let v = self.eval(v, env)?;
                match &target.kind {
                    ExprKind::Ident(name) => {
                        if let Err(v) = assign(env, name, v.clone()) {
                            env.borrow_mut().vars.insert(name.clone(), v);
                        }
                    }
                    ExprKind::Field(obj, key) => match self.eval(obj, env)? {
                        Value::Map(m) => {
                            m.borrow_mut().insert(key.clone(), v.clone());
                        }
                        o => return Err(err(format!("cannot set field on {}", o.type_name()))),
                    },
                    ExprKind::Index(obj, idx) => {
                        let (obj, idx) = (self.eval(obj, env)?, self.eval(idx, env)?);
                        match (obj, idx) {
                            (Value::List(l), Value::Int(i, _)) => {
                                let mut l = l.borrow_mut();
                                let i = list_index(i, l.len()).ok_or_else(|| err("index out of range".into()))?;
                                l[i] = v.clone();
                            }
                            (Value::Map(m), Value::Str(k)) => {
                                m.borrow_mut().insert(k.to_string(), v.clone());
                            }
                            (o, i) => return Err(err(format!("cannot index {} with {}", o.type_name(), i.type_name()))),
                        }
                    }
                    _ => unreachable!("parser validates assignment targets"),
                }
                v
            }
            ExprKind::Call(callee, args) => {
                // `x.f(a)`: a map's own fn field wins, else sugar for `f(x, a)`.
                let (f, mut argv) = match &callee.kind {
                    ExprKind::Field(obj, name) => {
                        let o = self.eval(obj, env)?;
                        let own = match &o {
                            Value::Map(m) => m.borrow().get(name).cloned(),
                            _ => None,
                        };
                        match own {
                            Some(f) => (f, vec![]),
                            None => (lookup(env, name).ok_or_else(|| err(format!("no method `{name}`")))?, vec![o]),
                        }
                    }
                    _ => (self.eval(callee, env)?, vec![]),
                };
                for a in args {
                    argv.push(self.eval(a, env)?);
                }
                self.call(&f, argv, &e.span)?
            }
            // `x.k`: map key, else zero-arg method (`s.upper` == `upper(s)`), else nil for maps.
            ExprKind::Field(obj, key) => {
                let o = self.eval(obj, env)?;
                if let Value::Map(m) = &o
                    && let Some(v) = m.borrow().get(key)
                {
                    return Ok(v.clone());
                }
                match lookup(env, key) {
                    Some(f @ (Value::Fn(_) | Value::Builtin(_))) => self.call(&f, vec![o], &e.span)?,
                    _ if matches!(o, Value::Map(_)) => Value::Nil,
                    _ => return Err(err(format!("{} has no field or method `{key}`", o.type_name()))),
                }
            }
            ExprKind::Index(obj, idx) => match (self.eval(obj, env)?, self.eval(idx, env)?) {
                (Value::List(l), Value::Int(i, _)) => {
                    let l = l.borrow();
                    let i = list_index(i, l.len()).ok_or_else(|| err("index out of range".into()))?;
                    l[i].clone()
                }
                (Value::Str(s), Value::Int(i, _)) => {
                    let i = list_index(i, s.chars().count()).ok_or_else(|| err("index out of range".into()))?;
                    Value::str(s.chars().nth(i).unwrap().to_string())
                }
                (Value::Map(m), Value::Str(k)) => m.borrow().get(&*k).cloned().unwrap_or(Value::Nil),
                (o, i) => return Err(err(format!("cannot index {} with {}", o.type_name(), i.type_name()))),
            },
            ExprKind::Slice(obj, from, to) => {
                let o = self.eval(obj, env)?;
                let mut bound = |b: &Option<Box<Expr>>| -> Result<Option<i64>, Ctl> {
                    match b {
                        None => Ok(None),
                        Some(b) => match self.eval(b, env)? {
                            Value::Int(n, _) => Ok(Some(n)),
                            v => Err(err(format!("slice bounds must be int, got {}", v.type_name()))),
                        },
                    }
                };
                let (from, to) = (bound(from)?, bound(to)?);
                match o {
                    Value::List(l) => {
                        let l = l.borrow();
                        let (a, b) = slice_range(from, to, l.len());
                        Value::list(l[a..b].to_vec())
                    }
                    Value::Str(s) => {
                        let (a, b) = slice_range(from, to, s.chars().count());
                        Value::str(s.chars().skip(a).take(b - a).collect::<String>())
                    }
                    o => return Err(err(format!("cannot slice {}", o.type_name()))),
                }
            }
            ExprKind::Fn(def) => Value::Fn(Rc::new(Closure { def: def.clone(), env: env.clone() })),
            ExprKind::If(cond, then, els) => {
                if self.eval(cond, env)?.truthy() {
                    self.eval(then, env)?
                } else if let Some(els) = els {
                    self.eval(els, env)?
                } else {
                    Value::Nil
                }
            }
            ExprKind::While(cond, body) => {
                while self.eval(cond, env)?.truthy() {
                    self.eval(body, env)?;
                }
                Value::Nil
            }
            ExprKind::For(name, iter, body) => {
                let items: Vec<Value> = match self.eval(iter, env)? {
                    Value::List(l) => l.borrow().clone(),
                    Value::Map(m) => m.borrow().keys().map(|k| Value::str(k.as_str())).collect(),
                    Value::Str(s) => s.chars().map(|c| Value::str(c.to_string())).collect(),
                    v => return Err(err(format!("cannot iterate {}", v.type_name()))),
                };
                for it in items {
                    let scope = child(env);
                    scope.borrow_mut().vars.insert(name.clone(), it);
                    self.eval(body, &scope)?;
                }
                Value::Nil
            }
            ExprKind::Block(stmts) => {
                let scope = child(env);
                let mut last = Value::Nil;
                for s in stmts {
                    last = self.eval(s, &scope)?;
                }
                last
            }
            ExprKind::Return(v) => {
                let v = match v {
                    Some(v) => self.eval(v, env)?,
                    None => Value::Nil,
                };
                return Err(Ctl::Return(v));
            }
        })
    }
}

/// Negative indices count from the end.
fn list_index(i: i64, len: usize) -> Option<usize> {
    let i = if i < 0 { len as i64 + i } else { i };
    (0..len as i64).contains(&i).then_some(i as usize)
}

/// Clamped `[from..to)`, negatives counting from the end.
fn slice_range(from: Option<i64>, to: Option<i64>, len: usize) -> (usize, usize) {
    let fix = |i: i64| (if i < 0 { len as i64 + i } else { i }).clamp(0, len as i64) as usize;
    let a = from.map_or(0, fix);
    let b = to.map_or(len, fix);
    (a, b.max(a))
}

pub fn range(a: i64, b: i64) -> Result<Value, String> {
    if b.saturating_sub(a) > 10_000_000 {
        return Err("range too large".into());
    }
    Ok(Value::list((a..b).map(Value::int).collect()))
}

pub fn convert(v: Value, target: &Target) -> Result<Value, String> {
    Ok(match (v, target) {
        (Value::Qty(x, u), Target::Units(specs)) => {
            let us = specs.iter().map(unit_of).collect::<Result<Vec<_>, _>>()?;
            if let Some(t) = us.iter().find(|t| t.dim() != u.dim()) {
                return Err(format!("cannot convert {u} to {t}"));
            }
            Value::str(units::split(u.to_si(x), &us))
        }
        (Value::Qty(x, u), Target::Unit(spec)) => {
            let t = unit_of(spec)?;
            if u.dim() != t.dim() {
                return Err(format!("cannot convert {u} to {t}"));
            }
            Value::Qty(t.value_from_si(u.to_si(x)), t)
        }
        (Value::Int(n, _), Target::Base(r)) if r.width == 0 || fits(n, r.width) => Value::Int(n, *r),
        (Value::Int(n, _), Target::Base(r)) => return Err(format!("{n} does not fit in {} bits", r.width)),
        (Value::Float(x), Target::Base(_)) if x.fract() == 0.0 && x.abs() < 9.2e18 => convert(Value::int(x as i64), target)?,
        (Value::Str(s), Target::Base(Radix { base: 16, width: 0 })) => Value::str(hex(s.as_bytes())),
        (Value::Date(z), Target::Unix) => Value::int(z.timestamp().as_second()),
        (Value::Date(z), Target::Tz(name)) if name == "local" => Value::date(z.with_time_zone(TimeZone::system())),
        (Value::Date(z), Target::Tz(name)) => Value::date(z.in_tz(name).map_err(|e| e.to_string())?),
        (v @ (Value::Int(..) | Value::Float(_)), Target::Unit(_) | Target::Units(_)) => {
            return Err(format!("{v} has no unit; attach one like `{v} km`"));
        }
        (v, _) => return Err(format!("cannot convert {} like that", v.type_name())),
    })
}


pub fn binary(op: BinOp, a: &Value, b: &Value) -> Result<Value, String> {
    use Value::*;
    let mismatch = || format!("unsupported operands {} and {}", a.type_name(), b.type_name());
    match op {
        BinOp::Eq => return Ok(Bool(a == b)),
        BinOp::Ne => return Ok(Bool(a != b)),
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let ord = compare(a, b).ok_or_else(|| format!("cannot compare {a:?} and {b:?}"))?;
            return Ok(Bool(match op {
                BinOp::Lt => ord.is_lt(),
                BinOp::Le => ord.is_le(),
                BinOp::Gt => ord.is_gt(),
                _ => ord.is_ge(),
            }));
        }
        _ => {}
    }
    Ok(match (op, a, b) {
        (BinOp::Range, Int(x, _), Int(y, _)) => range(*x, *y)?,
        (BinOp::Add, Str(_), _) | (BinOp::Add, _, Str(_)) => Value::str(format!("{a}{b}")),
        (BinOp::Mul, Str(s), Int(n, _)) | (BinOp::Mul, Int(n, _), Str(s)) => Value::str(s.repeat((*n).max(0) as usize)),
        (BinOp::Add, List(x), List(y)) => Value::list(x.borrow().iter().chain(y.borrow().iter()).cloned().collect()),

        (BinOp::Add, Date(z), Qty(v, u)) | (BinOp::Add, Qty(v, u), Date(z)) => Value::date(crate::dates::add(z, *v, u)?),
        (BinOp::Sub, Date(z), Qty(v, u)) => Value::date(crate::dates::add(z, -v, u)?),
        (BinOp::Sub, Date(x), Date(y)) => Qty(x.duration_since(y).as_secs_f64() / 86400.0, units::unit("d")?),

        (BinOp::Add | BinOp::Sub, Qty(x, u), Qty(y, w)) => {
            if u.dim() != w.dim() {
                return Err(format!("cannot add {u} and {w}"));
            }
            let y = u.value_from_si(w.to_si(*y));
            Value::qty(if op == BinOp::Add { x + y } else { x - y }, u.clone())
        }
        (BinOp::Mul | BinOp::Div, Qty(..), _) | (BinOp::Mul | BinOp::Div, _, Qty(..)) => {
            let split = |v: &Value| match v {
                Qty(x, u) => Some((*x, u.clone())),
                _ => Some((num(v)?, Unit::default())),
            };
            let ((x, u), (y, w)) = (split(a).ok_or_else(mismatch)?, split(b).ok_or_else(mismatch)?);
            if op == BinOp::Mul { Value::qty(x * y, u.mul(&w, 1)) } else { Value::qty(x / y, u.mul(&w, -1)) }
        }
        (BinOp::Pow, Qty(x, u), Int(n, _)) if (-9..=9).contains(n) => Value::qty(x.powi(*n as i32), u.pow(*n as i8)),

        (_, Int(x, bx), Int(y, by)) => {
            let base = if *bx != Radix::DEC { *bx } else { *by };
            let (x, y) = (*x, *y);
            let zero = || "division by zero".to_string();
            let r = match op {
                BinOp::Add => x.checked_add(y),
                BinOp::Sub => x.checked_sub(y),
                BinOp::Mul => x.checked_mul(y),
                BinOp::Div if y == 0 => return Err(zero()),
                BinOp::Div if x % y != 0 => return Ok(Float(x as f64 / y as f64)),
                BinOp::Div => x.checked_div(y),
                BinOp::IntDiv if y == 0 => return Err(zero()),
                BinOp::IntDiv => x.checked_div(y).map(|q| if x % y != 0 && (x < 0) != (y < 0) { q - 1 } else { q }),
                BinOp::Rem if y == 0 => return Err(zero()),
                BinOp::Rem => x.checked_rem_euclid(y),
                BinOp::Pow if y < 0 => return Ok(Float((x as f64).powf(y as f64))),
                BinOp::Pow => u32::try_from(y).ok().and_then(|y| x.checked_pow(y)),
                BinOp::BitAnd => Some(x & y),
                BinOp::BitOr => Some(x | y),
                BinOp::BitXor => Some(x ^ y),
                // ponytail: `<<` drops high bits like C instead of erroring on overflow.
                BinOp::Shl | BinOp::Shr if !(0..64).contains(&y) => return Err("shift must be 0-63".into()),
                BinOp::Shl => Some(x << y),
                BinOp::Shr => Some(x >> y),
                _ => return Err(mismatch()),
            };
            Int(r.ok_or("integer overflow")?, base)
        }
        _ => match (num(a), num(b)) {
            (Some(x), Some(y)) => Float(match op {
                BinOp::Add => x + y,
                BinOp::Sub => x - y,
                BinOp::Mul => x * y,
                BinOp::Div => x / y,
                BinOp::IntDiv => (x / y).floor(),
                BinOp::Rem => x.rem_euclid(y),
                BinOp::Pow => x.powf(y),
                _ => return Err(mismatch()),
            }),
            _ => return Err(mismatch()),
        },
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn eval(src: &str) -> Value {
        try_eval(src).unwrap_or_else(|e| panic!("{src}: {}", e.msg))
    }

    pub fn try_eval(src: &str) -> Result<Value, Error> {
        let ast = crate::parser::parse(crate::lexer::lex(src)?)?;
        Interp::new().run(&ast)
    }

    fn show(src: &str) -> String {
        eval(src).to_string()
    }

    #[test]
    fn arithmetic() {
        assert_eq!(show("1 + 2 * 3"), "7");
        assert_eq!(show("1 / 3"), "0.333333");
        assert_eq!(show("6 / 3"), "2");
        assert_eq!(show("-7 // 2"), "-4");
        assert_eq!(show("-7 % 3"), "2");
        assert_eq!(show("2 ** 10"), "1024");
        assert_eq!(show("-2 ** 2"), "-4");
        assert_eq!(show("2 ** 3 ** 2"), "512");
        assert_eq!(show("0.1 + 0.2"), "0.3");
        assert_eq!(show("1e20"), "1e20");
        assert_eq!(show(r#""ab" * 3"#), "ababab");
        assert!(try_eval("1 / 0").is_err());
    }

    #[test]
    fn units() {
        assert_eq!(show("5 km to mi"), "3.10686 mi");
        assert_eq!(show("72 F to C"), "22.2222 C");
        assert_eq!(show("-40 C to F"), "-40 F");
        assert_eq!(show("3 km / 20 min to kph"), "9 kph");
        assert_eq!(show("1.5 GB to MiB"), "1430.51 MiB");
        assert_eq!(show("2 m + 30 cm"), "2.3 m");
        assert_eq!(show("60 km/h to m/s"), "16.6667 m/s");
        assert_eq!(show("2 m * 3 m"), "6 m^2");
        assert_eq!(show("1 kWh / 1 J"), "3600000");
        assert_eq!(show("d = 5\nd * km to mi"), "3.10686 mi");
        assert_eq!(show("m = 3\n5 m to ft"), "16.4042 ft");
        assert_eq!(show("1 m > 50 cm"), "true");
        assert!(try_eval("5 km + 1 kg").is_err());
        assert!(try_eval("5 km to kg").is_err());
    }

    #[test]
    fn bases() {
        assert_eq!(show("255 to hex"), "0xff");
        assert_eq!(show("x = 255 to hex\nx + 1"), "0x100");
        assert_eq!(show("0xff to dec"), "255");
        assert_eq!(show("0b1010 + 1"), "0b1011");
        assert_eq!(show("10 to bin"), "0b1010");
        assert_eq!(show("35 to base(36)"), "36#z");
        assert_eq!(show("x = 36#z
x + 1"), "36#10");
        assert_eq!(show("x = 0b1010
x + 1"), "0b1011");
        assert_eq!(show("2#1010"), "0b1010");
        assert_eq!(show("-3#12 to dec"), "-5");
        assert!(try_eval("37#1").is_err());
        assert!(try_eval("2#12").is_err());
        assert_eq!(show("-1 to hex(32)"), "0xffffffff");
        assert_eq!(show("5 to bin(8)"), "0b00000101");
        assert_eq!(show("-1 to dec(8)"), "255");
        assert_eq!(show("~0x0f to hex(8)"), "0xf0");
        assert_eq!(show("-5 to hex"), "-0x5");
        assert!(try_eval("256 to hex(8)").is_err());
        assert_eq!(show(r#""hi" to hex"#), "6869");
    }

    #[test]
    fn type_conversions() {
        assert_eq!(show(r#""ab" to list"#), r#"["a", "b"]"#);
        assert_eq!(show(r#""0xff" to int"#), "0xff");
        assert_eq!(show(r#""2.5" to float * 2"#), "5");
        assert_eq!(show("[1, 2] to str"), "[1, 2]");
        assert_eq!(show("0 to bool"), "true");
        assert_eq!(show("nil to bool"), "false");
        assert_eq!(show("5 km to float"), "5");
        assert!(try_eval("5 to list").is_err());
        assert_eq!(show(r#""hi there" to base64"#), "aGkgdGhlcmU=");
        assert_eq!(show(r#"base64("hi there")"#), "aGkgdGhlcmU=");
        assert_eq!(show("hex(255)"), "0xff");
        assert_eq!(show("255.bin"), "0b11111111");
        assert_eq!(show("bin(5, 8)"), "0b00000101");
        assert_eq!(show("oct(8)"), "0o10");
        assert_eq!(show("dec(0xff)"), "255");
        assert_eq!(show("base(35, 36)"), "36#z");
        assert_eq!(show(r#"hex("hi")"#), "6869");
        assert_eq!(show("hex = 3
hex + 1"), "4");
        assert!(try_eval("hex(256, 8)").is_err());
        assert!(try_eval("base(1, 99)").is_err());
    }

    #[test]
    fn bitwise() {
        assert_eq!(show("0b1100 & 0b1010 to bin"), "0b1000");
        assert_eq!(show("0b1100 | 0b1010 to bin"), "0b1110");
        assert_eq!(show("0b1100 ^ 0b1010 to bin"), "0b110");
        assert_eq!(show("1 << 4"), "16");
        assert_eq!(show("-16 >> 2"), "-4");
        assert_eq!(show("~0"), "-1");
        assert_eq!(show("x = 0xff00 to hex
x & 0x0ff0"), "0xf00");
        assert_eq!(show("1 | 2 == 3"), "true");
        assert_eq!(show("0xf0 >> 4 + 0"), "0xf");
        assert!(try_eval("1 << 64").is_err());
        assert!(try_eval("1.5 & 1").is_err());
    }

    #[test]
    fn strings() {
        assert_eq!(show(r#"x = 2
"x={x}, sq={x ** 2}""#), "x=2, sq=4");
        assert_eq!(show(r#""hello"[1..4]"#), "ell");
        assert_eq!(show(r#""hello"[-3..]"#), "llo");
        assert_eq!(show(r#""hello".upper"#), "HELLO");
        assert_eq!(show(r#"" hi ".trim.reverse"#), "ih");
        assert_eq!(show(r#""a,b".split(",").len"#), "2");
    }

    #[test]
    fn variables_and_closures() {
        assert_eq!(show("x = 1\nx = x + 1\nx"), "2");
        let src = "
make = fn(n) { \\x -> x + n }
add2 = make(2)
add2(40)";
        assert_eq!(show(src), "42");
        let src = "
total = 0
for x in 1..5 { total = total + x }
total";
        assert_eq!(show(src), "10");
    }

    #[test]
    fn recursion_and_return() {
        let src = "
fib = fn(n) {
  if n < 2 { return n }
  fib(n - 1) + fib(n - 2)
}
fib(10)";
        assert_eq!(show(src), "55");
    }

    #[test]
    fn records_and_mutation() {
        let src = r#"
u = {name: "ann", tags: []}
u.age = 30
u.tags.push("x")
u.tags[0] = "y"
[u.age, u.tags, u.missing]"#;
        assert_eq!(show(src), r#"[30, ["y"], nil]"#);
    }

    #[test]
    fn dates() {
        assert_eq!(show(r#"date("2026-01-31") + 1 mo"#)[..10], *"2026-02-28");
        assert_eq!(show(r#"date("2026-12-25") - date("2026-12-20")"#), "5 d");
        assert_eq!(show(r#"date("2026-03-01T12:00") + 90 min"#)[..16], *"2026-03-01 13:30");
        assert_eq!(show(r#"date(0) to "UTC""#), "1970-01-01 00:00:00 +00:00");
        assert_eq!(show(r#"date(0) to unix"#), "0");
        assert_eq!(show(r#"(date(0) to UTC).year"#), "1970");
        assert_eq!(show(r#"date("2026-12-25")"#), "2026-12-25");
        assert_eq!(show("date(2026, 12, 25, 18, 30)")[..16], *"2026-12-25 18:30");
        assert_eq!(show("tomorrow - today"), "1 d");
        assert_eq!(show("today == now.start_of(\"day\")"), "true");
        assert_eq!(show(r#"date("2026-11-01").nth_weekday(4, "thursday")"#), "2026-11-26");
        assert_eq!(show(r#"date("2026-10-06").next("fri").weekday"#), "Friday");
        assert_eq!(show(r#"(date("2026-12-25T00:00Z") - date("2026-10-06T14:24Z")) to d h min"#), "79 d 9 h 36 min");
        assert_eq!(show("5.5 ft to ft in"), "5 ft 6 in");
        assert_eq!(show("-90 s to min s"), "-1 min 30 s");
        assert_eq!(show("5000 s.parts"), "1 h 23 min 20 s");
        assert_eq!(show("0.5 d.parts"), "12 h");
        assert!(try_eval("5 km to h min").is_err());
        assert!(try_eval(r#"now.start_of("fortnight")"#).is_err());
    }
}
