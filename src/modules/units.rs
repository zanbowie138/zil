//! Units: quantities with dimensions, arithmetic and conversion between them, live currency rates.

pub mod constants;
pub mod goofy;
pub mod kitchen;
pub mod money;

use crate::ast::{BinOp, Target, UnitSpec};
use crate::interp::Interp;
use crate::interp::mismatch;
use crate::lexer::Span;
use crate::modules::{Call, Claim, Doc, Fail, Module, doc};
use crate::value::{Table, Value, num};
use indexmap::IndexMap;
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt;
use std::rc::Rc;

pub const MODULE: Module = Module {
    name: "units",
    about: "numbers with units, combined and converted; currencies use live rates",
    #[rustfmt::skip]
    examples: &[
        ("units", &[
            ("speed from distance and time", "100 km / 2 h to mph"),
            ("feet and inches", "1.8 m to ft in"),
            ("mixed units add", "5 ft 11 in to cm"),
            ("drive time", "300 mi / 65 mph to h min"),
            ("download time", "4 GB / 100 Mbps to min"),
            ("why a 1 TB disk shows less", "1 TB to GiB"),
            ("area", "3 m * 4 m to ft^2"),
            ("running pace", "1 h / 10 km to min/mi"),
            ("force", "9.81 m/s^2 * 70 kg to N"),
            ("a month of a 60 W bulb", "60 W * 8 h * 30 to kWh"),
            ("temperature", "101 F to C"),
            ("20 USD to EUR   (currencies use live rates)", ""),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[
        ("types", &[("quantity", "5 km")]),
        ("operators", &[
            ("qty ± qty", "1 km + 300 m"),
            ("qty * / qty", "100 km / 2 h"),
            ("qty ** int", "(3 m) ** 2"),
            ("qty < qty", "1 mi > 1 km"),
        ]),
        ("pretty", &[
            ("long: best prefix, thousands separators", "pretty(12345678 m)"),
            ("short: best prefix, K M B T", r#"pretty(12345678 m, "short")"#),
            ("durations split into d h min s, money keeps its own display: see time, money", ""),
        ]),
        ("conversions", &[("to unit", "5 km to mi"), ("in unit", "5 km in mi"), ("to unit unit", "1.8 m to ft in"), ("to compound", "100 km / 2 h to mph")]),
        // Shown, not run: `unit` would define them in the session running help.
        ("your own units", &[
            ("unit sprint = 2 wk        then  3 sprint to d → 42 d", ""),
            ("unit pizza                a new base unit, for counting", ""),
            ("unit slice = pizza / 8    then  3 slice to pizza → 0.375 pizza", ""),
        ]),
    ],
    fns: FNS,
    call,
    targets: &[("best", "simplify")],
    ident: Some(|_, name| match unit(name) {
        Ok(u) => Some(Ok(Value::Qty(1.0, u))),
        Err(e) if e.starts_with("unknown unit") => None,
        Err(e) => Some(Err(e)),
    }),
    binary: Some(binary),
    compare: Some(compare),
    convert: Some(convert),
    topic: Some(topic),
    units: TABLE,
    children: &[constants::MODULE, money::MODULE, goofy::MODULE, kitchen::MODULE],
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("simplify", "simplify(q: quantity)", "the quantity in the prefixed unit of the same family that reads best, like `1.2 m` or `3.6 kW`; also `to best`",
        &["0.0012 km to best", "3600 J/s to best", "1 kg*m^2/s^2 to best", "1.5e9 B.simplify"], &[]),
    doc("all_units", "all_units()", "every unit as a table: {name, full, desc, aliases, kind, si, source}; source is units, goofy, kitchen, currency or user",
        &["all_units().len", r#"all_units().filter(|u| u.kind == "length").map(|u| u.name).take(5)"#, r#"all_units().filter(|u| u.source == "goofy")[0]"#], &["simplify"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    match (name, args) {
        ("simplify", [Value::Qty(x, u)]) => Ok(simplify(*x, u)),
        ("all_units", []) => Ok(all_units()),
        _ => Err(Fail::BadArgs),
    }
}

/// `x u` in the table unit whose scale is a power of ten times `u`'s (so bytes stay bytes, feet stay feet)
/// and gives the biggest value of at least 1. Temperatures, money and units with no such family stay as they are.
pub fn simplify(x: f64, u: &Unit) -> Value {
    let dim = u.dim();
    if u.offset() != 0.0 || dim[8] != 0 || x == 0.0 {
        return Value::Qty(x, u.clone());
    }
    let si = u.to_si(x).abs();
    let family = |row: &&Row| {
        let r = (row.1 / u.scale()).log10();
        row.3 == dim && row.2 == 0.0 && (r - r.round()).abs() < 1e-9
    };
    let rows: Vec<&Row> = TABLE.iter().filter(family).collect();
    let best = rows.iter().filter(|r| r.1 <= si * (1.0 + 1e-12)).max_by(|a, b| a.1.total_cmp(&b.1)).or_else(|| rows.iter().min_by(|a, b| a.1.total_cmp(&b.1)));
    match best {
        // Already in a unit of that size (`5 cd` stays candelas, not lumens).
        Some(row) if (row.1 / u.scale() - 1.0).abs() > 1e-12 || u.0.len() > 1 => {
            let t = unit(row.0.split(' ').next().unwrap()).expect("table unit");
            Value::Qty(u.to_si(x) / t.scale(), t)
        }
        _ => Value::Qty(x, u.clone()),
    }
}

fn binary(op: BinOp, a: &Value, b: &Value) -> Claim {
    use Value::*;
    Some(Ok(match (op, a, b) {
        (BinOp::Add | BinOp::Sub, Qty(x, u), Qty(y, w)) => {
            if u.dim() != w.dim() {
                return Some(Err(mismatch(op, a, b)));
            }
            let y = match convert_value(*y, w, u) {
                Ok(y) => y,
                Err(e) => return Some(Err(e)),
            };
            Value::qty(if op == BinOp::Add { x + y } else { x - y }, u.clone())
        }
        (BinOp::Mul | BinOp::Div, Qty(..), _) | (BinOp::Mul | BinOp::Div, _, Qty(..)) => {
            let split = |v: &Value| match v {
                Qty(x, u) => Some((*x, u.clone())),
                _ => Some((num(v)?, Unit::default())),
            };
            let (Some((x, u)), Some((y, w))) = (split(a), split(b)) else {
                return Some(Err(mismatch(op, a, b)));
            };
            let u = u.mul(&w, if op == BinOp::Mul { 1 } else { -1 });
            // USD/EUR is a plain number, but only at today's rate.
            if !u.0.is_empty() && u.dim() == NONE && u.scale().is_nan() {
                return Some(Err(rate_error()));
            }
            let (v, u) = named(if op == BinOp::Mul { x * y } else { x / y }, u);
            Value::qty(v, u)
        }
        (BinOp::Pow, Qty(x, u), Int(n, _)) if (-9..=9).contains(n) => Value::qty(x.powi(*n as i32), u.pow(*n as i8)),
        _ => return None,
    }))
}

/// A product of metric units as its named SI unit: `mA*kΩ` is `V`, `kg*m/s^2` is `N`. Anything with
/// non-metric terms (`W*h`), money, data or angle keeps the units as typed.
fn named(x: f64, u: Unit) -> (f64, Unit) {
    // ponytail: "metric" = power-of-ten factor to SI; fine for TABLE, revisit if a unit like 1e3 B joins it.
    let metric = |d: &UnitDef| d.offset == 0.0 && (d.scale().log10() - d.scale().log10().round()).abs() < 1e-9;
    let dim = u.dim();
    if matches!(u.0.as_slice(), [] | [(_, 1)]) || dim[7..].iter().any(|x| *x != 0) || !u.0.iter().all(|(d, _)| metric(d)) {
        return (x, u);
    }
    match TABLE.iter().find(|row| row.1 == 1.0 && row.2 == 0.0 && row.3 == dim) {
        Some(row) => (x * u.scale(), unit(row.0.split(' ').next().unwrap()).unwrap()),
        None => (x, u),
    }
}

fn compare(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::Qty(x, u), Value::Qty(y, w)) if u.dim() == w.dim() => x.partial_cmp(&convert_value(*y, w, u).ok()?),
        _ => None,
    }
}

/// `to km`, and `to ft in` as a mixed-unit string.
fn convert(v: &Value, t: &Target) -> Claim {
    let r = match (v, t) {
        (Value::Qty(x, u), Target::Units(specs)) => {
            specs.iter().map(unit_of).collect::<Result<Vec<_>, _>>().and_then(|us| match us.iter().find(|t| t.dim() != u.dim()) {
                Some(t) => Err(format!("cannot convert {} to {}", describe(u, None), describe(t, None))),
                None => Ok(Value::str(split(u.to_si(*x), &us))),
            })
        }
        (Value::Qty(x, u), Target::Unit(spec)) => unit_of(spec).and_then(|t| to_unit(*x, u, t)),
        (v, Target::Unit(_) | Target::Units(_)) if num(v).is_some() => {
            if let Err(e) = match t {
                Target::Unit(spec) => unit_of(spec).map(drop),
                Target::Units(specs) => specs.iter().try_for_each(|s| unit_of(s).map(drop)),
                _ => Ok(()),
            } {
                return Some(Err(e));
            }
            let to = match t {
                Target::Unit(spec) => spec_name(spec),
                Target::Units(specs) => specs.iter().map(spec_name).collect::<Vec<_>>().join(" "),
                _ => unreachable!(),
            };
            Err(format!("cannot convert a plain number to `{to}`\nhelp: attach a unit first, like `{v} km to {to}`"))
        }
        _ => return None,
    };
    Some(r)
}

/// `length (km)`, or just `` `km*kg` `` for a kind with no name.
pub fn describe(u: &Unit, written: Option<&str>) -> String {
    let name = written.map_or_else(|| u.to_string(), str::to_string);
    match DIMS.iter().find(|d| d.1 == u.dim()) {
        Some((kind, _)) => format!("{kind} ({name})"),
        None => format!("`{name}`"),
    }
}

/// A unit spec as written: `km/h`, `m^2`.
pub fn spec_name(spec: &UnitSpec) -> String {
    product(spec.iter().map(|(n, p)| (n.as_str(), *p)))
}

/// `a*b/c^2` from (name, power) terms.
fn product<'a>(terms: impl Iterator<Item = (&'a str, i8)>) -> String {
    let (num, den): (Vec<_>, Vec<_>) = terms.partition(|t| t.1 > 0);
    let term = |(n, p): &(&str, i8)| if p.abs() == 1 { n.to_string() } else { format!("{n}^{}", p.abs()) };
    let (num, den): (Vec<_>, Vec<_>) = (num.iter().map(term).collect(), den.iter().map(term).collect());
    match (num.is_empty(), den.is_empty()) {
        (_, true) => num.join("*"),
        (true, false) => format!("1/{}", den.join("/")),
        _ => format!("{}/{}", num.join("*"), den.join("/")),
    }
}

pub fn unit_of(spec: &UnitSpec) -> Result<Unit, String> {
    let mut u = Unit::default();
    for (name, p) in spec {
        u = u.mul(&unit(name)?.pow(*p), 1);
    }
    Ok(u)
}

/// `help("units")`, `help("length")`, `help("km")`, `help("currency")`.
fn topic(topic: &str) -> bool {
    use crate::DIM;
    use crate::help::{HEADING, NAME, code, eval, names, out, pad, paint, show};
    if topic == "units" {
        let w = DIMS.iter().map(|d| d.0.len()).max().unwrap_or(0);
        out!("  {}{}  3-letter codes (USD EUR GBP ...), live rates fetched when converting", paint(DIM, "currency"), pad("currency", w));
        out!("{}", paint(DIM, r#"help("length") or help("km") for details"#));
    } else if topic == "currency" {
        out!("{}: 3-letter codes like USD, EUR, GBP, JPY. Rates by Exchange Rate API (exchangerate-api.com),", paint(NAME, "currency"));
        out!("are fetched only when two currencies meet, after allow_network_access(), and cached for a day.");
        out!("  {}", code("100 USD to EUR"));
    } else if let Some(kind) = DIMS.iter().map(|d| d.0).find(|k| *k == topic) {
        let units = units_of(kind);
        out!("{}: {}", paint(HEADING, &format!("{kind} units")), names(&units, " "));
        show(vec![format!("1 {} to {}", units[1], units[0]), format!("1 {} to {}", units[0], units[units.len() - 1])]);
        out!("{}", paint(DIM, r#"help("km") etc. for one unit"#));
    } else if let Some((names, scale, offset, dim, about)) = rows().find(|row| row.0.split(' ').any(|n| n == topic)) {
        let mut names = names.split(' ');
        let name = names.next().unwrap();
        let (full, desc) = about.split_once(": ").expect("unit description is \"full name: description\"");
        let kind = DIMS.iter().find(|d| d.1 == *dim).map(|d| d.0);
        out!("{}{}", paint(NAME, name), if full == name { String::new() } else { format!(": {full}") });
        out!("  {desc}");
        let row = |label: &str, text: String| out!("  {}  {text}", paint(DIM, &format!("{label:<6}")));
        // A goofy-only kind (gee) has no name and no real units to compare with.
        let Some(kind) = kind else {
            row("equals", code(&format!("1 {name} = {}", eval(&format!("1 {name} to {}", base_name(dim))))));
            return true;
        };
        row("kind", kind.to_string());
        let aliases: Vec<_> = names.collect();
        if !aliases.is_empty() {
            row("also", aliases.join(", "));
        }
        // The SI or named base unit of the kind; temperatures show their zero, since 1 C is not 1 K apart from 0 C.
        let base = base_name(dim);
        if base != name {
            let x = if *offset != 0.0 { 0 } else { 1 };
            row("equals", code(&format!("{x} {name} = {}", eval(&format!("{x} {name} to {base}")))));
        }
        // Examples convert to the closest-sized everyday unit, not the base unit `equals` already shows.
        let closeness = |r: &&(&str, f64)| (r.1 / scale).log10().abs();
        let others: Vec<_> = TABLE.iter().filter(|r| r.3 == *dim).map(|r| (r.0.split(' ').next().unwrap(), r.1)).filter(|r| r.0 != name).collect();
        let other = others.iter().filter(|r| r.0 != base).min_by(|a, b| closeness(a).total_cmp(&closeness(b))).or(others.first()).unwrap().0;
        out!("");
        show(vec![format!("1 {name} to {other}"), format!("1 {other} to {name}")]);
        out!("{}", paint(DIM, &format!("all {kind} units: help(\"{kind}\")")));
    } else if let Some((_, desc)) = USER.with(|u| u.borrow().iter().find(|(n, _)| n == topic).cloned()) {
        match desc.as_str() {
            "" => out!("{}: a base unit you defined with `unit {topic}`", paint(NAME, topic)),
            _ => out!("{}: a unit you defined, 1 {topic} = {desc}", paint(NAME, topic)),
        }
    } else {
        return false;
    }
    true
}

/// `rows` grouped by kind, in `DIMS` order, leaving out kinds with none.
pub fn kinds(rows: &'static [Row]) -> Vec<(&'static str, Vec<&'static Row>)> {
    DIMS.iter().map(|(kind, dim)| (*kind, rows.iter().filter(|r| r.3 == *dim).collect::<Vec<_>>())).filter(|k| !k.1.is_empty()).collect()
}

/// `all_units()`: every unit as a table of {name, full, desc, aliases, kind, si, source}. Never fetches rates, so
/// a currency's `si` is nil until they're loaded.
fn all_units() -> Value {
    let entry = |name: &str, full: &str, desc: &str, aliases: Vec<&str>, scale: f64, dim: &Dim, source: &str| {
        let kind = match DIMS.iter().find(|d| d.1 == *dim) {
            Some(d) => d.0.to_string(),
            None if *dim == MONEY => "money".into(),
            None if *dim == NONE => "number".into(),
            None => base_name(dim),
        };
        let si = if scale.is_nan() {
            Value::Nil
        } else {
            let base = base_terms(dim).iter().fold(Unit::default(), |u, (n, p)| u.mul(&unit(n).expect("base unit").pow(*p), 1));
            Value::qty(scale, base)
        };
        Value::map(IndexMap::from([
            ("name".into(), Value::str(name)),
            ("full".into(), Value::str(full)),
            ("desc".into(), Value::str(desc)),
            ("aliases".into(), Value::list(aliases.into_iter().map(Value::str).collect())),
            ("kind".into(), Value::str(kind)),
            ("si".into(), si),
            ("source".into(), Value::str(source)),
        ]))
    };
    let mut maps = Vec::new();
    for (source, table) in [("units", TABLE), ("goofy", goofy::TABLE), ("kitchen", kitchen::TABLE)] {
        for (names, scale, _, dim, about) in table {
            let mut names = names.split(' ');
            let name = names.next().unwrap();
            let (full, desc) = about.split_once(": ").expect("unit description is \"full name: description\"");
            maps.push(entry(name, full, desc, names.collect(), *scale, dim, source));
        }
    }
    let user = USER.with(|u| u.borrow().clone());
    // A currency is registered under its code and its lowercase alias; user units can be money too.
    let mut codes: Vec<_> = REGISTRY
        .with(|r| r.borrow().iter().filter(|(k, u)| u.dim == MONEY && **k == u.name && !user.iter().any(|(n, _)| n == *k)).map(|(_, u)| u.clone()).collect());
    codes.sort_by(|a, b| a.name.cmp(&b.name));
    for def in codes {
        maps.push(entry(&def.name, &def.name, "currency", vec![&def.name.to_lowercase()], def.scale.get(), &def.dim, "currency"));
    }
    for (name, desc) in &user {
        let def = lookup(name).expect("user unit is registered");
        maps.push(entry(name, name, desc, vec![], def.scale.get(), &def.dim, "user"));
    }
    Value::table(Table::from_maps(&maps).expect("all maps"))
}

/// The unit a kind is defined in, as (name, power) terms: its named SI unit (`J`, `lx`), else SI and user bases.
fn base_terms(dim: &Dim) -> Vec<(String, i8)> {
    if let Some(row) = TABLE.iter().find(|r| r.1 == 1.0 && r.2 == 0.0 && r.3 == *dim) {
        return vec![(row.0.split(' ').next().unwrap().to_string(), 1)];
    }
    const BASES: [&str; 11] = ["m", "kg", "s", "A", "K", "mol", "cd", "bit", "EUR", "rad", "Sv"];
    // User bases take slots in the order they were made.
    let user = USER.with(|u| u.borrow().iter().filter(|(_, d)| d.is_empty()).map(|(n, _)| n.clone()).collect::<Vec<_>>());
    BASES.iter().map(|n| n.to_string()).chain(user).zip(dim).filter(|(_, p)| **p != 0).map(|(n, p)| (n, *p)).collect()
}

/// The unit a kind is defined in, written out: `J`, `m/s`, `bit/s`, `pizza`.
fn base_name(dim: &Dim) -> String {
    product(base_terms(dim).iter().map(|(n, p)| (n.as_str(), *p)))
}

fn units_of(kind: &str) -> Vec<&'static str> {
    let dim = DIMS.iter().find(|d| d.0 == kind).unwrap().1;
    TABLE.iter().filter(|row| row.3 == dim).map(|row| row.0.split(' ').next().unwrap()).collect()
}

/// Exponents of the 7 SI bases (m kg s A K mol cd), the non-SI extras data, money, angle, dose,
/// then slots for user base units (`unit pizza`).
pub type Dim = [i8; 16];
pub const NONE: Dim = [0; 16];
const FIRST_USER_BASE: usize = 11;

/// Length, mass, time, current.
pub const fn d(l: i8, m: i8, t: i8, i: i8) -> Dim {
    let mut x = NONE;
    (x[0], x[1], x[2], x[3]) = (l, m, t, i);
    x
}

/// Base `i` to the power `p`, times `rest`.
const fn base(i: usize, p: i8, rest: Dim) -> Dim {
    let mut x = rest;
    x[i] += p;
    x
}
const LEN: Dim = d(1, 0, 0, 0);
const MASS: Dim = d(0, 1, 0, 0);
pub const TIME: Dim = d(0, 0, 1, 0);
pub const PER_TIME: Dim = d(0, 0, -1, 0);
const CURRENT: Dim = d(0, 0, 0, 1);
const TEMP: Dim = base(4, 1, NONE);
const AMOUNT: Dim = base(5, 1, NONE);
const MOLAR: Dim = base(5, 1, d(-3, 0, 0, 0));
const LIGHT: Dim = base(6, 1, NONE);
const LUX: Dim = base(6, 1, d(-2, 0, 0, 0));
pub const DATA: Dim = base(7, 1, NONE);
const RATE: Dim = base(7, 1, PER_TIME);
pub const MONEY: Dim = base(8, 1, NONE);
const ANGLE: Dim = base(9, 1, NONE);
// Its own base, not J/kg, so specific energies don't print as sieverts.
pub const DOSE: Dim = base(10, 1, NONE);
const VOLT: Dim = d(2, 1, -3, -1);
const OHM: Dim = d(2, 1, -3, -2);
const COULOMB: Dim = d(0, 0, 1, 1);
const FARAD: Dim = d(-2, -1, 4, 2);
const SIEMENS: Dim = d(-2, -1, 3, 2);
const WEBER: Dim = d(2, 1, -2, -1);
const TESLA: Dim = d(0, 1, -2, -1);
const HENRY: Dim = d(2, 1, -2, -2);

/// Names of the kinds of unit in TABLE, for `help`. Each needs at least two units.
#[rustfmt::skip]
pub const DIMS: &[(&str, Dim)] = &[
    ("length", LEN), ("mass", MASS), ("time", TIME), ("temperature", TEMP),
    ("volume", d(3, 0, 0, 0)), ("area", d(2, 0, 0, 0)), ("speed", d(1, 0, -1, 0)),
    ("data", DATA), ("rate", RATE), ("energy", d(2, 1, -2, 0)), ("power", d(2, 1, -3, 0)),
    ("pressure", d(-1, 1, -2, 0)), ("force", d(1, 1, -2, 0)), ("angle", ANGLE),
    ("current", CURRENT), ("voltage", VOLT), ("resistance", OHM), ("charge", COULOMB),
    ("capacitance", FARAD), ("conductance", SIEMENS), ("magnetic_flux", WEBER), ("magnetic_field", TESLA),
    ("inductance", HENRY), ("frequency", PER_TIME), ("amount", AMOUNT), ("concentration", MOLAR),
    ("light", LIGHT), ("illuminance", LUX), ("dose", DOSE),
];

/// Space-separated names (first is the display name), factor to SI, offset (temperatures only), dimension.
/// SI bases: m, kg, s, A, K, mol, cd; extras: bit, EUR, rad. Within a dimension, the first unit with
/// factor 1 is the one metric products are renamed to (see `named`), so J beats N*m and lm beats cd.
#[rustfmt::skip]
/// A unit row: space-separated names (primary first), scale to SI, offset, dimension, "full name: description" for help.
pub type Row = (&'static str, f64, f64, Dim, &'static str);

pub const TABLE: &[Row] = &[
    ("m meter meters metre metres", 1.0, 0.0, LEN, "meter: SI base unit of length, the distance light travels in 1/299792458 s"),
    ("km kilometer kilometers", 1e3, 0.0, LEN, "kilometer: 1000 meters"),
    ("cm centimeter centimeters", 1e-2, 0.0, LEN, "centimeter: a hundredth of a meter"),
    ("mm millimeter millimeters", 1e-3, 0.0, LEN, "millimeter: a thousandth of a meter"),
    ("µm um μm micrometer micrometers micron", 1e-6, 0.0, LEN, "micrometer: a millionth of a meter, also called a micron; about the size of a bacterium"),
    ("nm nanometer nanometers", 1e-9, 0.0, LEN, "nanometer: a billionth of a meter; visible light is 380 to 750 nm"),
    ("mi mile miles", 1609.344, 0.0, LEN, "mile: the international mile, 5280 feet"),
    ("yd yard yards", 0.9144, 0.0, LEN, "yard: 3 feet, exactly 0.9144 m"),
    ("ft foot feet", 0.3048, 0.0, LEN, "foot: 12 inches, exactly 0.3048 m"),
    ("in inch inches", 0.0254, 0.0, LEN, "inch: exactly 2.54 cm"),
    ("nmi", 1852.0, 0.0, LEN, "nautical mile: exactly 1852 m, about one minute of latitude"),
    ("au", 1.495978707e11, 0.0, LEN, "astronomical unit: about the mean distance from Earth to the Sun"),
    ("ly lightyear lightyears", 9.4607304725808e15, 0.0, LEN, "light-year: how far light travels in a Julian year"),
    ("kg kilogram kilograms", 1.0, 0.0, MASS, "kilogram: SI base unit of mass"),
    ("g gram grams", 1e-3, 0.0, MASS, "gram: a thousandth of a kilogram"),
    ("mg milligram milligrams", 1e-6, 0.0, MASS, "milligram: a thousandth of a gram"),
    ("µg ug μg microgram micrograms", 1e-9, 0.0, MASS, "microgram: a millionth of a gram"),
    ("t tonne tonnes", 1e3, 0.0, MASS, "tonne: the metric ton, 1000 kg"),
    ("lb lbs pound pounds", 0.45359237, 0.0, MASS, "pound: the avoirdupois pound, exactly 0.45359237 kg"),
    ("oz ounce ounces", 0.028349523125, 0.0, MASS, "ounce: the avoirdupois ounce, 1/16 pound; floz is the fluid ounce"),
    ("st stone", 6.35029318, 0.0, MASS, "stone: 14 pounds, used for body weight in the UK"),
    ("s sec secs second seconds", 1.0, 0.0, TIME, "second: SI base unit of time"),
    ("ms millisecond milliseconds", 1e-3, 0.0, TIME, "millisecond: a thousandth of a second"),
    ("µs us μs microsecond microseconds", 1e-6, 0.0, TIME, "microsecond: a millionth of a second"),
    ("ns nanosecond nanoseconds", 1e-9, 0.0, TIME, "nanosecond: a billionth of a second"),
    ("min mins minute minutes", 60.0, 0.0, TIME, "minute: 60 seconds"),
    ("h hr hrs hour hours", 3600.0, 0.0, TIME, "hour: 60 minutes"),
    ("d day days", 86400.0, 0.0, TIME, "day: 24 hours"),
    ("wk week weeks", 604800.0, 0.0, TIME, "week: 7 days"),
    ("mo month months", 2629746.0, 0.0, TIME, "month: the average Gregorian month, 1/12 of a year or about 30.44 days"),
    ("yr year years", 31556952.0, 0.0, TIME, "year: the average Gregorian year, 365.2425 days"),
    // Paid time: 8 h days, 40 h weeks, 52 weeks a year. `25 USD/h to USD/workyr` is a salary; `USD/yr` is calendar time.
    ("workday workdays", 8.0 * 3600.0, 0.0, TIME, "work day: 8 hours of paid time"),
    ("workwk workweek workweeks", 40.0 * 3600.0, 0.0, TIME, "work week: 40 hours of paid time"),
    ("workmo workmonth workmonths", 2080.0 / 12.0 * 3600.0, 0.0, TIME, "work month: 1/12 of a work year, about 173 hours"),
    ("workyr workyear workyears", 2080.0 * 3600.0, 0.0, TIME, "work year: 2080 hours, 52 weeks of 40; USD/h to USD/workyr gives a salary"),
    ("K kelvin", 1.0, 0.0, TEMP, "kelvin: SI base unit of temperature, counted from absolute zero"),
    ("C celsius degC", 1.0, 273.15, TEMP, "degree Celsius: kelvin minus 273.15; water freezes at 0 and boils at 100"),
    ("F fahrenheit degF", 5.0 / 9.0, 459.67, TEMP, "degree Fahrenheit: water freezes at 32 and boils at 212; a degree is 5/9 of a kelvin"),
    ("L l liter liters litre litres", 1e-3, 0.0, d(3, 0, 0, 0), "liter: a cubic decimeter, 1000 cm^3"),
    ("mL ml milliliter milliliters", 1e-6, 0.0, d(3, 0, 0, 0), "milliliter: a thousandth of a liter, 1 cm^3"),
    ("gal gallon gallons", 3.785411784e-3, 0.0, d(3, 0, 0, 0), "gallon: the US liquid gallon, 231 cubic inches"),
    ("qt quart quarts", 9.46352946e-4, 0.0, d(3, 0, 0, 0), "quart: the US liquid quart, a quarter gallon"),
    ("pt pint pints", 4.73176473e-4, 0.0, d(3, 0, 0, 0), "pint: the US liquid pint, half a quart"),
    ("cup cups", 2.365882365e-4, 0.0, d(3, 0, 0, 0), "cup: the US customary cup, half a pint"),
    ("floz", 2.95735295625e-5, 0.0, d(3, 0, 0, 0), "fluid ounce: the US fluid ounce, 1/8 cup"),
    ("tbsp tablespoon tablespoons", 1.478676478125e-5, 0.0, d(3, 0, 0, 0), "tablespoon: the US tablespoon, half a fluid ounce"),
    ("tsp teaspoon teaspoons", 4.92892159375e-6, 0.0, d(3, 0, 0, 0), "teaspoon: the US teaspoon, a third of a tablespoon"),
    ("ha hectare hectares", 1e4, 0.0, d(2, 0, 0, 0), "hectare: 10000 m^2, a square 100 m on a side"),
    ("acre acres", 4046.8564224, 0.0, d(2, 0, 0, 0), "acre: the international acre, 43560 square feet"),
    ("kph kmh", 1.0 / 3.6, 0.0, d(1, 0, -1, 0), "kilometer per hour: road speeds in most of the world"),
    ("mph", 0.44704, 0.0, d(1, 0, -1, 0), "mile per hour: road speeds in the US and UK"),
    ("kn knot knots", 1852.0 / 3600.0, 0.0, d(1, 0, -1, 0), "knot: one nautical mile per hour, used at sea and in the air"),
    ("bit bits", 1.0, 0.0, DATA, "bit: one binary digit, 0 or 1"),
    ("B byte bytes", 8.0, 0.0, DATA, "byte: 8 bits"),
    ("KB", 8e3, 0.0, DATA, "kilobyte: 1000 bytes; KiB is 1024"),
    ("MB", 8e6, 0.0, DATA, "megabyte: 10^6 bytes; MiB is 1024^2"),
    ("GB", 8e9, 0.0, DATA, "gigabyte: 10^9 bytes, as drive makers count; GiB is 1024^3"),
    ("TB", 8e12, 0.0, DATA, "terabyte: 10^12 bytes; TiB is 1024^4"),
    ("PB", 8e15, 0.0, DATA, "petabyte: 10^15 bytes"),
    ("KiB", 8.0 * 1024.0, 0.0, DATA, "kibibyte: 1024 bytes"),
    ("MiB", 8.0 * 1048576.0, 0.0, DATA, "mebibyte: 1024 KiB, 1048576 bytes"),
    ("GiB", 8.0 * 1073741824.0, 0.0, DATA, "gibibyte: 1024 MiB; what many operating systems call a GB"),
    ("TiB", 8.0 * 1099511627776.0, 0.0, DATA, "tebibyte: 1024 GiB"),
    ("PiB", 8.0 * 1125899906842624.0, 0.0, DATA, "pebibyte: 1024 TiB"),
    ("kbit Kb", 1e3, 0.0, DATA, "kilobit: 1000 bits"),
    ("Mbit Mb", 1e6, 0.0, DATA, "megabit: 10^6 bits; Mb is bits, MB is bytes"),
    ("Gbit Gb", 1e9, 0.0, DATA, "gigabit: 10^9 bits"),
    ("bps", 1.0, 0.0, RATE, "bit per second: the base unit of data rate"),
    ("kbps", 1e3, 0.0, RATE, "kilobit per second: 1000 bits each second"),
    ("Mbps", 1e6, 0.0, RATE, "megabit per second: how internet speeds are quoted; divide by 8 for MB/s"),
    ("Gbps", 1e9, 0.0, RATE, "gigabit per second: 1000 Mbps"),
    ("J joule joules", 1.0, 0.0, d(2, 1, -2, 0), "joule: SI unit of energy, a newton of force over a meter"),
    ("kJ", 1e3, 0.0, d(2, 1, -2, 0), "kilojoule: 1000 joules; food energy on labels outside the US"),
    ("MJ", 1e6, 0.0, d(2, 1, -2, 0), "megajoule: 10^6 joules"),
    ("cal calorie calories", 4.184, 0.0, d(2, 1, -2, 0), "calorie: the small calorie, warms 1 g of water by 1 °C"),
    ("kcal Cal", 4184.0, 0.0, d(2, 1, -2, 0), "kilocalorie: 1000 calories, the Calorie on food labels"),
    ("Wh", 3600.0, 0.0, d(2, 1, -2, 0), "watt-hour: one watt for an hour, 3600 J"),
    ("kWh", 3.6e6, 0.0, d(2, 1, -2, 0), "kilowatt-hour: 1000 Wh, how electricity is billed"),
    ("eV", 1.602176634e-19, 0.0, d(2, 1, -2, 0), "electronvolt: the energy an electron gains across 1 volt"),
    ("BTU btu", 1055.05585262, 0.0, d(2, 1, -2, 0), "British thermal unit: warms a pound of water by 1 °F; heating and air conditioning"),
    ("W watt watts", 1.0, 0.0, d(2, 1, -3, 0), "watt: SI unit of power, a joule per second"),
    ("kW", 1e3, 0.0, d(2, 1, -3, 0), "kilowatt: 1000 watts"),
    ("MW", 1e6, 0.0, d(2, 1, -3, 0), "megawatt: 10^6 watts"),
    ("hp horsepower", 745.699_871_582_270_2, 0.0, d(2, 1, -3, 0), "horsepower: mechanical horsepower, 550 foot-pounds per second"),
    ("Pa pascal", 1.0, 0.0, d(-1, 1, -2, 0), "pascal: SI unit of pressure, a newton per square meter"),
    ("kPa", 1e3, 0.0, d(-1, 1, -2, 0), "kilopascal: 1000 pascals"),
    ("MPa", 1e6, 0.0, d(-1, 1, -2, 0), "megapascal: 10^6 pascals, about 145 psi"),
    ("bar", 1e5, 0.0, d(-1, 1, -2, 0), "bar: 100 kPa, about the air pressure at sea level"),
    ("atm", 101325.0, 0.0, d(-1, 1, -2, 0), "standard atmosphere: 101325 Pa, the average air pressure at sea level"),
    ("psi", 6894.757293168, 0.0, d(-1, 1, -2, 0), "pound per square inch: pound-force per square inch; tire pressure in the US"),
    ("mmHg", 133.322387415, 0.0, d(-1, 1, -2, 0), "millimeter of mercury: used for blood pressure; 760 mmHg is 1 atm"),
    ("N newton newtons", 1.0, 0.0, d(1, 1, -2, 0), "newton: SI unit of force, accelerates 1 kg by 1 m/s^2"),
    ("kN", 1e3, 0.0, d(1, 1, -2, 0), "kilonewton: 1000 newtons"),
    ("lbf", 4.4482216152605, 0.0, d(1, 1, -2, 0), "pound-force: the weight of one pound under standard gravity"),
    ("rad radian radians", 1.0, 0.0, ANGLE, "radian: the angle whose arc equals the radius; a full turn is 2π"),
    ("deg degree degrees", PI / 180.0, 0.0, ANGLE, "degree: 1/360 of a turn"),
    ("turn turns", 2.0 * PI, 0.0, ANGLE, "turn: one full revolution, 360 degrees"),
    ("A amp amps ampere amperes", 1.0, 0.0, CURRENT, "ampere: SI base unit of electric current"),
    ("mA milliamp milliamps", 1e-3, 0.0, CURRENT, "milliampere: a thousandth of an ampere"),
    ("µA uA μA", 1e-6, 0.0, CURRENT, "microampere: a millionth of an ampere"),
    ("kA", 1e3, 0.0, CURRENT, "kiloampere: 1000 amperes"),
    ("V volt volts", 1.0, 0.0, VOLT, "volt: SI unit of voltage, a watt per ampere"),
    ("mV", 1e-3, 0.0, VOLT, "millivolt: a thousandth of a volt"),
    ("µV uV μV", 1e-6, 0.0, VOLT, "microvolt: a millionth of a volt"),
    ("kV", 1e3, 0.0, VOLT, "kilovolt: 1000 volts"),
    ("Ω ohm ohms", 1.0, 0.0, OHM, "ohm: SI unit of resistance, a volt per ampere"),
    ("mΩ mohm", 1e-3, 0.0, OHM, "milliohm: a thousandth of an ohm"),
    ("kΩ kohm", 1e3, 0.0, OHM, "kiloohm: 1000 ohms"),
    ("MΩ Mohm", 1e6, 0.0, OHM, "megaohm: 10^6 ohms"),
    // C and F stay Celsius and Fahrenheit.
    ("coulomb coulombs", 1.0, 0.0, COULOMB, "coulomb: SI unit of charge, an ampere for a second; C here is Celsius"),
    ("mAh", 3.6, 0.0, COULOMB, "milliampere-hour: battery capacity, 3.6 coulombs"),
    ("Ah", 3600.0, 0.0, COULOMB, "ampere-hour: battery capacity, 3600 coulombs"),
    ("farad farads", 1.0, 0.0, FARAD, "farad: SI unit of capacitance, a coulomb per volt; F here is Fahrenheit"),
    ("mF", 1e-3, 0.0, FARAD, "millifarad: a thousandth of a farad"),
    ("µF uF μF", 1e-6, 0.0, FARAD, "microfarad: a millionth of a farad"),
    ("nF", 1e-9, 0.0, FARAD, "nanofarad: a billionth of a farad"),
    ("pF", 1e-12, 0.0, FARAD, "picofarad: a trillionth of a farad"),
    ("S siemens", 1.0, 0.0, SIEMENS, "siemens: SI unit of conductance, the inverse of an ohm"),
    ("mS", 1e-3, 0.0, SIEMENS, "millisiemens: a thousandth of a siemens"),
    ("Wb weber webers", 1.0, 0.0, WEBER, "weber: SI unit of magnetic flux, a volt-second"),
    ("mWb", 1e-3, 0.0, WEBER, "milliweber: a thousandth of a weber"),
    ("T tesla teslas", 1.0, 0.0, TESLA, "tesla: SI unit of magnetic field, a weber per square meter; t is the tonne"),
    ("mT", 1e-3, 0.0, TESLA, "millitesla: a thousandth of a tesla; a fridge magnet is about 5 mT"),
    ("µT uT μT", 1e-6, 0.0, TESLA, "microtesla: a millionth of a tesla; Earth's field is 25 to 65 µT"),
    ("H henry henries", 1.0, 0.0, HENRY, "henry: SI unit of inductance, a weber per ampere"),
    ("mH", 1e-3, 0.0, HENRY, "millihenry: a thousandth of a henry"),
    ("µH uH μH", 1e-6, 0.0, HENRY, "microhenry: a millionth of a henry"),
    ("Hz hertz", 1.0, 0.0, PER_TIME, "hertz: one cycle per second"),
    ("kHz", 1e3, 0.0, PER_TIME, "kilohertz: 1000 hertz"),
    ("MHz", 1e6, 0.0, PER_TIME, "megahertz: 10^6 hertz"),
    ("GHz", 1e9, 0.0, PER_TIME, "gigahertz: 10^9 hertz; CPU clocks and Wi-Fi"),
    ("rpm", 1.0 / 60.0, 0.0, PER_TIME, "revolution per minute: rotational speed, 1/60 Hz"),
    ("mol mole moles", 1.0, 0.0, AMOUNT, "mole: SI base unit of amount, 6.02214076e23 particles"),
    ("mmol", 1e-3, 0.0, AMOUNT, "millimole: a thousandth of a mole"),
    ("µmol umol μmol", 1e-6, 0.0, AMOUNT, "micromole: a millionth of a mole"),
    ("kmol", 1e3, 0.0, AMOUNT, "kilomole: 1000 moles"),
    ("M molar", 1e3, 0.0, MOLAR, "molar: a mole per liter"),
    ("mM millimolar", 1.0, 0.0, MOLAR, "millimolar: a thousandth of a mole per liter"),
    ("µM uM μM micromolar", 1e-3, 0.0, MOLAR, "micromolar: a millionth of a mole per liter"),
    ("Sv sievert sieverts", 1.0, 0.0, DOSE, "sievert: SI unit of radiation dose to the body; a CT scan is about 10 mSv"),
    ("mSv millisievert millisieverts", 1e-3, 0.0, DOSE, "millisievert: a thousandth of a sievert; natural background is about 3 mSv a year"),
    ("µSv uSv μSv microsievert microsieverts", 1e-6, 0.0, DOSE, "microsievert: a millionth of a sievert; a dental X-ray is about 5 µSv"),
    // Steradians are dimensionless in SI, so lumens and candelas share a dimension.
    ("lm lumen lumens", 1.0, 0.0, LIGHT, "lumen: the total visible light a source gives off"),
    ("cd candela candelas", 1.0, 0.0, LIGHT, "candela: SI base unit of luminous intensity, the light sent in one direction"),
    ("lx lux", 1.0, 0.0, LUX, "lux: illuminance, a lumen per square meter"),
    ("fc footcandle footcandles", 10.763910416709722, 0.0, LUX, "foot-candle: a lumen per square foot, about 10.76 lux"),
];

#[derive(Debug)]
pub struct UnitDef {
    pub name: String,
    /// NaN for a currency whose rate isn't loaded yet; read it through `scale()`.
    scale: Cell<f64>,
    pub offset: f64,
    pub dim: Dim,
}

/// A product of units with integer powers, e.g. km/h = [(km, 1), (h, -1)].
#[derive(Debug, Clone, Default)]
pub struct Unit(pub Vec<(Rc<UnitDef>, i8)>);

impl UnitDef {
    /// Factor to SI; the first currency rate asked for loads them all.
    pub fn scale(&self) -> f64 {
        if self.scale.get().is_nan() {
            load_rates_once();
        }
        self.scale.get()
    }
}

impl Unit {
    pub fn dim(&self) -> Dim {
        let mut dim = NONE;
        for (u, p) in &self.0 {
            for (acc, x) in dim.iter_mut().zip(u.dim) {
                *acc += x * p;
            }
        }
        dim
    }

    pub fn scale(&self) -> f64 {
        self.0.iter().map(|(u, p)| u.scale().powi(*p as i32)).product()
    }

    /// Offsets only apply to a lone unit like `C`, not inside `C/s`.
    pub fn offset(&self) -> f64 {
        match self.0.as_slice() {
            [(u, 1)] => u.offset,
            _ => 0.0,
        }
    }

    pub fn to_si(&self, v: f64) -> f64 {
        (v + self.offset()) * self.scale()
    }

    pub fn value_from_si(&self, si: f64) -> f64 {
        si / self.scale() - self.offset()
    }

    /// `self * other^sign`, merging repeated units (m * m = m^2).
    pub fn mul(&self, other: &Unit, sign: i8) -> Unit {
        let mut terms = self.0.clone();
        for (u, p) in &other.0 {
            match terms.iter_mut().find(|(t, _)| t.name == u.name) {
                Some(t) => t.1 += p * sign,
                None => terms.push((u.clone(), p * sign)),
            }
        }
        terms.retain(|(_, p)| *p != 0);
        Unit(terms)
    }

    pub fn pow(&self, n: i8) -> Unit {
        Unit(self.0.iter().map(|(u, p)| (u.clone(), p * n)).collect())
    }

    /// Single term with power 1 and the given name (e.g. "mo" for calendar math).
    pub fn is(&self, name: &str) -> bool {
        matches!(self.0.as_slice(), [(u, 1)] if u.name == name)
    }
}

/// `x u` as a quantity in `t`, if the dimensions match.
pub fn to_unit(x: f64, u: &Unit, t: Unit) -> Result<Value, String> {
    if u.dim() != t.dim() {
        return Err(format!("cannot convert {} to {}", describe(u, None), describe(&t, None)));
    }
    Ok(Value::Qty(convert_value(x, u, &t)?, t))
}

/// `x` in `from` as a value in `to` (same dimension). Through `from / to`, so currencies that cancel
/// (`USD/h` to `USD/workyr`) need no exchange rate.
pub fn convert_value(x: f64, from: &Unit, to: &Unit) -> Result<f64, String> {
    if from.offset() != 0.0 || to.offset() != 0.0 {
        return Ok(to.value_from_si(from.to_si(x)));
    }
    let f = from.mul(to, -1).scale();
    if f.is_nan() { Err(rate_error()) } else { Ok(x * f) }
}

/// `si` as whole amounts of each unit but the last, which gets the remainder: "5 ft 6 in", "79 d 9 h 36 min".
pub fn split(si: f64, units: &[Unit]) -> String {
    let (last, big) = units.split_last().expect("at least one unit");
    let round = |x: f64| (x * 1e6).round() / 1e6;
    let mut rest = round(last.value_from_si(si.abs()));
    let mut parts = Vec::new();
    for u in big {
        let f = u.scale() / last.scale();
        let n = (rest / f + 1e-9).floor();
        rest = round(rest - n * f);
        if n != 0.0 {
            parts.push(format!("{n} {u}"));
        }
    }
    if rest != 0.0 || parts.is_empty() {
        parts.push(format!("{} {last}", crate::value::fmt_float(rest)));
    }
    let sign = if si < 0.0 { "-" } else { "" };
    format!("{sign}{}", parts.join(" "))
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", product(self.0.iter().map(|(u, p)| (u.name.as_str(), *p))))
    }
}

thread_local! {
    static REGISTRY: RefCell<HashMap<String, Rc<UnitDef>>> = RefCell::new(builtin_units());
    static RATES_LOADED: RefCell<bool> = const { RefCell::new(false) };
    static RATES_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
    /// Units made with `unit`: name and what it was defined as, for `help`.
    static USER: RefCell<Vec<(String, String)>> = const { RefCell::new(Vec::new()) };
}

/// `unit pizza` (no value) makes a new base unit; `unit slice = pizza / 8` or `unit dozen = 12` one
/// in terms of others. Built-in units can't be redefined; user ones can.
pub fn define(name: &str, value: Option<&Value>) -> Result<(), String> {
    let user = USER.with(|u| u.borrow().iter().position(|(n, _)| n == name));
    if user.is_none() && lookup(name).is_ok() {
        return Err(format!("`{name}` is already a unit"));
    }
    let (scale, dim, desc) = match value {
        // Redefining a base keeps its slot, so values made with it still convert.
        None if user.is_some_and(|i| USER.with(|u| u.borrow()[i].1.is_empty())) => return Ok(()),
        None => {
            let used = USER.with(|u| u.borrow().iter().filter(|(_, d)| d.is_empty()).count());
            // ponytail: fixed slots in Dim; widen Dim if 6 user bases ever feels tight.
            if FIRST_USER_BASE + used >= NONE.len() {
                return Err(format!("too many base units: at most {}", NONE.len() - FIRST_USER_BASE));
            }
            (1.0, base(FIRST_USER_BASE + used, 1, NONE), String::new())
        }
        Some(v @ Value::Qty(x, u)) => (u.to_si(*x), u.dim(), v.to_string()),
        Some(v) => match num(v) {
            Some(x) => (x, NONE, v.to_string()),
            None => return Err(format!("expected a quantity or number, got {}", v.type_name())),
        },
    };
    if scale.is_nan() {
        return Err(rate_error());
    }
    let def = Rc::new(UnitDef { name: name.into(), scale: Cell::new(scale), offset: 0.0, dim });
    REGISTRY.with(|r| r.borrow_mut().insert(name.into(), def));
    USER.with(|u| {
        let mut u = u.borrow_mut();
        u.retain(|(n, _)| n != name);
        u.push((name.into(), desc));
    });
    Ok(())
}

/// Names of units made with `unit`, for completion.
pub fn user_units() -> Vec<String> {
    USER.with(|u| u.borrow().iter().map(|(n, _)| n.clone()).collect())
}

/// Common currencies, known without fetching, so `25 USD/h * 40 h` stays offline.
const CURRENCIES: &str = "AUD BGN BRL CAD CHF CNY CZK DKK EUR GBP HKD HUF IDR ILS INR ISK JPY KRW MXN MYR NOK NZD PHP PLN RON SEK SGD THB TRY USD ZAR";

fn currency(code: &str, scale: f64) -> Rc<UnitDef> {
    Rc::new(UnitDef { name: code.into(), scale: Cell::new(scale), offset: 0.0, dim: MONEY })
}

fn rate_error() -> String {
    RATES_ERROR.with(|e| format!("cannot load currency rates: {}", e.borrow()))
}

/// Fills in every currency's rate, once; on failure their scales stay NaN and `rate_error` says why.
fn load_rates_once() {
    if RATES_LOADED.with(|l| l.replace(true)) {
        return;
    }
    RATES_ERROR.with(|e| e.borrow_mut().clear());
    match load_rates() {
        Ok(rates) => REGISTRY.with(|r| {
            let mut r = r.borrow_mut();
            for (code, per_eur) in rates {
                match r.get(&code) {
                    Some(def) => def.scale.set(1.0 / per_eur),
                    None => {
                        let def = currency(&code, 1.0 / per_eur);
                        r.entry(code.to_lowercase()).or_insert(def.clone());
                        r.insert(code, def);
                    }
                }
            }
        }),
        Err(e) => RATES_ERROR.with(|x| *x.borrow_mut() = e),
    }
}

/// Every unit row: the real ones, then those children add.
pub fn rows() -> impl Iterator<Item = &'static Row> {
    TABLE.iter().chain(goofy::TABLE).chain(kitchen::TABLE)
}

fn builtin_units() -> HashMap<String, Rc<UnitDef>> {
    let mut map = HashMap::new();
    for (names, scale, offset, dim, _) in rows() {
        let primary = names.split(' ').next().unwrap();
        let def = Rc::new(UnitDef { name: primary.into(), scale: Cell::new(*scale), offset: *offset, dim: *dim });
        for n in names.split(' ') {
            map.insert(n.to_string(), def.clone());
        }
    }
    // `banana_for_scale`.
    for (names, ..) in goofy::TABLE {
        let primary = names.split(' ').next().unwrap();
        map.insert(format!("{primary}_for_scale"), map[primary].clone());
    }
    for code in CURRENCIES.split(' ') {
        let def = currency(code, if code == "EUR" { 1.0 } else { f64::NAN });
        map.insert(code.to_lowercase(), def.clone());
        map.insert(code.into(), def);
    }
    map
}

pub fn lookup(name: &str) -> Result<Rc<UnitDef>, String> {
    if let Some(u) = REGISTRY.with(|r| r.borrow().get(name).cloned()) {
        return Ok(u);
    }
    // Any other code the rate source has (VND, ARS, ...): load the rates to find out.
    let looks_like_currency = name.len() == 3 && name.bytes().all(|b| b.is_ascii_uppercase());
    if looks_like_currency && !RATES_LOADED.with(|l| *l.borrow()) {
        load_rates_once();
        return lookup(name);
    }
    // Rates failed to load, so a real code like VND isn't registered: say why, not just "unknown".
    if looks_like_currency && RATES_ERROR.with(|e| !e.borrow().is_empty()) {
        return Err(format!("unknown unit `{name}` ({})", rate_error()));
    }
    let known: Vec<String> = REGISTRY.with(|r| r.borrow().keys().cloned().collect());
    let targets = crate::modules::modules().flat_map(|m| m.targets.iter().map(|t| t.0));
    Err(format!("unknown unit `{name}`{}", crate::error::did_you_mean(name, known.iter().map(String::as_str).chain(targets))))
}

/// Dimension of `spec` from already-loaded units only, so REPL completion never fetches rates.
pub fn known_dim(spec: &UnitSpec) -> Option<Dim> {
    REGISTRY.with(|r| {
        let r = r.borrow();
        let mut dim = NONE;
        for (name, p) in spec {
            for (acc, x) in dim.iter_mut().zip(r.get(name)?.dim) {
                *acc += x * p;
            }
        }
        Some(dim)
    })
}

/// Display names of every loaded unit of dimension `dim`, sorted.
pub fn names_with_dim(dim: Dim) -> Vec<String> {
    let mut v: Vec<String> = REGISTRY.with(|r| r.borrow().values().filter(|u| u.dim == dim).map(|u| u.name.clone()).collect());
    v.sort();
    v.dedup();
    v
}

/// A unit from space-separated `name^power` terms: `m s^-1`.
pub fn terms(s: &str) -> Result<Unit, String> {
    let mut u = Unit::default();
    for t in s.split(' ') {
        let (name, p) = t.split_once('^').unwrap_or((t, "1"));
        u = u.mul(&unit(name)?.pow(p.parse().map_err(|_| format!("bad power in `{t}`"))?), 1);
    }
    Ok(u)
}

pub fn unit(name: &str) -> Result<Unit, String> {
    Ok(Unit(vec![(lookup(name)?, 1)]))
}

/// Lets the next currency conversion try loading rates again, now that it may go online.
pub fn retry_rates() {
    RATES_LOADED.with(|l| *l.borrow_mut() = false);
}

fn fetch_rates() -> Result<String, String> {
    if !crate::modules::sys::network_allowed() {
        return Err("not cached; call allow_network_access() to fetch them".into());
    }
    ureq::get("https://open.er-api.com/v6/latest/EUR").call().and_then(|mut r| r.body_mut().read_to_string()).map_err(|e| e.to_string())
}

/// Units per 1 EUR from open.er-api.com (about 160 currencies, updated daily), cached on disk for a day.
fn load_rates() -> Result<Vec<(String, f64)>, String> {
    let cache = crate::cache_dir().map(|d| d.join("er-rates.json"));
    let age = cache.as_ref().and_then(|p| p.metadata().ok()?.modified().ok()?.elapsed().ok());
    let body = match &cache {
        Some(p) if age.is_some_and(|a| a.as_secs() < 24 * 3600) => std::fs::read_to_string(p).map_err(|e| e.to_string())?,
        _ => match fetch_rates() {
            Ok(body) => {
                if let Some(p) = &cache {
                    let _ = std::fs::create_dir_all(p.parent().unwrap());
                    let _ = std::fs::write(p, &body);
                }
                body
            }
            // Offline or not allowed online: fall back to a stale cache if there is one.
            Err(e) => match cache.as_ref().and_then(|p| std::fs::read_to_string(p).ok()) {
                Some(body) => {
                    let days = age.map_or(0, |a| a.as_secs() / 86400);
                    eprintln!("note: using currency rates cached {days} day{} ago", if days == 1 { "" } else { "s" });
                    body
                }
                None => return Err(e),
            },
        },
    };
    let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let mut rates: Vec<(String, f64)> =
        json["rates"].as_object().ok_or("unexpected response")?.iter().filter_map(|(k, v)| Some((k.clone(), v.as_f64()?))).collect();
    rates.push(("EUR".into(), 1.0));
    Ok(rates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interp::tests::{show, try_eval};

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9 * b.abs().max(1.0)
    }

    #[test]
    fn units_lists_every_source() {
        try_eval("unit widget").unwrap();
        for source in ["units", "goofy", "kitchen", "currency", "user"] {
            assert_eq!(show(&format!(r#"all_units().filter(|u| u.source == "{source}").len > 0"#)), "true", "{source}");
        }
        assert_eq!(show(r#"all_units().filter(|u| u.name == "mi")[0].si"#), "1609.34 m");
        assert_eq!(show(r#"all_units().filter(|u| u.name == "USD")[0].kind"#), "money");
        assert_eq!(show(r#"all_units().filter(|u| u.name == "widget")[0].kind"#), "widget");
    }

    #[test]
    fn convert_linear_and_offset() {
        let (km, mi) = (unit("km").unwrap(), unit("mi").unwrap());
        assert!(close(mi.value_from_si(km.to_si(5.0)), 3.1068559611866697));
        let (f, c) = (unit("F").unwrap(), unit("C").unwrap());
        assert!(close(c.value_from_si(f.to_si(212.0)), 100.0));
    }

    #[test]
    fn compound_units() {
        let kmh = unit("km").unwrap().mul(&unit("h").unwrap(), -1);
        assert_eq!(kmh.dim(), unit("kph").unwrap().dim());
        assert_eq!(kmh.to_string(), "km/h");
        let m2 = unit("m").unwrap().mul(&unit("m").unwrap(), 1);
        assert_eq!(m2.to_string(), "m^2");
        assert!(unit("m").unwrap().mul(&unit("m").unwrap(), -1).0.is_empty());
    }

    #[test]
    fn end_to_end() {
        assert_eq!(show("5 ft 11 in to cm"), "180.34 cm");
        assert_eq!(show("1 h 30 min 15 s to s"), "5415 s");
        assert_eq!(show("2 * 1 h 30 min"), "3 h");
        assert_eq!(show(r#"parse(1.8 m to ft in) to m"#), "1.8 m");
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
        assert_eq!(show("2 A * 5 ohm"), "10 V");
        assert_eq!(show("2 mA * 5 kohm"), "10 V");
        assert_eq!(show("12 V / 4 Ω"), "3 A");
        assert_eq!(show("9.81 m/s^2 * 70 kg"), "686.7 N");
        assert_eq!(show("2 N * 3 m"), "6 J");
        assert_eq!(show("1 / 2 s"), "0.5 Hz");
        assert_eq!(show("2 A * 3 s"), "6 coulomb");
        assert_eq!(show("60 W * 8 h"), "480 W*h");
        assert_eq!(show("25 USD/h * 40 h"), "$1,000.00");
        assert_eq!(show("2 kV * 3"), "6 kV");
        assert_eq!(show("2 N * 3 m to N*m"), "6 N*m");
        assert_eq!(show("1 uF to nF"), "1000 nF");
        assert_eq!(show("2000 mAh to coulomb"), "7200 coulomb");
        assert_eq!(show("3000 rpm to Hz"), "50 Hz");
        assert_eq!(
            show(
                "unit sprint = 2 wk
3 sprint to d"
            ),
            "42 d"
        );
        assert_eq!(
            show(
                "unit pizza
unit slice = pizza / 8
2 pizza to slice"
            ),
            "16 slice"
        );
        assert_eq!(
            show(
                "unit dozen = 12
2 dozen to dozen"
            ),
            "2 dozen"
        );
        assert!(try_eval("unit km = 3").is_err());
        assert!(
            try_eval(
                "unit widget
1 pizza + 1 widget"
            )
            .is_err()
        );
        assert_eq!(
            show(
                "unit = 5
unit * 2"
            ),
            "10"
        );
        assert_eq!(show("0.0012 km to best"), "1.2 m");
        assert_eq!(show("3600 J/s to best"), "3.6 kW");
        assert_eq!(show("1 kg*m^2/s^2 to best"), "1 J");
        assert_eq!(show("1.5e9 B.simplify"), "1.5 GB");
        assert_eq!(show("5000 ft to best"), "5000 ft");
        assert_eq!(show("20 C to best"), "20 C");
        assert_eq!(show("5 cd to best"), "5 cd");
        assert_eq!(show("0.00002 m to best"), "20 µm");
        assert!(try_eval("5 to best").is_err());
        assert!(try_eval("5 km + 1 kg").is_err());
        assert!(try_eval("5 km to kg").is_err());
    }
}
