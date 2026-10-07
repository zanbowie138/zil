//! Numbers: digits, rounding, logs, trigonometry (angles as units), formatting, number theory, bits.

use super::{Call, Claim, Doc, Fail, Module, doc, units};
use crate::ast::{Radix, Target};
use crate::interp::Interp;
use crate::lexer::Span;
use crate::value::{Value, compare, exact, fits, fmt_float, mask, num, ratio};
use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};
use std::rc::Rc;

pub const MODULE: Module = Module {
    name: "math",
    about: "digits, rounding, logs, trigonometry (angles as units), formatting, number theory, bits",
    #[rustfmt::skip]
    examples: &[
        ("numbers", &[
            ("exact fractions", r#"(1..=6).map(\x -> 1/x).sum to frac"#),
            ("decimals are exact", "0.1 + 0.2 == 0.3"),
            ("1/3 * 3 is exactly 1", "1/3 * 3 == 1"),
            ("decimal to fraction", "0.75 to frac"),
            ("big ints never overflow", "2 ** 100"),
            ("prime factors", "factors(2 ** 32 + 1)"),
            ("is a Mersenne number prime", "is_prime(2 ** 61 - 1)"),
            ("poker hands", "choose(52, 5)"),
            ("modular power", "mod_pow(3, 1000, 7)"),
            ("add a percent", "80 + 15%"),
            ("percent of", "15% of 64.50"),
            ("trig takes angle units", "sin(30 deg)"),
            ("radians back to degrees", "atan2(1, 1) to deg"),
            ("digit sum", "(2 ** 100).digits.sum"),
            ("format specs", r#""{1234567.891:,.2f}""#),
        ]),
        ("bits", &[
            ("see the bits", "0xf0 to bits"),
            ("results keep their base", "0xff + 1"),
            ("two's complement", "-1 to hex(16)"),
            ("xor", "0b1010 ^ 0b0110"),
            ("count set bits", "popcount(0xff)"),
            ("rotate within 8 bits", "rotr(1, 1, 8)"),
            ("swap byte order", "byteswap(0x1234, 16)"),
            ("clear a bit", "clear_bit(0xff, 0)"),
            ("hex with a prefix", r#""{255:#06x}""#),
            ("32-bit binary", r#""{0xdeadbeef:032b}""#),
            ("base 36", "255 to base(36)"),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[
        ("constants", &[("pi e tau phi inf nan", "")]),
        ("operators", &[("n%", "20%"), ("x ± n%", "50 + 10%"), ("n% of x", "20% of 50")]),
        ("format specs", &[
            (r#""{x:.2f}"  fixed decimals"#, r#""{pi:.2f}""#),
            (r#""{x:>8}"  width, align < > ^"#, r#""[{42:>8}]""#),
            (r#""{n:08x}"  zero pad; x X b o d"#, r#""{255:08x}""#),
            (r#""{n:#06x}"  # adds 0x 0b 0o"#, r#""{255:#06x}""#),
            (r#""{x:,}"  commas; also + e %"#, r#""{1234567:,}""#),
            (r#"format("%5.2f", x)  printf"#, r#"format("%5.2f|%-4d|", pi, 7)"#),
        ]),
        ("conversions", &[("to bits", "0xf0 to bits")]),
    ],
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("rounding", &["abs", "round", "floor", "ceil", "trunc", "sign", "clamp"]),
        ("powers", &["sqrt", "cbrt", "exp", "ln", "log", "hypot", "is_nan"]),
        ("trig", &["sin", "cos", "tan", "asin", "acos", "atan", "atan2", "sinh", "cosh", "tanh"]),
        ("format", &["fixed", "sci", "percent", "commas"]),
        ("integers", &["gcd", "lcm", "is_prime", "factors", "factorial", "choose", "mod_pow"]),
        ("bits", &["popcount", "bit", "set_bit", "clear_bit", "rotl", "rotr", "byteswap"]),
        ("digits", &["digits", "from_digits"]),
    ],
    call,
    #[rustfmt::skip]
    consts: &[
        ("pi", std::f64::consts::PI), ("e", std::f64::consts::E), ("tau", std::f64::consts::TAU),
        ("phi", 1.618033988749895), ("inf", f64::INFINITY), ("nan", f64::NAN),
    ],
    convert: Some(convert),
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("digits", "digits(n)", "list of digits in the number's own base", &["1234.digits", "0b1011.digits"], &["from_digits"]),
    doc("from_digits", "from_digits(list, base?)", "build a number from digits, kept in that base", &["[1, 2, 3].from_digits", "[1, 0, 1, 1].from_digits(2)"], &["digits"]),
    doc("sqrt", "sqrt(x)", "square root", &["sqrt(2)"], &["cbrt"]),
    doc("abs", "abs(x)", "absolute value; keeps units", &["abs(-3)", "abs(-2 km)"], &["round", "sign"]),
    doc("round", "round(x, digits?) / round(x, step)", "round to nearest; keeps units. A non-int second arg rounds to a multiple of it", &["round(pi, 2)", "round(2.5 km)", "round(7.3, 0.25)", "round(17 min, 15 min)"], &["floor", "ceil", "trunc"]),
    doc("floor", "floor(x)", "round down", &["floor(2.7)"], &["ceil", "round"]),
    doc("ceil", "ceil(x)", "round up", &["ceil(2.1)"], &["floor", "round"]),
    doc("trunc", "trunc(x)", "round toward zero; keeps units", &["trunc(-2.7)"], &["floor", "round"]),
    doc("sign", "sign(x)", "-1, 0 or 1", &["sign(-5 km)", "sign(0)"], &["abs"]),
    doc("clamp", "clamp(x, lo, hi)", "limit x to [lo, hi]", &["clamp(15, 0, 10)", "clamp(5 m, 1 m, 2 m)"], &["min", "max"]),
    doc("cbrt", "cbrt(x)", "cube root", &["cbrt(27)"], &["sqrt"]),
    doc("exp", "exp(x)", "e to the power x", &["exp(1)"], &["ln"]),
    doc("ln", "ln(x)", "natural log", &["ln(e)"], &["log", "exp"]),
    doc("log", "log(x, base?)", "log base 10, or another base", &["log(1000)", "log(8, 2)"], &["ln"]),
    doc("hypot", "hypot(x, y)", "sqrt(x² + y²) without overflow; keeps units", &["hypot(3, 4)", "hypot(3 m, 4 m)"], &["sqrt", "atan2"]),
    doc("is_nan", "is_nan(x)", "whether x is nan (nan != nan)", &["is_nan(nan)", "is_nan(inf - inf)"], &[]),
    doc("sin", "sin(x)", "sine of radians or an angle unit", &["sin(30 deg)", "sin(pi / 2)"], &["cos", "tan", "asin"]),
    doc("cos", "cos(x)", "cosine of radians or an angle unit", &["cos(60 deg)"], &["sin", "tan", "acos"]),
    doc("tan", "tan(x)", "tangent of radians or an angle unit", &["tan(45 deg)"], &["sin", "cos", "atan"]),
    doc("asin", "asin(x)", "arcsine, as an angle", &["asin(1) to deg"], &["sin"]),
    doc("acos", "acos(x)", "arccosine, as an angle", &["acos(0) to deg"], &["cos"]),
    doc("atan", "atan(x)", "arctangent, as an angle", &["atan(1) to deg"], &["tan", "atan2"]),
    doc("atan2", "atan2(y, x)", "angle of the point (x, y), as an angle", &["atan2(1, -1) to deg"], &["atan", "hypot"]),
    doc("sinh", "sinh(x)", "hyperbolic sine", &["sinh(1)"], &["cosh", "tanh"]),
    doc("cosh", "cosh(x)", "hyperbolic cosine", &["cosh(1)"], &["sinh", "tanh"]),
    doc("tanh", "tanh(x)", "hyperbolic tangent", &["tanh(1)"], &["sinh", "cosh"]),
    doc("fixed", "fixed(x, digits)", "string with exactly that many decimals; keeps units", &["pi.fixed(2)", "(5 km to mi).fixed(1)"], &["sci", "commas", "round"]),
    doc("sci", "sci(x, digits?)", "string in scientific notation", &["123456.sci", "123456.sci(2)"], &["fixed"]),
    doc("percent", "percent(x, digits?)", "string as a percentage", &["0.256.percent", "(1/3).percent(1)"], &["fixed"]),
    doc("commas", "commas(x, digits?)", "string with thousands separators", &["1234567.commas", "1234.5.commas(2)"], &["fixed"]),
    doc("gcd", "gcd(a, b, ...) / gcd(list)", "greatest common divisor", &["gcd(12, 18)", "[12, 18, 27].gcd"], &["lcm"]),
    doc("lcm", "lcm(a, b, ...) / lcm(list)", "least common multiple", &["lcm(4, 6)", "(1..=20).lcm"], &["gcd"]),
    doc("is_prime", "is_prime(n)", "primality (Miller-Rabin; exact below 3e24)", &["is_prime(97)", "is_prime(2 ** 61 - 1)"], &["factors"]),
    doc("factors", "factors(n)", "prime factors, smallest first", &["360.factors", "factors(2 ** 32 + 1)"], &["is_prime", "gcd"]),
    doc("factorial", "factorial(n)", "n!, exact", &["factorial(5)", "factorial(30)"], &["choose"]),
    doc("choose", "choose(n, k)", "ways to pick k of n, exact", &["choose(5, 2)", "choose(52, 5)"], &["factorial"]),
    doc("mod_pow", "mod_pow(b, e, m)", "b ** e % m without the huge power", &["mod_pow(2, 100, 7)", "mod_pow(3, 10 ** 18, 1000000007)"], &["gcd"]),
    doc("popcount", "popcount(n)", "number of 1 bits (two's complement for negatives)", &["popcount(0b1011)", "popcount(-1)"], &["bit"]),
    doc("bit", "bit(n, i)", "bit i of n (0 is the lowest), as 0 or 1", &["bit(0b100, 2)"], &["set_bit", "clear_bit", "popcount"]),
    doc("set_bit", "set_bit(n, i)", "n with bit i set", &["set_bit(0b1, 4)"], &["clear_bit", "bit"]),
    doc("clear_bit", "clear_bit(n, i)", "n with bit i cleared", &["clear_bit(0xff, 0)"], &["set_bit", "bit"]),
    doc("rotl", "rotl(n, k, width)", "rotate the low width bits left by k", &["rotl(0x81, 1, 8)", "rotl(0x80000000, 1, 32)"], &["rotr"]),
    doc("rotr", "rotr(n, k, width)", "rotate the low width bits right by k", &["rotr(0x81, 1, 8)"], &["rotl"]),
    doc("byteswap", "byteswap(n, width)", "reverse the bytes of a width-bit number", &["byteswap(0x1234, 16)", "byteswap(0x12345678, 32)"], &["rotl"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
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
        ("digits", [Big(n, r)]) => Value::list(n.magnitude().to_radix_be(r.base).into_iter().map(|d| Value::int(d as i64)).collect()),
        ("from_digits", [List(l)]) => from_digits(&l.borrow(), 10)?,
        ("from_digits", [List(l), Int(b, _)]) if (2..=36).contains(b) => from_digits(&l.borrow(), *b as u32)?,
        ("sqrt", [v]) if num(v).is_some() => Float(num(v).unwrap().sqrt()),
        ("abs" | "round" | "floor" | "ceil" | "trunc", [v]) => round_like(name, v, 0).ok_or(Fail::BadArgs)?,
        ("round", [v, Int(n, _)]) => round_like(name, v, *n).ok_or(Fail::BadArgs)?,
        ("round", [v, step]) => round_to(v, step).ok_or(Fail::BadArgs)?,
        ("sign", [v]) => {
            let x = match v {
                Qty(x, _) => *x,
                _ => num(v).ok_or(Fail::BadArgs)?,
            };
            if x.is_nan() {
                Float(x)
            } else {
                Value::int(if x > 0.0 {
                    1
                } else if x < 0.0 {
                    -1
                } else {
                    0
                })
            }
        }
        ("clamp", [v, lo, hi]) => {
            let cmp = |a: &Value, b: &Value| compare(a, b).ok_or_else(|| format!("cannot compare {a:?} and {b:?}"));
            if cmp(lo, hi)?.is_gt() {
                return Err("lo is greater than hi".into());
            }
            if cmp(v, lo)?.is_lt() {
                lo.clone()
            } else if cmp(v, hi)?.is_gt() {
                hi.clone()
            } else {
                v.clone()
            }
        }
        ("cbrt" | "exp" | "sinh" | "cosh" | "tanh", [v]) if num(v).is_some() => {
            let x = num(v).unwrap();
            Float(match name {
                "cbrt" => x.cbrt(),
                "exp" => x.exp(),
                "sinh" => x.sinh(),
                "cosh" => x.cosh(),
                _ => x.tanh(),
            })
        }
        ("is_nan", [v]) => Bool(matches!(v, Float(x) | Qty(x, _) if x.is_nan())),
        ("ln", [v]) if num(v).is_some() => Float(num(v).unwrap().ln()),
        ("log", [v]) if num(v).is_some() => Float(num(v).unwrap().log10()),
        ("log", [v, b]) if num(v).is_some() && num(b).is_some() => Float(num(v).unwrap().log(num(b).unwrap())),
        ("hypot", [Qty(x, u), Qty(y, w)]) if u.dim() == w.dim() => Qty(x.hypot(u.value_from_si(w.to_si(*y))), u.clone()),
        ("hypot", [x, y]) if num(x).is_some() && num(y).is_some() => Float(num(x).unwrap().hypot(num(y).unwrap())),
        ("sin" | "cos" | "tan", [v]) => {
            let rad = match v {
                Qty(x, u) if u.dim() == units::unit("rad").unwrap().dim() => u.to_si(*x),
                _ => num(v).ok_or(Fail::BadArgs)?,
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
            Qty(rad, units::unit("rad").unwrap())
        }
        ("atan2", [y, x]) => {
            let (y, x) = match (y, x) {
                (Qty(y, u), Qty(x, w)) if u.dim() == w.dim() => (u.to_si(*y), w.to_si(*x)),
                _ => (num(y).ok_or(Fail::BadArgs)?, num(x).ok_or(Fail::BadArgs)?),
            };
            Qty(y.atan2(x), units::unit("rad").unwrap())
        }
        ("fixed" | "sci" | "percent" | "commas", [v, rest @ ..]) if rest.len() <= 1 && (num(v).is_some() || matches!(v, Qty(..))) => {
            let digits = match rest {
                [] if name != "fixed" => None,
                [Int(d, _)] if (0..=100).contains(d) => Some(*d as usize),
                _ => return Err(Fail::BadArgs),
            };
            let kind = match name {
                "sci" => Some('e'),
                "percent" => Some('%'),
                _ => digits.map(|_| 'f'),
            };
            Value::str(render(v, &Spec { kind, prec: digits, commas: name == "commas", ..Spec::PLAIN })?)
        }
        ("gcd" | "lcm", [List(l)]) => gcd_lcm(name, &l.borrow())?,
        ("gcd" | "lcm", vs) if vs.len() >= 2 => gcd_lcm(name, vs)?,
        ("is_prime", [v]) => Bool(is_prime(&int(v)?)),
        ("factors", [v]) => {
            let mut n = int(v)?;
            if !n.is_positive() {
                return Err("expects a positive integer".into());
            }
            let mut out = Vec::new();
            for p in [2u32, 3, 5] {
                while (&n % p).is_zero() {
                    n /= p;
                    out.push(BigInt::from(p));
                }
            }
            factor(n, &mut out);
            out.sort();
            Value::list(out.into_iter().map(big).collect())
        }
        ("factorial", [Int(n, _)]) if (0..=20_000).contains(n) => big((1..=*n).map(BigInt::from).product()),
        ("factorial", [Int(..)]) => return Err("n must be 0-20000".into()),
        ("choose", [Int(n, _), Int(k, _)]) if *n >= 0 && *k >= 0 => {
            if k > n {
                return Ok(Value::int(0));
            }
            let k = *k.min(&(n - k));
            if k > 100_000 {
                return Err("result too large".into());
            }
            let mut r = BigInt::one();
            for i in 0..k {
                r = r * (n - i) / (i + 1);
            }
            big(r)
        }
        ("mod_pow", [b, e, m]) => {
            let (b, e, m) = (int(b)?, int(e)?, int(m)?);
            if !m.is_positive() || e.is_negative() {
                return Err("needs e >= 0 and m > 0".into());
            }
            big(b.modpow(&e, &m))
        }
        ("popcount", [Int(n, _)]) => Value::int(n.count_ones() as i64),
        ("popcount", [Big(n, _)]) if !n.is_negative() => Value::int(n.magnitude().count_ones() as i64),
        ("popcount", [Big(..)]) => return Err("negative big ints have infinitely many 1 bits".into()),
        // Through BigInt so bits past 63 work; two's complement, like the i64 ops.
        ("bit", [n @ (Int(..) | Big(..)), Int(i, _)]) if *i >= 0 => Value::int(int(n)?.bit(*i as u64) as i64),
        ("set_bit" | "clear_bit", [n @ (Int(_, r) | Big(_, r)), Int(i, _)]) if *i >= 0 => {
            let mut b = int(n)?;
            b.set_bit(*i as u64, name == "set_bit");
            exact(BigRational::from_integer(b), *r, false)
        }
        ("rotl" | "rotr", [Int(n, r), Int(k, _), Int(w, _)]) => {
            let (m, w) = low_bits(*n, *w)?;
            let k = k.rem_euclid(w as i64) as u32;
            let k = if name == "rotr" { (w - k) % w } else { k };
            let v = if k == 0 { m } else { (m << k | m >> (w - k)) & mask(w) };
            unsigned(v, *r)
        }
        ("byteswap", [Int(n, r), Int(w, _)]) if w % 8 == 0 => {
            let (m, w) = low_bits(*n, *w)?;
            unsigned(m.swap_bytes() >> (64 - w), *r)
        }
        _ => return Err(Fail::BadArgs),
    })
}

/// abs/round/floor/ceil/trunc on numbers and quantities (keeping the unit).
fn round_like(name: &str, v: &Value, digits: i64) -> Option<Value> {
    let f = |x: f64| -> f64 {
        let p = 10f64.powi(digits as i32);
        match name {
            "abs" => x.abs(),
            "round" => (x * p).round() / p,
            "floor" => x.floor(),
            "trunc" => x.trunc(),
            _ => x.ceil(),
        }
    };
    Some(match v {
        Value::Int(n, b) if name == "abs" => Value::Int(n.checked_abs()?, *b),
        Value::Int(..) if digits >= 0 => v.clone(),
        Value::Big(n, b) if name == "abs" => exact(BigRational::from_integer(n.abs()), *b, false),
        Value::Big(..) if digits >= 0 => v.clone(),
        Value::Frac(r, f) if name == "abs" => Value::Frac(Rc::new(r.abs()), *f),
        Value::Frac(r, _) if digits == 0 => exact(
            match name {
                "round" => r.round(),
                "floor" => r.floor(),
                "trunc" => r.trunc(),
                _ => r.ceil(),
            },
            Radix::DEC,
            false,
        ),
        Value::Frac(..) => Value::Float(f(num(v)?)),
        Value::Float(x) if digits == 0 && name != "abs" && f(*x).abs() < 9.2e18 => Value::int(f(*x) as i64),
        Value::Float(x) => Value::Float(f(*x)),
        Value::Qty(x, u) => Value::Qty(f(*x), u.clone()),
        _ => return None,
    })
}

/// `round(x, step)`: the nearest multiple of `step`, exact for ints and fractions.
fn round_to(v: &Value, step: &Value) -> Option<Value> {
    if let (Some(x), Some(s)) = (ratio(v), ratio(step)) {
        return (!s.is_zero()).then(|| exact((x / &s).round() * s, Radix::DEC, false));
    }
    Some(match (v, step) {
        (Value::Qty(x, u), Value::Qty(s, w)) if u.dim() == w.dim() => {
            let s = u.value_from_si(w.to_si(*s));
            Value::Qty((x / s).round() * s, u.clone())
        }
        (Value::Qty(x, u), s) => Value::Qty((x / num(s)?).round() * num(s)?, u.clone()),
        _ => Value::Float((num(v)? / num(step)?).round() * num(step)?),
    })
}

/// A format spec, as in Python: `[[fill]align][+][0][width][,][.precision][type]`, type one of
/// `f e % x X b o d s`. Serves `"{x:spec}"`, `format("%...", ...)` and fixed/sci/percent/commas.
pub struct Spec {
    fill: char,
    align: Option<char>,
    plus: bool,
    /// `#`: 0x / 0b / 0o prefix on x X b o.
    alt: bool,
    zero: bool,
    width: usize,
    commas: bool,
    prec: Option<usize>,
    kind: Option<char>,
}

impl Spec {
    const PLAIN: Spec = Spec { fill: ' ', align: None, plus: false, alt: false, zero: false, width: 0, commas: false, prec: None, kind: None };

    pub fn parse(s: &str) -> Result<Spec, String> {
        let c: Vec<char> = s.chars().collect();
        let at = |i: usize| c.get(i).copied().unwrap_or('\0');
        let (mut sp, mut i) = (Spec::PLAIN, 0);
        if matches!(at(1), '<' | '>' | '^') {
            (sp.fill, sp.align, i) = (c[0], Some(c[1]), 2);
        } else if matches!(at(0), '<' | '>' | '^') {
            (sp.align, i) = (Some(c[0]), 1);
        }
        let flag = |ch: char, i: &mut usize| {
            let hit = at(*i) == ch;
            *i += hit as usize;
            hit
        };
        let number = |i: &mut usize| {
            let start = *i;
            while at(*i).is_ascii_digit() {
                *i += 1;
            }
            c[start..*i].iter().collect::<String>().parse::<usize>().ok().filter(|n| *n <= 1000)
        };
        sp.plus = flag('+', &mut i);
        sp.alt = flag('#', &mut i);
        sp.zero = flag('0', &mut i);
        sp.width = number(&mut i).unwrap_or(0);
        sp.commas = flag(',', &mut i);
        if flag('.', &mut i) {
            sp.prec = Some(number(&mut i).ok_or_else(|| format!("bad precision in format spec {s:?}"))?);
        }
        if at(i) != '\0' && "fe%xXbods".contains(at(i)) {
            sp.kind = Some(at(i));
            i += 1;
        }
        if i != c.len() {
            return Err(format!("bad format spec {s:?}"));
        }
        Ok(sp)
    }
}

pub fn render(v: &Value, sp: &Spec) -> Result<String, String> {
    let (n, unit) = match v {
        Value::Qty(x, u) => (Value::Float(*x), Some(u)),
        _ => (v.clone(), None),
    };
    let numeric = num(&n).is_some();
    let x = || num(&n).ok_or_else(|| format!("cannot format {} as a number", v.type_name()));
    let mut s = match sp.kind {
        Some('f') => format!("{:.*}", sp.prec.unwrap_or(6), x()?),
        Some('e') => match sp.prec {
            Some(p) => format!("{:.*e}", p, x()?),
            None => format!("{:e}", x()?),
        },
        Some('%') => match sp.prec {
            Some(p) => format!("{:.*}%", p, x()? * 100.0),
            None => format!("{}%", fmt_float(x()? * 100.0)),
        },
        Some(k @ ('x' | 'X' | 'b' | 'o' | 'd')) => {
            let i = ratio(&n).filter(|r| r.is_integer()).ok_or_else(|| format!("cannot format {} as an integer", v.type_name()))?.to_integer();
            let base = match k {
                'x' | 'X' => 16,
                'b' => 2,
                'o' => 8,
                _ => 10,
            };
            let d = i.abs().to_str_radix(base);
            let d = if k == 'X' { d.to_uppercase() } else { d };
            let prefix = match k {
                _ if !sp.alt => "",
                'x' => "0x",
                'X' => "0X",
                'b' => "0b",
                'o' => "0o",
                _ => "",
            };
            format!("{}{prefix}{d}", if i.is_negative() { "-" } else { "" })
        }
        _ if numeric && sp.prec.is_some() => format!("{:.*}", sp.prec.unwrap(), x()?),
        _ => {
            let s = n.to_string();
            match sp.prec {
                Some(p) => s.chars().take(p).collect(),
                None => s,
            }
        }
    };
    if sp.commas && numeric && !matches!(sp.kind, Some('x' | 'X' | 'b' | 'o')) {
        s = commas(&s);
    }
    if sp.plus && numeric && !s.starts_with('-') {
        s.insert(0, '+');
    }
    if let Some(u) = unit {
        s = format!("{s} {u}");
    }
    let pad = sp.width.saturating_sub(s.chars().count());
    if sp.zero && sp.align.is_none() && numeric {
        // Zeros go after the sign and any `#` prefix: `{255:#06x}` is `0x00ff`.
        let prefix = 2 * (sp.alt && matches!(sp.kind, Some('x' | 'X' | 'b' | 'o'))) as usize;
        s.insert_str(s.starts_with(['-', '+']) as usize + prefix, &"0".repeat(pad));
        return Ok(s);
    }
    let fill = |k: usize| sp.fill.to_string().repeat(k);
    Ok(match sp.align.unwrap_or(if numeric { '>' } else { '<' }) {
        '<' => s + &fill(pad),
        '^' => fill(pad / 2) + &s + &fill(pad - pad / 2),
        _ => fill(pad) + &s,
    })
}

/// Thousands separators in the first run of digits: `-1234.5` -> `-1,234.5`.
fn commas(s: &str) -> String {
    let start = s.find(|c: char| c.is_ascii_digit()).unwrap_or(s.len());
    let end = s[start..].find(|c: char| !c.is_ascii_digit()).map_or(s.len(), |e| start + e);
    let d = &s[start..end];
    let mut out = String::new();
    for (i, ch) in d.chars().enumerate() {
        if i > 0 && (d.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    format!("{}{out}{}", &s[..start], &s[end..])
}

/// `format("%5.2f and %d", x, n)`: printf-style, translated to a `Spec`. Flags `- + # 0 ,`; `%%` is a literal `%`.
pub fn printf(f: &str, args: &[Value]) -> Result<String, String> {
    let (mut out, mut args, mut chars) = (String::new(), args.iter(), f.chars());
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let mut body = String::new();
        loop {
            let ch = chars.next().ok_or("unfinished % at the end of the format")?;
            body.push(ch);
            if ch.is_ascii_alphabetic() || ch == '%' {
                break;
            }
        }
        if body == "%" {
            out.push('%');
            continue;
        }
        let bad = format!("bad format %{body}");
        let kind = match body.pop().unwrap() {
            'i' | 'u' => 'd',
            k => k,
        };
        let flags: String = body.chars().take_while(|c| "-+#0,".contains(*c)).collect();
        let rest = &body[flags.len()..];
        let (width, prec) = rest.split_once('.').map_or((rest, None), |(w, p)| (w, Some(p)));
        let spec = format!(
            "{}{}{}{}{width}{}{}{kind}",
            if flags.contains('-') { "<" } else { "" },
            if flags.contains('+') { "+" } else { "" },
            if flags.contains('#') { "#" } else { "" },
            if flags.contains('0') { "0" } else { "" },
            if flags.contains(',') { "," } else { "" },
            prec.map(|p| format!(".{p}")).unwrap_or_default(),
        );
        let v = args.next().ok_or("more % specs than arguments")?;
        out += &render(v, &Spec::parse(&spec).map_err(|_| bad)?)?;
    }
    if args.next().is_some() {
        return Err("more arguments than % specs".into());
    }
    Ok(out)
}

/// `0xf0 to bits` -> `"0b1111_0000"`: binary in groups of four. `bits` is also a unit, so this hooks
/// `convert` rather than registering a target.
fn convert(v: &Value, t: &Target) -> Claim {
    let n = match t {
        Target::Unit(spec) if *spec == [("bits".to_string(), 1)] => ratio(v).filter(|r| r.is_integer())?.to_integer(),
        _ => return None,
    };
    let d = n.magnitude().to_str_radix(2);
    let d = format!("{d:0>w$}", w = d.len().div_ceil(4) * 4);
    let groups: Vec<_> = d.as_bytes().chunks(4).map(|g| std::str::from_utf8(g).unwrap()).collect();
    Some(Ok(Value::str(format!("{}0b{}", if n.is_negative() { "-" } else { "" }, groups.join("_")))))
}

fn int(v: &Value) -> Result<BigInt, Fail> {
    match v {
        Value::Int(n, _) => Ok((*n).into()),
        Value::Big(n, _) => Ok((**n).clone()),
        _ => Err(Fail::BadArgs),
    }
}

fn big(n: BigInt) -> Value {
    exact(BigRational::from_integer(n), Radix::DEC, false)
}

fn gcd_lcm(name: &str, vs: &[Value]) -> Call {
    let mut acc = int(vs.first().ok_or("empty list")?)?;
    for v in &vs[1..] {
        acc = if name == "gcd" { acc.gcd(&int(v)?) } else { acc.lcm(&int(v)?) };
    }
    Ok(big(acc))
}

const WITNESSES: [u32; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

/// Miller-Rabin with the first 12 primes as witnesses, which is exact below 3.3e24.
// ponytail: probabilistic above 3.3e24 (fixed witnesses); add random witnesses if huge primes ever matter.
fn is_prime(n: &BigInt) -> bool {
    if *n < BigInt::from(2) {
        return false;
    }
    for p in WITNESSES {
        if (n % p).is_zero() {
            return *n == BigInt::from(p);
        }
    }
    let n1: BigInt = n - 1u32;
    let s = n1.trailing_zeros().unwrap();
    let d = &n1 >> s;
    'next: for a in WITNESSES {
        let mut x = BigInt::from(a).modpow(&d, n);
        if x.is_one() || x == n1 {
            continue;
        }
        for _ in 1..s {
            x = &x * &x % n;
            if x == n1 {
                continue 'next;
            }
        }
        return false;
    }
    true
}

/// Prime factors of an odd `n`, by Pollard's rho.
// ponytail: rho takes ~sqrt(smallest factor) steps, so a product of two 20+ digit primes hangs; ECM if that matters.
fn factor(n: BigInt, out: &mut Vec<BigInt>) {
    if n.is_one() {
        return;
    }
    if is_prime(&n) {
        out.push(n);
        return;
    }
    for c in 1u32.. {
        let f = |x: &BigInt| (x * x + c) % &n;
        let (mut x, mut y, mut d) = (BigInt::from(2), BigInt::from(2), BigInt::one());
        while d.is_one() {
            x = f(&x);
            y = f(&f(&y));
            d = (&x - &y).abs().gcd(&n);
        }
        if d != n {
            let rest = &n / &d;
            factor(d, out);
            factor(rest, out);
            return;
        }
    }
}

/// `n` as its low `w` bits, unsigned; `n` must fit in `w` bits, signed or unsigned.
fn low_bits(n: i64, w: i64) -> Result<(u64, u32), Fail> {
    if !(1..=64).contains(&w) {
        return Err("width must be 1-64 bits".into());
    }
    let w = w as u32;
    if !fits(n, w) {
        return Err(format!("{n} does not fit in {w} bits").into());
    }
    Ok((n as u64 & mask(w), w))
}

fn unsigned(v: u64, r: Radix) -> Value {
    let r = Radix { width: 0, ..r };
    i64::try_from(v).map_or_else(|_| Value::Big(Rc::new(v.into()), r), |n| Value::Int(n, r))
}

fn from_digits(l: &[Value], base: u32) -> Result<Value, String> {
    let mut acc = BigInt::from(0);
    for v in l {
        let d = match v {
            Value::Int(d, _) if (0..base as i64).contains(d) => *d,
            _ => return Err(format!("{v:?} is not a base-{base} digit")),
        };
        acc = acc * base + d;
    }
    Ok(exact(BigRational::from_integer(acc), Radix { base, width: 0 }, false))
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn math() {
        assert_eq!(show("sqrt(16)"), "4");
        assert_eq!(show("round(pi, 2)"), "3.14");
        assert_eq!(show("round(2.5 km)"), "3 km");
        assert_eq!(show("sin(30 deg)"), "0.5");
        assert_eq!(show("asin(1) to deg"), "90 deg");
        assert_eq!(show("log(1000)"), "3");
        assert_eq!(show("1234.digits"), "[1, 2, 3, 4]");
        assert_eq!(show("0b110.digits"), "[1, 1, 0]");
        assert_eq!(show("0.digits"), "[0]");
        assert_eq!(show("[1, 1, 0].from_digits(2)"), "0b110");
        assert_eq!(show("0xff.digits.from_digits(16)"), "0xff");
        assert!(try_eval("[2].from_digits(2)").is_err());
    }

    #[test]
    fn more_math() {
        assert_eq!(show("atan2(1, -1) to deg"), "135 deg");
        assert_eq!(show("[hypot(3, 4), hypot(3 m, 400 cm), cbrt(27), exp(0)]"), "[5, 5 m, 3, 1]");
        assert_eq!(show("[clamp(15, 0, 10), clamp(-1, 0, 10), clamp(5 m, 1 m, 2 m)]"), "[10, 0, 2 m]");
        assert_eq!(show("[sign(-5 km), sign(0), sign(2/3), trunc(-2.7), trunc(7/2), trunc(-2.7 m)]"), "[-1, 0, 1, -2, 3, -2 m]");
        assert_eq!(show("[round(7.3, 0.25), round(7, 1/2), round(17 min, 15 min), round(80 min, 1 h)]"), "[7.25, 7, 15 min, 60 min]");
        assert_eq!(show("[is_nan(nan), is_nan(1), nan == nan, inf > 10 ** 300, tau / pi]"), "[true, false, false, true, 2]");
        assert!(try_eval("clamp(1, 5, 0)").is_err());
    }

    #[test]
    fn formatting() {
        assert_eq!(show(r#"["{255:#x}", "{255:#06x}", "{255:#X}", "{5:#b}", format("%#o", 8)]"#), r#"["0xff", "0x00ff", "0XFF", "0b101", "0o10"]"#);
        assert_eq!(show("[pi.fixed(2), (5 km to mi).fixed(1), 2.fixed(0)]"), r#"["3.14", "3.1 mi", "2"]"#);
        assert_eq!(show("[123456.sci, 123456.sci(2), 0.256.percent, (1/3).percent(1)]"), r#"["1.23456e5", "1.23e5", "25.6%", "33.3%"]"#);
        assert_eq!(
            show("[1234567.commas, (-1234.5).commas(2), (2 ** 70).commas, 999.commas]"),
            r#"["1,234,567", "-1,234.50", "1,180,591,620,717,411,303,424", "999"]"#
        );
        assert_eq!(
            show(
                r#"x = 3.14159
n = 255
"{x:.2f}|{n:>6}|{n:<6}|{n:^7}|{n:08x}|{n:X}|{n:b}|{-n:o}|{x:+.1f}|{1234567:,}|{-42:06}|{"ab":*>4}|{0.5:%}|{1/8:.1%}""#
            ),
            "3.14|   255|255   |  255  |000000ff|FF|11111111|-377|+3.1|1,234,567|-00042|**ab|50%|12.5%"
        );
        assert_eq!(
            show(
                r#"d = 5 km
"{d:.1f} / {d:>8}""#
            ),
            "5.0 km /     5 km"
        );
        assert_eq!(
            show(r#"format("%5.2f|%-4d|%05d|%x|%s|%,d|%%|%+.1e", 3.14159, 7, 42, 255, "hi", 1234567, 1234.5)"#),
            " 3.14|7   |00042|ff|hi|1,234,567|%|+1.2e3"
        );
        assert_eq!(
            show(
                r#"x = {a: 1}
"{ {b: x.a}.b :>3}""#
            ),
            "  1"
        );
        assert!(try_eval(r#""{1:q}""#).is_err());
        assert!(try_eval(r#""{1.5:x}""#).is_err());
        assert!(try_eval(r#"format("%d %d", 1)"#).is_err());
        assert!(try_eval(r#"format("%d", 1, 2)"#).is_err());
    }

    #[test]
    fn number_theory() {
        assert_eq!(show("[gcd(12, 18), [12, 18, 27].gcd, lcm(4, 6), (1..=20).lcm, gcd(0, 5)]"), "[6, 3, 12, 232792560, 5]");
        assert_eq!(show("(1..30).filter(is_prime)"), "[2, 3, 5, 7, 11, 13, 17, 19, 23, 29]");
        assert_eq!(
            show("[is_prime(2 ** 61 - 1), is_prime(2 ** 61 + 1), is_prime(3215031751), is_prime(1), is_prime(-7)]"),
            "[true, false, false, false, false]"
        );
        assert_eq!(show("[360.factors, 1.factors, 97.factors, factors(2 ** 32 + 1)]"), "[[2, 2, 2, 3, 3, 5], [], [97], [641, 6700417]]");
        assert_eq!(show("factors(600851475143)"), "[71, 839, 1471, 6857]");
        assert_eq!(show("[factorial(0), factorial(20), factorial(25)]"), "[1, 2432902008176640000, 15511210043330985984000000]");
        assert_eq!(show("[choose(5, 2), choose(52, 5), choose(3, 5), choose(100, 50)]"), "[10, 2598960, 0, 100891344545564193334812497256]");
        assert_eq!(show("[mod_pow(2, 100, 7), mod_pow(-2, 3, 5), mod_pow(3, 10 ** 18, 1000000007)]"), "[2, 2, 246336683]");
        assert!(try_eval("factors(0)").is_err());
        assert!(try_eval("factorial(-1)").is_err());
        assert!(try_eval("mod_pow(2, 3, 0)").is_err());
    }

    #[test]
    fn bits() {
        assert_eq!(show("[popcount(0b1011), popcount(-1), bit(0b100, 2), bit(0b100, 1)]"), "[3, 64, 1, 0]");
        assert_eq!(show("[set_bit(0b1, 4), clear_bit(0xff, 0)]"), "[0b10001, 0xfe]");
        assert_eq!(show("[popcount(2 ** 64 - 1), bit(2 ** 70, 70), bit(-1, 100)]"), "[64, 1, 1]");
        assert_eq!(show("[set_bit(0x0, 64), clear_bit(2 ** 70 + 1, 70)]"), "[0x10000000000000000, 1]");
        assert!(try_eval("popcount(-(2 ** 70))").is_err());
        assert_eq!(
            show("[rotl(0x81, 1, 8), rotr(0x81, 1, 8), rotl(0x81, 9, 8), rotl(-1, 4, 8), rotr(1, 1, 64)]"),
            "[0x3, 0xc0, 0x3, 255, 9223372036854775808]"
        );
        assert_eq!(show("[byteswap(0x1234, 16), byteswap(0x12345678, 32), byteswap(1, 64)]"), "[0x3412, 0x78563412, 72057594037927936]");
        assert_eq!(show("[0xf0 to bits, 5 to bits, 0 to bits]"), r#"["0b1111_0000", "0b0101", "0b0000"]"#);
        assert_eq!(show("1 B to bits"), "8 bit");
        assert!(try_eval("rotl(256, 1, 8)").is_err());
        assert!(try_eval("byteswap(1, 12)").is_err());
    }
}
