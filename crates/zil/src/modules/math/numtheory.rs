//! Number theory on exact integers: gcd, primes, factoring, combinatorics.

use super::{big, int};
use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::{Value, num, ratio};
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};

pub const MODULE: Module = Module {
    name: "numtheory",
    about: "gcd, primes, factoring, factorials, modular powers; exact at any size",
    #[rustfmt::skip]
    examples: &[
        ("numtheory", &[
            ("prime factors", "factors(2 ** 32 + 1)"),
            ("is a Mersenne number prime", "is_prime(2 ** 61 - 1)"),
            ("poker hands", "choose(52, 5)"),
            ("modular power", "mod_pow(3, 1000, 7)"),
            ("a 209-digit Fibonacci", "fib(1000).str.len"),
            ("π's famous fractions", "pi.cfrac(4)"),
            ("the next prime after a googol", "(10 ** 100).next_prime - 10 ** 100"),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("gcd", "gcd(a: int, b: int, ...) / gcd(xs: list)", "greatest common divisor", &["gcd(12, 18)", "[12, 18, 27].gcd"], &["lcm"]),
    doc("lcm", "lcm(a: int, b: int, ...) / lcm(xs: list)", "least common multiple", &["lcm(4, 6)", "(1..=20).lcm"], &["gcd"]),
    doc("is_prime", "is_prime(n: int)", "primality (Miller-Rabin; exact below 3e24)", &["is_prime(97)", "is_prime(2 ** 61 - 1)"], &["factors"]),
    doc("factors", "factors(n: int)", "prime factors, smallest first", &["360.factors", "factors(2 ** 32 + 1)"], &["is_prime", "gcd"]),
    doc("factorial", "factorial(n: int)", "n!, exact", &["factorial(5)", "factorial(30)"], &["choose"]),
    doc("choose", "choose(n: int, k: int)", "ways to pick k of n, exact", &["choose(5, 2)", "choose(52, 5)"], &["factorial"]),
    doc("mod_pow", "mod_pow(b: int, e: int, m: int)", "b ** e % m without the huge power", &["mod_pow(2, 100, 7)", "mod_pow(3, 10 ** 18, 1000000007)"], &["gcd"]),
    doc("fib", "fib(n: int)", "the nth Fibonacci number (fib(0) = 0), exact", &["fib(10)", "fib(100)"], &["factorial"]),
    doc("next_prime", "next_prime(n: int)", "the smallest prime greater than n", &["next_prime(100)", "next_prime(2 ** 64)"], &["is_prime"]),
    doc("totient", "totient(n: int)", "Euler's φ: how many of 1..n share no factor with n", &["totient(36)", "totient(97)"], &["factors", "gcd"]),
    doc("isqrt", "isqrt(n: int)", "the integer square root, floor(√n), exact at any size", &["isqrt(99)", "isqrt(10 ** 40)"], &["sqrt"]),
    doc("cfrac", "cfrac(x: num, terms?: int)", "continued fraction terms [a0; a1, a2, ...]; exact for fractions, up to terms (default 20) for floats", &["(415/93).cfrac", "pi.cfrac(5)", "sqrt(2).cfrac(6)"], &["frac"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("gcd" | "lcm", [List(l)]) => gcd_lcm(name, &l.borrow())?,
        ("gcd" | "lcm", vs) if vs.len() >= 2 => gcd_lcm(name, vs)?,
        ("is_prime", [v]) => Bool(is_prime(&int(v)?)),
        ("factors", [v]) => {
            let mut n = int(v)?;
            if !n.is_positive() {
                return Err(Fail::Arg(0, format!("expected a positive integer, got {n}")));
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
        ("factorial", [Int(n, _)]) => return Err(Fail::Arg(0, format!("n must be from 0 to 20000, got {n}"))),
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
            if e.is_negative() {
                return Err(Fail::Arg(1, format!("the exponent must be 0 or more, got {e}")));
            }
            if !m.is_positive() {
                return Err(Fail::Arg(2, format!("the modulus must be positive, got {m}")));
            }
            big(b.modpow(&e, &m))
        }
        ("fib", [Int(n, _)]) if (0..=100_000).contains(n) => {
            let (mut a, mut b) = (BigInt::zero(), BigInt::one());
            for _ in 0..*n {
                (a, b) = (b.clone(), a + b);
            }
            big(a)
        }
        ("fib", [Int(n, _)]) => return Err(Fail::Arg(0, format!("n must be from 0 to 100000, got {n}"))),
        ("next_prime", [v]) => {
            let mut n = int(v)?.max(BigInt::one()) + 1u32;
            while !is_prime(&n) {
                n += 1u32;
            }
            big(n)
        }
        ("totient", [v]) => {
            let n = int(v)?;
            if !n.is_positive() {
                return Err(Fail::Arg(0, format!("expected a positive integer, got {n}")));
            }
            let mut ps = vec![];
            let mut m = n.clone();
            for p in [2u32, 3, 5] {
                while (&m % p).is_zero() {
                    m /= p;
                    ps.push(BigInt::from(p));
                }
            }
            factor(m, &mut ps);
            ps.sort();
            ps.dedup();
            big(ps.iter().fold(n, |acc, p| acc / p * (p - 1u32)))
        }
        ("isqrt", [v]) => {
            let n = int(v)?;
            if n.is_negative() {
                return Err(Fail::Arg(0, format!("expected 0 or more, got {n}")));
            }
            big(n.sqrt())
        }
        ("cfrac", [v, rest @ ..]) if rest.len() <= 1 => {
            let terms = match rest {
                [] => 20,
                [Int(t, _)] if (1..=1000).contains(t) => *t as usize,
                _ => return Err(Fail::Arg(1, "terms must be an int from 1 to 1000".into())),
            };
            let mut out = vec![];
            if let Some(r) = ratio(v) {
                let (mut p, mut q) = (r.numer().clone(), r.denom().clone());
                while !q.is_zero() && out.len() < terms {
                    let (a, rem) = p.div_mod_floor(&q);
                    out.push(big(a));
                    (p, q) = (q, rem);
                }
            } else {
                let mut x = num(v).ok_or(Fail::BadArgs)?;
                for _ in 0..terms {
                    let a = x.floor();
                    out.push(Value::int(a as i64));
                    // ponytail: stop once the float's error swamps the remainder; exact input avoids this.
                    if (x - a).abs() < 1e-9 {
                        break;
                    }
                    x = 1.0 / (x - a);
                }
            }
            Value::list(out)
        }
        _ => return Err(Fail::BadArgs),
    })
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

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

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
        assert_eq!(show("[fib(0), fib(1), fib(10), fib(100)]"), "[0, 1, 55, 354224848179261915075]");
        assert_eq!(
            show("[next_prime(0), next_prime(13), next_prime(2 ** 61 - 2), totient(1), totient(36), totient(97)]"),
            "[2, 17, 2305843009213693951, 1, 12, 96]"
        );
        assert_eq!(show("[isqrt(0), isqrt(99), isqrt(100), isqrt(10 ** 40)]"), "[0, 9, 10, 100000000000000000000]");
        assert_eq!(show("[(415/93).cfrac, 3.cfrac, (-7/2).cfrac, pi.cfrac(4), 0.75.cfrac]"), "[[4, 2, 6, 7], [3], [-4, 2], [3, 7, 15, 1], [0, 1, 3]]");
        assert!(try_eval("isqrt(-1)").is_err());
        assert!(try_eval("totient(0)").is_err());
        assert!(try_eval("factors(0)").is_err());
        assert!(try_eval("factorial(-1)").is_err());
        assert!(try_eval("mod_pow(2, 3, 0)").is_err());
    }
}
