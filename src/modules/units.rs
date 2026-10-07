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
use crate::value::{Value, num};
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
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    match (name, args) {
        ("simplify", [Value::Qty(x, u)]) => Ok(simplify(*x, u)),
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
    use crate::help::{out, show};
    if topic == "units" {
        let w = DIMS.iter().map(|d| d.0.len()).max().unwrap_or(0);
        out!("  {:<w$}  3-letter codes (USD EUR GBP ...), live rates fetched when converting", "currency");
        out!("help(\"length\") or help(\"km\") for details");
    } else if topic == "currency" {
        out!("currency: 3-letter codes like USD, EUR, GBP, JPY. Rates by Exchange Rate API (exchangerate-api.com),");
        out!("are fetched only when two currencies meet, after allow_network_access(), and cached for a day.");
        out!("  100 USD to EUR");
    } else if let Some(kind) = DIMS.iter().map(|d| d.0).find(|k| *k == topic) {
        let units = units_of(kind);
        out!("{kind} units: {}", units.join(" "));
        show(vec![format!("1 {} to {}", units[1], units[0]), format!("1 {} to {}", units[0], units[units.len() - 1])]);
    } else if let Some((names, _, _, dim)) = rows().find(|row| row.0.split(' ').any(|n| n == topic)) {
        let mut names = names.split(' ');
        let name = names.next().unwrap();
        let kind = DIMS.iter().find(|d| d.1 == *dim).unwrap().0;
        let aliases: Vec<_> = names.collect();
        let aka = if aliases.is_empty() { String::new() } else { format!("  (also {})", aliases.join(", ")) };
        out!("{name}: {kind}{aka}");
        let other = units_of(kind).into_iter().find(|u| *u != name).unwrap();
        show(vec![format!("1 {name} to {other}"), format!("1 {other} to {name}")]);
        out!("all {kind} units: help(\"{kind}\")");
    } else if let Some((_, desc)) = USER.with(|u| u.borrow().iter().find(|(n, _)| n == topic).cloned()) {
        match desc.as_str() {
            "" => out!("{topic}: a base unit you defined with `unit {topic}`"),
            _ => out!("{topic}: a unit you defined, 1 {topic} = {desc}"),
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

fn units_of(kind: &str) -> Vec<&'static str> {
    let dim = DIMS.iter().find(|d| d.0 == kind).unwrap().1;
    TABLE.iter().filter(|row| row.3 == dim).map(|row| row.0.split(' ').next().unwrap()).collect()
}

/// Exponents of the 7 SI bases (m kg s A K mol cd), the non-SI extras data, money, angle,
/// then slots for user base units (`unit pizza`).
pub type Dim = [i8; 16];
pub const NONE: Dim = [0; 16];
const FIRST_USER_BASE: usize = 10;

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
    ("light", LIGHT), ("illuminance", LUX),
];

/// Space-separated names (first is the display name), factor to SI, offset (temperatures only), dimension.
/// SI bases: m, kg, s, A, K, mol, cd; extras: bit, EUR, rad. Within a dimension, the first unit with
/// factor 1 is the one metric products are renamed to (see `named`), so J beats N*m and lm beats cd.
#[rustfmt::skip]
/// A unit row: space-separated names (primary first), scale to SI, offset, dimension.
pub type Row = (&'static str, f64, f64, Dim);

pub const TABLE: &[Row] = &[
    ("m meter meters metre metres", 1.0, 0.0, LEN),
    ("km kilometer kilometers", 1e3, 0.0, LEN),
    ("cm centimeter centimeters", 1e-2, 0.0, LEN),
    ("mm millimeter millimeters", 1e-3, 0.0, LEN),
    ("µm um μm micrometer micrometers micron", 1e-6, 0.0, LEN),
    ("nm nanometer nanometers", 1e-9, 0.0, LEN),
    ("mi mile miles", 1609.344, 0.0, LEN),
    ("yd yard yards", 0.9144, 0.0, LEN),
    ("ft foot feet", 0.3048, 0.0, LEN),
    ("in inch inches", 0.0254, 0.0, LEN),
    ("nmi", 1852.0, 0.0, LEN),
    ("au", 1.495978707e11, 0.0, LEN),
    ("ly lightyear lightyears", 9.4607304725808e15, 0.0, LEN),
    ("kg kilogram kilograms", 1.0, 0.0, MASS),
    ("g gram grams", 1e-3, 0.0, MASS),
    ("mg milligram milligrams", 1e-6, 0.0, MASS),
    ("µg ug μg microgram micrograms", 1e-9, 0.0, MASS),
    ("t tonne tonnes", 1e3, 0.0, MASS),
    ("lb lbs pound pounds", 0.45359237, 0.0, MASS),
    ("oz ounce ounces", 0.028349523125, 0.0, MASS),
    ("st stone", 6.35029318, 0.0, MASS),
    ("s sec secs second seconds", 1.0, 0.0, TIME),
    ("ms millisecond milliseconds", 1e-3, 0.0, TIME),
    ("µs us μs microsecond microseconds", 1e-6, 0.0, TIME),
    ("ns nanosecond nanoseconds", 1e-9, 0.0, TIME),
    ("min mins minute minutes", 60.0, 0.0, TIME),
    ("h hr hrs hour hours", 3600.0, 0.0, TIME),
    ("d day days", 86400.0, 0.0, TIME),
    ("wk week weeks", 604800.0, 0.0, TIME),
    ("mo month months", 2629746.0, 0.0, TIME),
    ("yr year years", 31556952.0, 0.0, TIME),
    // Paid time: 8 h days, 40 h weeks, 52 weeks a year. `25 USD/h to USD/workyr` is a salary; `USD/yr` is calendar time.
    ("workday workdays", 8.0 * 3600.0, 0.0, TIME),
    ("workwk workweek workweeks", 40.0 * 3600.0, 0.0, TIME),
    ("workmo workmonth workmonths", 2080.0 / 12.0 * 3600.0, 0.0, TIME),
    ("workyr workyear workyears", 2080.0 * 3600.0, 0.0, TIME),
    ("K kelvin", 1.0, 0.0, TEMP),
    ("C celsius degC", 1.0, 273.15, TEMP),
    ("F fahrenheit degF", 5.0 / 9.0, 459.67, TEMP),
    ("L l liter liters litre litres", 1e-3, 0.0, d(3, 0, 0, 0)),
    ("mL ml milliliter milliliters", 1e-6, 0.0, d(3, 0, 0, 0)),
    ("gal gallon gallons", 3.785411784e-3, 0.0, d(3, 0, 0, 0)),
    ("qt quart quarts", 9.46352946e-4, 0.0, d(3, 0, 0, 0)),
    ("pt pint pints", 4.73176473e-4, 0.0, d(3, 0, 0, 0)),
    ("cup cups", 2.365882365e-4, 0.0, d(3, 0, 0, 0)),
    ("floz", 2.95735295625e-5, 0.0, d(3, 0, 0, 0)),
    ("tbsp", 1.478676478125e-5, 0.0, d(3, 0, 0, 0)),
    ("tsp", 4.92892159375e-6, 0.0, d(3, 0, 0, 0)),
    ("ha hectare hectares", 1e4, 0.0, d(2, 0, 0, 0)),
    ("acre acres", 4046.8564224, 0.0, d(2, 0, 0, 0)),
    ("kph kmh", 1.0 / 3.6, 0.0, d(1, 0, -1, 0)),
    ("mph", 0.44704, 0.0, d(1, 0, -1, 0)),
    ("kn knot knots", 1852.0 / 3600.0, 0.0, d(1, 0, -1, 0)),
    ("bit bits", 1.0, 0.0, DATA),
    ("B byte bytes", 8.0, 0.0, DATA),
    ("KB", 8e3, 0.0, DATA),
    ("MB", 8e6, 0.0, DATA),
    ("GB", 8e9, 0.0, DATA),
    ("TB", 8e12, 0.0, DATA),
    ("PB", 8e15, 0.0, DATA),
    ("KiB", 8.0 * 1024.0, 0.0, DATA),
    ("MiB", 8.0 * 1048576.0, 0.0, DATA),
    ("GiB", 8.0 * 1073741824.0, 0.0, DATA),
    ("TiB", 8.0 * 1099511627776.0, 0.0, DATA),
    ("PiB", 8.0 * 1125899906842624.0, 0.0, DATA),
    ("kbit Kb", 1e3, 0.0, DATA),
    ("Mbit Mb", 1e6, 0.0, DATA),
    ("Gbit Gb", 1e9, 0.0, DATA),
    ("bps", 1.0, 0.0, RATE),
    ("kbps", 1e3, 0.0, RATE),
    ("Mbps", 1e6, 0.0, RATE),
    ("Gbps", 1e9, 0.0, RATE),
    ("J joule joules", 1.0, 0.0, d(2, 1, -2, 0)),
    ("kJ", 1e3, 0.0, d(2, 1, -2, 0)),
    ("MJ", 1e6, 0.0, d(2, 1, -2, 0)),
    ("cal calorie calories", 4.184, 0.0, d(2, 1, -2, 0)),
    ("kcal Cal", 4184.0, 0.0, d(2, 1, -2, 0)),
    ("Wh", 3600.0, 0.0, d(2, 1, -2, 0)),
    ("kWh", 3.6e6, 0.0, d(2, 1, -2, 0)),
    ("eV", 1.602176634e-19, 0.0, d(2, 1, -2, 0)),
    ("BTU btu", 1055.05585262, 0.0, d(2, 1, -2, 0)),
    ("W watt watts", 1.0, 0.0, d(2, 1, -3, 0)),
    ("kW", 1e3, 0.0, d(2, 1, -3, 0)),
    ("MW", 1e6, 0.0, d(2, 1, -3, 0)),
    ("hp horsepower", 745.699_871_582_270_2, 0.0, d(2, 1, -3, 0)),
    ("Pa pascal", 1.0, 0.0, d(-1, 1, -2, 0)),
    ("kPa", 1e3, 0.0, d(-1, 1, -2, 0)),
    ("MPa", 1e6, 0.0, d(-1, 1, -2, 0)),
    ("bar", 1e5, 0.0, d(-1, 1, -2, 0)),
    ("atm", 101325.0, 0.0, d(-1, 1, -2, 0)),
    ("psi", 6894.757293168, 0.0, d(-1, 1, -2, 0)),
    ("mmHg", 133.322387415, 0.0, d(-1, 1, -2, 0)),
    ("N newton newtons", 1.0, 0.0, d(1, 1, -2, 0)),
    ("kN", 1e3, 0.0, d(1, 1, -2, 0)),
    ("lbf", 4.4482216152605, 0.0, d(1, 1, -2, 0)),
    ("rad radian radians", 1.0, 0.0, ANGLE),
    ("deg degree degrees", PI / 180.0, 0.0, ANGLE),
    ("turn turns", 2.0 * PI, 0.0, ANGLE),
    ("A amp amps ampere amperes", 1.0, 0.0, CURRENT),
    ("mA milliamp milliamps", 1e-3, 0.0, CURRENT),
    ("µA uA μA", 1e-6, 0.0, CURRENT),
    ("kA", 1e3, 0.0, CURRENT),
    ("V volt volts", 1.0, 0.0, VOLT),
    ("mV", 1e-3, 0.0, VOLT),
    ("µV uV μV", 1e-6, 0.0, VOLT),
    ("kV", 1e3, 0.0, VOLT),
    ("Ω ohm ohms", 1.0, 0.0, OHM),
    ("mΩ mohm", 1e-3, 0.0, OHM),
    ("kΩ kohm", 1e3, 0.0, OHM),
    ("MΩ Mohm", 1e6, 0.0, OHM),
    // C and F stay Celsius and Fahrenheit.
    ("coulomb coulombs", 1.0, 0.0, COULOMB),
    ("mAh", 3.6, 0.0, COULOMB),
    ("Ah", 3600.0, 0.0, COULOMB),
    ("farad farads", 1.0, 0.0, FARAD),
    ("mF", 1e-3, 0.0, FARAD),
    ("µF uF μF", 1e-6, 0.0, FARAD),
    ("nF", 1e-9, 0.0, FARAD),
    ("pF", 1e-12, 0.0, FARAD),
    ("S siemens", 1.0, 0.0, SIEMENS),
    ("mS", 1e-3, 0.0, SIEMENS),
    ("Wb weber webers", 1.0, 0.0, WEBER),
    ("mWb", 1e-3, 0.0, WEBER),
    ("T tesla teslas", 1.0, 0.0, TESLA),
    ("mT", 1e-3, 0.0, TESLA),
    ("µT uT μT", 1e-6, 0.0, TESLA),
    ("H henry henries", 1.0, 0.0, HENRY),
    ("mH", 1e-3, 0.0, HENRY),
    ("µH uH μH", 1e-6, 0.0, HENRY),
    ("Hz hertz", 1.0, 0.0, PER_TIME),
    ("kHz", 1e3, 0.0, PER_TIME),
    ("MHz", 1e6, 0.0, PER_TIME),
    ("GHz", 1e9, 0.0, PER_TIME),
    ("rpm", 1.0 / 60.0, 0.0, PER_TIME),
    ("mol mole moles", 1.0, 0.0, AMOUNT),
    ("mmol", 1e-3, 0.0, AMOUNT),
    ("µmol umol μmol", 1e-6, 0.0, AMOUNT),
    ("kmol", 1e3, 0.0, AMOUNT),
    ("M molar", 1e3, 0.0, MOLAR),
    ("mM millimolar", 1.0, 0.0, MOLAR),
    ("µM uM μM micromolar", 1e-3, 0.0, MOLAR),
    // Steradians are dimensionless in SI, so lumens and candelas share a dimension.
    ("lm lumen lumens", 1.0, 0.0, LIGHT),
    ("cd candela candelas", 1.0, 0.0, LIGHT),
    ("lx lux", 1.0, 0.0, LUX),
    ("fc footcandle footcandles", 10.763910416709722, 0.0, LUX),
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
    for (names, scale, offset, dim) in rows() {
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
