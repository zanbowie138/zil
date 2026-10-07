//! Units: quantities with dimensions, arithmetic and conversion between them, live currency rates.

use super::{Claim, Module};
use crate::ast::{BinOp, Target, UnitSpec};
use crate::interp::mismatch;
use crate::value::{Value, num};
use std::cell::RefCell;
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
        ("conversions", &[("to unit", "5 km to mi"), ("to unit unit", "1.8 m to ft in"), ("to compound", "100 km / 2 h to mph")]),
    ],
    ident: Some(|_, name| match unit(name) {
        Ok(u) => Some(Ok(Value::Qty(1.0, u))),
        Err(e) if e.starts_with("unknown unit") => None,
        Err(e) => Some(Err(e)),
    }),
    binary: Some(binary),
    compare: Some(compare),
    convert: Some(convert),
    topic: Some(topic),
    ..Module::EMPTY
};

fn binary(op: BinOp, a: &Value, b: &Value) -> Claim {
    use Value::*;
    Some(Ok(match (op, a, b) {
        (BinOp::Add | BinOp::Sub, Qty(x, u), Qty(y, w)) => {
            if u.dim() != w.dim() {
                return Some(Err(format!("cannot add {u} and {w}")));
            }
            let y = u.value_from_si(w.to_si(*y));
            Value::qty(if op == BinOp::Add { x + y } else { x - y }, u.clone())
        }
        (BinOp::Mul | BinOp::Div, Qty(..), _) | (BinOp::Mul | BinOp::Div, _, Qty(..)) => {
            let split = |v: &Value| match v {
                Qty(x, u) => Some((*x, u.clone())),
                _ => Some((num(v)?, Unit::default())),
            };
            let (Some((x, u)), Some((y, w))) = (split(a), split(b)) else {
                return Some(Err(mismatch(a, b)));
            };
            if op == BinOp::Mul { Value::qty(x * y, u.mul(&w, 1)) } else { Value::qty(x / y, u.mul(&w, -1)) }
        }
        (BinOp::Pow, Qty(x, u), Int(n, _)) if (-9..=9).contains(n) => Value::qty(x.powi(*n as i32), u.pow(*n as i8)),
        _ => return None,
    }))
}

fn compare(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::Qty(x, u), Value::Qty(y, w)) if u.dim() == w.dim() => u.to_si(*x).partial_cmp(&w.to_si(*y)),
        _ => None,
    }
}

/// `to km`, and `to ft in` as a mixed-unit string.
fn convert(v: &Value, t: &Target) -> Claim {
    let r = match (v, t) {
        (Value::Qty(x, u), Target::Units(specs)) => {
            specs.iter().map(unit_of).collect::<Result<Vec<_>, _>>().and_then(|us| match us.iter().find(|t| t.dim() != u.dim()) {
                Some(t) => Err(format!("cannot convert {u} to {t}")),
                None => Ok(Value::str(split(u.to_si(*x), &us))),
            })
        }
        (Value::Qty(x, u), Target::Unit(spec)) => unit_of(spec).and_then(|t| {
            if u.dim() != t.dim() {
                return Err(format!("cannot convert {u} to {t}"));
            }
            Ok(Value::Qty(t.value_from_si(u.to_si(*x)), t))
        }),
        (v, Target::Unit(_) | Target::Units(_)) if num(v).is_some() => Err(format!("{v} has no unit; attach one like `{v} km`")),
        _ => return None,
    };
    Some(r)
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
    use crate::help::show;
    if topic == "units" {
        println!("\nunits");
        for (kind, _) in DIMS {
            println!("  {kind:<12} {}", units_of(kind).join(" "));
        }
        println!("  {:<12} 3-letter codes (USD EUR GBP ...), live rates fetched on first use", "currency");
        println!("help(\"length\") or help(\"km\") for details");
    } else if matches!(topic, "currency" | "money") {
        println!("currency: 3-letter codes like USD, EUR, GBP, JPY. Rates come from frankfurter.dev,");
        println!("are fetched on first use (with a prompt) and cached for a day.");
        println!("  100 USD to EUR");
    } else if let Some(kind) = DIMS.iter().map(|d| d.0).find(|k| *k == topic) {
        let units = units_of(kind);
        println!("{kind} units: {}", units.join(" "));
        show(vec![format!("1 {} to {}", units[1], units[0]), format!("1 {} to {}", units[0], units[units.len() - 1])]);
    } else if let Some((names, _, _, dim)) = TABLE.iter().find(|row| row.0.split(' ').any(|n| n == topic)) {
        let mut names = names.split(' ');
        let name = names.next().unwrap();
        let kind = DIMS.iter().find(|d| d.1 == *dim).unwrap().0;
        let aliases: Vec<_> = names.collect();
        let aka = if aliases.is_empty() { String::new() } else { format!("  (also {})", aliases.join(", ")) };
        println!("{name}: {kind}{aka}");
        let other = units_of(kind).into_iter().find(|u| *u != name).unwrap();
        show(vec![format!("1 {name} to {other}"), format!("1 {other} to {name}")]);
        println!("all {kind} units: help(\"{kind}\")");
    } else {
        return false;
    }
    true
}

fn units_of(kind: &str) -> Vec<&'static str> {
    let dim = DIMS.iter().find(|d| d.0 == kind).unwrap().1;
    TABLE.iter().filter(|row| row.3 == dim).map(|row| row.0.split(' ').next().unwrap()).collect()
}

/// Exponents of the base dimensions: length, mass, time, temperature, data, money, angle.
pub type Dim = [i8; 7];

const fn d(l: i8, m: i8, t: i8) -> Dim {
    [l, m, t, 0, 0, 0, 0]
}
const LEN: Dim = d(1, 0, 0);
const MASS: Dim = d(0, 1, 0);
const TIME: Dim = d(0, 0, 1);
const TEMP: Dim = [0, 0, 0, 1, 0, 0, 0];
const DATA: Dim = [0, 0, 0, 0, 1, 0, 0];
const RATE: Dim = [0, 0, -1, 0, 1, 0, 0];
const MONEY: Dim = [0, 0, 0, 0, 0, 1, 0];
const ANGLE: Dim = [0, 0, 0, 0, 0, 0, 1];

/// Names of the kinds of unit in TABLE, for `help`.
#[rustfmt::skip]
pub const DIMS: &[(&str, Dim)] = &[
    ("length", LEN), ("mass", MASS), ("time", TIME), ("temperature", TEMP),
    ("volume", d(3, 0, 0)), ("area", d(2, 0, 0)), ("speed", d(1, 0, -1)),
    ("data", DATA), ("rate", RATE), ("energy", d(2, 1, -2)), ("power", d(2, 1, -3)),
    ("pressure", d(-1, 1, -2)), ("force", d(1, 1, -2)), ("angle", ANGLE),
];

/// Space-separated names (first is the display name), factor to SI, offset (temperatures only), dimension.
/// SI bases: m, kg, s, K, bit, EUR, rad.
#[rustfmt::skip]
pub const TABLE: &[(&str, f64, f64, Dim)] = &[
    ("m meter meters metre metres", 1.0, 0.0, LEN),
    ("km kilometer kilometers", 1e3, 0.0, LEN),
    ("cm centimeter centimeters", 1e-2, 0.0, LEN),
    ("mm millimeter millimeters", 1e-3, 0.0, LEN),
    ("um micrometer micrometers micron", 1e-6, 0.0, LEN),
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
    ("ug microgram micrograms", 1e-9, 0.0, MASS),
    ("t tonne tonnes", 1e3, 0.0, MASS),
    ("lb lbs pound pounds", 0.45359237, 0.0, MASS),
    ("oz ounce ounces", 0.028349523125, 0.0, MASS),
    ("st stone", 6.35029318, 0.0, MASS),

    ("s sec secs second seconds", 1.0, 0.0, TIME),
    ("ms millisecond milliseconds", 1e-3, 0.0, TIME),
    ("us microsecond microseconds", 1e-6, 0.0, TIME),
    ("ns nanosecond nanoseconds", 1e-9, 0.0, TIME),
    ("min mins minute minutes", 60.0, 0.0, TIME),
    ("h hr hrs hour hours", 3600.0, 0.0, TIME),
    ("d day days", 86400.0, 0.0, TIME),
    ("wk week weeks", 604800.0, 0.0, TIME),
    ("mo month months", 2629746.0, 0.0, TIME),
    ("yr year years", 31556952.0, 0.0, TIME),

    ("K kelvin", 1.0, 0.0, TEMP),
    ("C celsius degC", 1.0, 273.15, TEMP),
    ("F fahrenheit degF", 5.0 / 9.0, 459.67, TEMP),

    ("L l liter liters litre litres", 1e-3, 0.0, d(3, 0, 0)),
    ("mL ml milliliter milliliters", 1e-6, 0.0, d(3, 0, 0)),
    ("gal gallon gallons", 3.785411784e-3, 0.0, d(3, 0, 0)),
    ("qt quart quarts", 9.46352946e-4, 0.0, d(3, 0, 0)),
    ("pt pint pints", 4.73176473e-4, 0.0, d(3, 0, 0)),
    ("cup cups", 2.365882365e-4, 0.0, d(3, 0, 0)),
    ("floz", 2.95735295625e-5, 0.0, d(3, 0, 0)),
    ("tbsp", 1.478676478125e-5, 0.0, d(3, 0, 0)),
    ("tsp", 4.92892159375e-6, 0.0, d(3, 0, 0)),

    ("ha hectare hectares", 1e4, 0.0, d(2, 0, 0)),
    ("acre acres", 4046.8564224, 0.0, d(2, 0, 0)),

    ("kph kmh", 1.0 / 3.6, 0.0, d(1, 0, -1)),
    ("mph", 0.44704, 0.0, d(1, 0, -1)),
    ("kn knot knots", 1852.0 / 3600.0, 0.0, d(1, 0, -1)),

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

    ("J joule joules", 1.0, 0.0, d(2, 1, -2)),
    ("kJ", 1e3, 0.0, d(2, 1, -2)),
    ("MJ", 1e6, 0.0, d(2, 1, -2)),
    ("cal calorie calories", 4.184, 0.0, d(2, 1, -2)),
    ("kcal Cal", 4184.0, 0.0, d(2, 1, -2)),
    ("Wh", 3600.0, 0.0, d(2, 1, -2)),
    ("kWh", 3.6e6, 0.0, d(2, 1, -2)),
    ("eV", 1.602176634e-19, 0.0, d(2, 1, -2)),
    ("BTU btu", 1055.05585262, 0.0, d(2, 1, -2)),

    ("W watt watts", 1.0, 0.0, d(2, 1, -3)),
    ("kW", 1e3, 0.0, d(2, 1, -3)),
    ("MW", 1e6, 0.0, d(2, 1, -3)),
    ("hp horsepower", 745.699_871_582_270_2, 0.0, d(2, 1, -3)),

    ("Pa pascal", 1.0, 0.0, d(-1, 1, -2)),
    ("kPa", 1e3, 0.0, d(-1, 1, -2)),
    ("MPa", 1e6, 0.0, d(-1, 1, -2)),
    ("bar", 1e5, 0.0, d(-1, 1, -2)),
    ("atm", 101325.0, 0.0, d(-1, 1, -2)),
    ("psi", 6894.757293168, 0.0, d(-1, 1, -2)),
    ("mmHg", 133.322387415, 0.0, d(-1, 1, -2)),

    ("N newton newtons", 1.0, 0.0, d(1, 1, -2)),
    ("kN", 1e3, 0.0, d(1, 1, -2)),
    ("lbf", 4.4482216152605, 0.0, d(1, 1, -2)),

    ("rad radian radians", 1.0, 0.0, ANGLE),
    ("deg degree degrees", PI / 180.0, 0.0, ANGLE),
    ("turn turns", 2.0 * PI, 0.0, ANGLE),
];

#[derive(Debug)]
pub struct UnitDef {
    pub name: String,
    pub scale: f64,
    pub offset: f64,
    pub dim: Dim,
}

/// A product of units with integer powers, e.g. km/h = [(km, 1), (h, -1)].
#[derive(Debug, Clone, Default)]
pub struct Unit(pub Vec<(Rc<UnitDef>, i8)>);

impl Unit {
    pub fn dim(&self) -> Dim {
        let mut dim = [0; 7];
        for (u, p) in &self.0 {
            for (acc, x) in dim.iter_mut().zip(u.dim) {
                *acc += x * p;
            }
        }
        dim
    }

    pub fn scale(&self) -> f64 {
        self.0.iter().map(|(u, p)| u.scale.powi(*p as i32)).product()
    }

    /// Offsets only apply to a lone unit like `C`, not inside `C/s`.
    fn offset(&self) -> f64 {
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
        let term = |u: &UnitDef, p: i8| {
            if p == 1 { u.name.clone() } else { format!("{}^{p}", u.name) }
        };
        let num: Vec<_> = self.0.iter().filter(|(_, p)| *p > 0).map(|(u, p)| term(u, *p)).collect();
        let den: Vec<_> = self.0.iter().filter(|(_, p)| *p < 0).map(|(u, p)| term(u, -p)).collect();
        match (num.is_empty(), den.is_empty()) {
            (_, true) => write!(f, "{}", num.join("*")),
            (true, false) => write!(f, "1/{}", den.join("/")),
            _ => write!(f, "{}/{}", num.join("*"), den.join("/")),
        }
    }
}

thread_local! {
    static REGISTRY: RefCell<HashMap<String, Rc<UnitDef>>> = RefCell::new(builtin_units());
    static RATES_LOADED: RefCell<bool> = const { RefCell::new(false) };
}

fn builtin_units() -> HashMap<String, Rc<UnitDef>> {
    let mut map = HashMap::new();
    for (names, scale, offset, dim) in TABLE {
        let primary = names.split(' ').next().unwrap();
        let def = Rc::new(UnitDef { name: primary.into(), scale: *scale, offset: *offset, dim: *dim });
        for n in names.split(' ') {
            map.insert(n.to_string(), def.clone());
        }
    }
    map
}

pub fn lookup(name: &str) -> Result<Rc<UnitDef>, String> {
    if let Some(u) = REGISTRY.with(|r| r.borrow().get(name).cloned()) {
        return Ok(u);
    }
    let looks_like_currency = name.len() == 3 && name.bytes().all(|b| b.is_ascii_alphabetic());
    if looks_like_currency && !RATES_LOADED.with(|l| l.replace(true)) {
        let rates = load_rates().map_err(|e| format!("cannot load currency rates: {e}"))?;
        REGISTRY.with(|r| {
            let mut r = r.borrow_mut();
            for (code, per_eur) in rates {
                let def = Rc::new(UnitDef { name: code.clone(), scale: 1.0 / per_eur, offset: 0.0, dim: MONEY });
                r.entry(code.to_lowercase()).or_insert(def.clone());
                r.insert(code, def);
            }
        });
        return lookup(name);
    }
    Err(format!("unknown unit `{name}`"))
}

pub fn unit(name: &str) -> Result<Unit, String> {
    Ok(Unit(vec![(lookup(name)?, 1)]))
}

/// Asks on the terminal before going online; non-interactive runs never fetch.
fn confirm_fetch() -> Result<(), String> {
    use std::io::{IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        return Err("rates not cached and stdin is not a terminal to confirm a fetch".into());
    }
    eprint!("Fetch currency rates from api.frankfurter.dev? [y/N] ");
    let _ = std::io::stderr().flush();
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer).map_err(|e| e.to_string())?;
    match answer.trim() {
        "y" | "Y" | "yes" => Ok(()),
        _ => Err("fetch declined".into()),
    }
}

/// Units per 1 EUR from frankfurter.dev (ECB data), cached on disk for a day.
fn load_rates() -> Result<Vec<(String, f64)>, String> {
    let cache = crate::cache_dir().map(|d| d.join("rates.json"));
    let fresh = cache.as_ref().and_then(|p| p.metadata().ok()?.modified().ok()?.elapsed().ok()).is_some_and(|age| age.as_secs() < 24 * 3600);
    let body = match &cache {
        Some(p) if fresh => std::fs::read_to_string(p).map_err(|e| e.to_string())?,
        _ => match confirm_fetch()
            .and_then(|()| ureq::get("https://api.frankfurter.dev/v1/latest").call().and_then(|mut r| r.body_mut().read_to_string()).map_err(|e| e.to_string()))
        {
            Ok(body) => {
                if let Some(p) = &cache {
                    let _ = std::fs::create_dir_all(p.parent().unwrap());
                    let _ = std::fs::write(p, &body);
                }
                body
            }
            // Offline: fall back to a stale cache if there is one.
            Err(e) => match cache.as_ref().and_then(|p| std::fs::read_to_string(p).ok()) {
                Some(body) => body,
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
        assert!(try_eval("5 km + 1 kg").is_err());
        assert!(try_eval("5 km to kg").is_err());
    }
}
