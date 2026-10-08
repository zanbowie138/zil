//! Money: currencies as quantities, shown as money (`$1,234.56`, `¥1,200`), and paid time (workday, workyr). Finance builds on it.

use super::{MONEY, NONE, Unit};
use crate::modules::math::formatting::commas;
use crate::modules::{Module, Section};
use crate::value::fmt_float;

pub const MODULE: Module =
    Module { name: "money", about: "currencies, live exchange rates, paid time like workday and workyr", examples: EXAMPLES, guide: GUIDE, ..Module::EMPTY };

#[rustfmt::skip]
const EXAMPLES: &[Section] = &[
    ("money", &[
        ("yearly salary from hourly", "$25/h to USD/workyr"),
        ("what a meeting costs", "6 * 95000 USD/workyr * 1 h"),
        ("hours of work to buy it", "$1200 / ($40/h) to workday"),
        ("unit price", "$4.99 / 12 oz to USD/lb"),
        ("a subscription per year", "$15.99/mo to USD/yr"),
    ]),
];

#[rustfmt::skip]
const GUIDE: &[Section] = &[
    ("types", &[("money", "1234.5 USD")]),
    ("pretty", &[("long and short: as money always shows", r#"pretty(1234567 USD, "short")"#)]),
    ("literals", &[
        ("$ € £ before a number", "$25/h"),
        ("any currency code", "1200 JPY"),
    ]),
    ("paid time", &[
        ("workday 8 h, workwk 40 h, workmo, workyr 2080 h; USD/yr stays calendar time", ""),
        ("hourly to salary", "$25/h to USD/workyr"),
        ("salary to hourly", "85000 USD/workyr to USD/h"),
    ]),
    ("currencies", &[("rates load (from cache, or online after allow_network_access()) only to convert between currencies: 20 USD to EUR", "")]),
];

/// `$1,234.56`, `$25.00/h`, `5%/yr`; `None` leaves the quantity's usual display.
pub fn show(x: f64, u: &Unit) -> Option<String> {
    match currency_of(x, u) {
        Some((x, cur, per)) if per.0.iter().all(|(_, p)| *p < 0) => {
            let per: String = per.0.iter().map(|(d, p)| if *p == -1 { format!("/{}", d.name) } else { format!("/{}^{}", d.name, -p) }).collect();
            Some(format!("{}{per}", amount(x, &cur.0[0].0.name)))
        }
        // ponytail: any small per-period number reads as a rate (0.5/d shows 50%/d); tag percent quantities if that misleads.
        _ => match u.0.as_slice() {
            [(t, -1)] if x.abs() < 1.0 && ["d", "wk", "mo", "yr"].contains(&t.name.as_str()) => Some(format!("{}%/{}", fmt_float(x * 100.0), t.name)),
            _ => None,
        },
    }
}

/// `$1,234.56`, `-€5.00`, `¥1,200`, `12.50 CAD`; amounts under a cent keep their digits.
fn amount(x: f64, code: &str) -> String {
    let places = places(code);
    // Float residue like `$0.1 + $0.2 - $0.3`.
    let x = if x.abs() < 1e-9 { 0.0 } else { x };
    let tiny = x != 0.0 && x.abs() < 0.5 / 10f64.powi(places);
    let digits = if tiny || !x.is_finite() { fmt_float(x.abs()) } else { commas(&format!("{:.*}", places as usize, x.abs())) };
    let sign = if x < 0.0 { "-" } else { "" };
    match code {
        "USD" => format!("{sign}${digits}"),
        "EUR" => format!("{sign}€{digits}"),
        "GBP" => format!("{sign}£{digits}"),
        "JPY" => format!("{sign}¥{digits}"),
        "INR" => format!("{sign}₹{digits}"),
        "KRW" => format!("{sign}₩{digits}"),
        _ => format!("{sign}{digits} {code}"),
    }
}

/// Decimal places a currency is counted in; plain numbers count cents.
pub fn places(code: &str) -> i32 {
    if matches!(code, "JPY" | "KRW" | "ISK") { 0 } else { 2 }
}

/// A money quantity as amount, currency and what's left (`/h` in `USD/h`); a leftover that cancels
/// (`h/workyr` in `$25/h * 1 workyr`) is folded into the amount. `None` unless exactly one currency.
pub fn currency_of(x: f64, u: &Unit) -> Option<(f64, Unit, Unit)> {
    let mut money = u.0.iter().filter(|(d, _)| d.dim == MONEY);
    let (c, 1) = money.next()? else { return None };
    if money.next().is_some() {
        return None;
    }
    let cur = Unit(vec![(c.clone(), 1)]);
    let rest = Unit(u.0.iter().filter(|(d, _)| d.dim != MONEY).cloned().collect());
    if rest.dim() == NONE { Some((x * rest.scale(), cur, Unit::default())) } else { Some((x, cur, rest)) }
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::show;

    #[test]
    fn display() {
        assert_eq!(show("1234567.891 USD"), "$1,234,567.89");
        assert_eq!(show("-$5"), "-$5.00");
        assert_eq!(show("€3.5"), "€3.50");
        assert_eq!(show("1200 JPY"), "¥1,200");
        assert_eq!(show("12.5 CAD"), "12.50 CAD");
        assert_eq!(show("0.0042 USD/kWh"), "$0.0042/kWh");
        assert_eq!(show("5%/yr"), "5%/yr");
        assert_eq!(show("$25/h * 40 h"), "$1,000.00");
        assert_eq!(show("$25/h * 1 workyr"), "$52,000.00");
        assert_eq!(show("$0.1 + $0.2 - $0.3"), "$0.00");
    }
}
