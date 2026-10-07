//! Money: currency display, pay, growth, loans, splitting bills. Functions tell their arguments apart
//! by unit: `$1000` is a sum, `$500/mo` a payment, `7%/yr` a rate, `10 yr` a time, `"monthly"` compounding.

use super::math::commas;
use super::units::{MONEY, NONE, PER_TIME, TIME, Unit, convert_value, unit};
use super::{Call, Doc, Fail, Module, Section, doc};
use crate::interp::Interp;
use crate::lexer::Span;
use crate::value::{Value, fmt_float, num};
use indexmap::IndexMap;

pub const MODULE: Module = Module {
    name: "money",
    about: "currencies shown as money, pay, interest, loans, splitting bills",
    examples: EXAMPLES,
    guide: GUIDE,
    fns: FNS,
    groups: &[
        ("pay", &["salary"]),
        ("growth", &["grow", "cagr", "doubling", "apy", "real_rate"]),
        ("loans", &["payment", "payoff", "amortize"]),
        ("splitting", &["share", "settle"]),
        ("percent", &["change", "margin", "markup"]),
    ],
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const EXAMPLES: &[Section] = &[
    ("money", &[
        ("yearly salary from hourly", "$25/h to USD/workyr"),
        ("what a meeting costs", "6 * 95000 USD/workyr * 1 h"),
        ("hours of work to buy it", "$1200 / ($40/h) to workday"),
        ("unit price", "$4.99 / 12 oz to USD/lb"),
        ("a subscription per year", "$15.99/mo to USD/yr"),
        ("mortgage payment", "payment($400000, 6.5%/yr, 30 yr)"),
        ("interest over the loan", "payment($400000, 6.5%/yr, 30 yr) * 30 yr - $400000"),
        ("debt-free date", "today + payoff($5000, 22%/yr, $200/mo)"),
        ("saving $500 a month", "grow($0, 7%/yr, 30 yr, $500/mo)"),
        ("split a bill", "share($100, 3)"),
        ("who owes whom", "settle({ana: $120, ben: $0, cy: $30})"),
    ]),
];

#[rustfmt::skip]
const GUIDE: &[Section] = &[
    ("types", &[("money", "1234.5 USD")]),
    ("literals", &[
        ("$ € £ before a number", "$25/h"),
        ("any currency code", "1200 JPY"),
    ]),
    ("paid time", &[
        ("workday 8 h, workwk 40 h, workmo, workyr 2080 h; USD/yr stays calendar time", ""),
        ("hourly to salary", "$25/h to USD/workyr"),
        ("salary to hourly", "85000 USD/workyr to USD/h"),
    ]),
    ("arguments", &[
        ("functions tell arguments apart by unit: $1000 a sum, $500/mo a payment,", ""),
        ("7%/yr a rate, 10 yr a time, \"monthly\" how often interest compounds", ""),
    ]),
    ("currencies", &[("rates load (with a prompt) only to convert between currencies: 20 USD to EUR", "")]),
];

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("salary", "salary(rate, opts?)", "pay per hour, day, week, month and year; opts {hours: 40 a week, weeks: 52 paid}", &["salary($25/h)", "salary(85000 USD/yr, {hours: 37.5})"], &["grow"]),
    doc("grow", "grow(sum, rate, time, payment?, compounding?)", "future value: a sum and/or payments growing at a rate", &["grow($10000, 7%/yr, 30 yr)", "grow($0, 7%/yr, 30 yr, $500/mo)", r#"grow($1000, 5%/yr, 1 yr, "daily")"#], &["cagr", "payment"]),
    doc("cagr", "cagr(start, end, time)", "compound annual growth rate", &["cagr($1000, $2500, 8 yr)"], &["grow", "change"]),
    doc("doubling", "doubling(rate, compounding?)", "time for money to double", &["doubling(7%/yr)"], &["grow"]),
    doc("apy", "apy(rate, compounding)", "effective yearly rate after compounding", &[r#"apy(5%/yr, "daily")"#, r#"apy(5%/yr, "monthly")"#], &["grow"]),
    doc("real_rate", "real_rate(rate, inflation)", "return after inflation", &["real_rate(7%/yr, 3%/yr)"], &["grow"]),
    doc("payment", "payment(loan, rate, time, compounding?)", "monthly payment that pays off a loan", &["payment($400000, 6.5%/yr, 30 yr)", "payment($25000, 7%/yr, 5 yr) * 5 yr"], &["payoff", "amortize"]),
    doc("payoff", "payoff(balance, rate, payment, compounding?)", "how long a payment takes to clear a balance, in whole payments", &["payoff($5000, 22%/yr, $200/mo)", "today + payoff($5000, 22%/yr, $200/mo)"], &["payment"]),
    doc("amortize", "amortize(loan, rate, time, opts?)", "payment schedule: {n, payment, interest, principal, balance} per month; opts {extra: $200/mo}", &["amortize($1000, 12%/yr, 3 mo)", "amortize($400000, 6.5%/yr, 30 yr, {extra: $300/mo}).len * 1 mo to yr"], &["payment"]),
    doc("share", "share(sum, n or weights)", "split into parts that add up exactly, to the cent", &["share($100, 3)", "share($100, [2, 1, 1])"], &["settle"]),
    doc("settle", "settle({name: paid, ...})", "who pays whom so everyone paid the same", &["settle({ana: $120, ben: $0, cy: $30})"], &["share"]),
    doc("change", "change(from, to)", "fractional change from one value to another", &["change($80, $100)", "change(80, 60).percent"], &["margin", "cagr"]),
    doc("margin", "margin(cost, price)", "profit as a fraction of the price", &["margin($60, $100)"], &["markup"]),
    doc("markup", "markup(cost, price)", "profit as a fraction of the cost", &["markup($60, $100)"], &["margin"]),
];

const YR: f64 = 31556952.0;
const MO: f64 = YR / 12.0;

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
fn places(code: &str) -> i32 {
    if matches!(code, "JPY" | "KRW" | "ISK") { 0 } else { 2 }
}

/// A money quantity as amount, currency and what's left (`/h` in `USD/h`); a leftover that cancels
/// (`h/workyr` in `$25/h * 1 workyr`) is folded into the amount. `None` unless exactly one currency.
fn currency_of(x: f64, u: &Unit) -> Option<(f64, Unit, Unit)> {
    let mut money = u.0.iter().filter(|(d, _)| d.dim == MONEY);
    let (c, 1) = money.next()? else { return None };
    if money.next().is_some() {
        return None;
    }
    let cur = Unit(vec![(c.clone(), 1)]);
    let rest = Unit(u.0.iter().filter(|(d, _)| d.dim != MONEY).cloned().collect());
    if rest.dim() == NONE { Some((x * rest.scale(), cur, Unit::default())) } else { Some((x, cur, rest)) }
}

/// Arguments sorted by unit.
#[derive(Default)]
struct Args {
    /// Value and currency, empty for plain numbers.
    sums: Vec<(f64, Unit)>,
    /// Nominal yearly rate, and the seconds it's quoted per (`1%/mo` compounds monthly).
    rates: Vec<(f64, f64)>,
    /// Seconds.
    time: Option<f64>,
    /// Amount per period, currency, period unit.
    pay: Option<(f64, Unit, Unit)>,
    /// Compounding periods a year; infinite for continuous.
    compound: Option<f64>,
    opts: IndexMap<String, Value>,
}

impl Args {
    fn new(vals: &[Value]) -> Result<Args, Fail> {
        let mut a = Args::default();
        for v in vals {
            match v {
                Value::Map(m) => a.opts = m.borrow().clone(),
                Value::Str(s) => a.compound = Some(compounding(s)?),
                Value::Qty(x, u) if u.dim() == TIME => a.time = Some(u.to_si(*x)),
                Value::Qty(x, u) if u.dim() == PER_TIME => {
                    let period = match u.0.as_slice() {
                        [(t, -1)] => t.scale(),
                        _ => YR,
                    };
                    a.rates.push((u.to_si(*x) * YR, period));
                }
                Value::Qty(x, u) => match currency_of(*x, u) {
                    Some((x, cur, rest)) if rest.0.is_empty() => a.sums.push((x, cur)),
                    Some((x, cur, rest)) if rest.dim() == PER_TIME && rest.0.len() == 1 => a.pay = Some((x, cur, rest.pow(-1))),
                    _ => return Err(Fail::BadArgs),
                },
                v => a.sums.push((num(v).ok_or(Fail::BadArgs)?, Unit::default())),
            }
        }
        Ok(a)
    }

    fn rate(&self) -> Result<(f64, f64), Fail> {
        match self.rates[..] {
            [r] => Ok(r),
            _ => Err("needs one rate, like 7%/yr".into()),
        }
    }

    fn time(&self) -> Result<f64, Fail> {
        self.time.ok_or_else(|| "needs a time, like 10 yr".into())
    }

    /// The one sum; `what` names it in the error.
    fn sum(&self, what: &str) -> Result<(f64, Unit), Fail> {
        match &self.sums[..] {
            [s] => Ok(s.clone()),
            _ => Err(format!("needs one {what}, like $1000").into()),
        }
    }

    /// The payment per period, in `cur` (its own currency if `cur` is a plain number), and the period.
    fn pay(&self, cur: &Unit) -> Result<Option<(f64, Unit, Unit)>, Fail> {
        let Some((x, c, period)) = &self.pay else { return Ok(None) };
        if cur.0.is_empty() {
            return Ok(Some((*x, c.clone(), period.clone())));
        }
        Ok(Some((convert_value(*x, c, cur)?, cur.clone(), period.clone())))
    }

    /// Interest per period of `secs`; compounds once per period unless told otherwise.
    fn per_period(&self, r: f64, secs: f64) -> f64 {
        match self.compound {
            Some(m) if m.is_infinite() => (r * secs / YR).exp() - 1.0,
            Some(m) => (1.0 + r / m).powf(m * secs / YR) - 1.0,
            None => r * secs / YR,
        }
    }

    fn only_opts(&self, keys: &[&str]) -> Result<(), Fail> {
        match self.opts.keys().find(|k| !keys.contains(&k.as_str())) {
            Some(k) => Err(format!("unknown option {k}; expected one of {}", keys.join(", ")).into()),
            None => Ok(()),
        }
    }
}

/// Compounding periods a year.
fn compounding(s: &str) -> Result<f64, Fail> {
    Ok(match s {
        "yearly" | "annually" => 1.0,
        "quarterly" => 4.0,
        "monthly" => 12.0,
        "weekly" => YR / 604800.0,
        "daily" => YR / 86400.0,
        "continuous" | "continuously" => f64::INFINITY,
        _ => return Err(format!("unknown compounding {s:?}; use yearly, quarterly, monthly, weekly, daily or continuous").into()),
    })
}

/// Monthly payment on a loan of `p` over `n` months at `i` a month.
fn pmt(p: f64, i: f64, n: f64) -> f64 {
    if i == 0.0 { p / n } else { p * i / (1.0 - (1.0 + i).powf(-n)) }
}

/// Two values as numbers in the first one's unit.
fn pair(a: &Value, b: &Value) -> Result<(f64, f64), Fail> {
    match (a, b) {
        (Value::Qty(x, u), Value::Qty(y, w)) if u.dim() == w.dim() => Ok((*x, convert_value(*y, w, u)?)),
        _ => Ok((num(a).ok_or(Fail::BadArgs)?, num(b).ok_or(Fail::BadArgs)?)),
    }
}

/// A money value, or a plain number, as amount and currency.
fn money(v: &Value) -> Result<(f64, Unit), Fail> {
    match v {
        Value::Qty(x, u) => match currency_of(*x, u) {
            Some((x, cur, rest)) if rest.0.is_empty() => Ok((x, cur)),
            _ => Err(Fail::BadArgs),
        },
        v => Ok((num(v).ok_or(Fail::BadArgs)?, Unit::default())),
    }
}

/// Smallest counted unit per 1: 100 for cents.
fn per_unit(cur: &Unit) -> f64 {
    10f64.powi(cur.0.first().map_or(2, |(c, _)| places(&c.name)))
}

fn per_yr() -> Result<Unit, Fail> {
    Ok(unit("yr")?.pow(-1))
}

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    let a = || Args::new(args);
    Ok(match (name, args) {
        ("salary", [Qty(x, u), rest @ ..]) if rest.len() <= 1 => {
            let a = a()?;
            a.only_opts(&["hours", "weeks"])?;
            let opt = |k: &str, default: f64| a.opts.get(k).map_or(Some(default), num).ok_or(format!("{k} must be a number"));
            let (hours, weeks) = (opt("hours", 40.0)?, opt("weeks", 52.0)?);
            let (x, cur, per) = match currency_of(*x, u) {
                Some(c) => c,
                None => (*x, Unit::default(), u.clone()),
            };
            let [(t, -1)] = per.0.as_slice() else { return Err("needs pay per time, like $25/h or 85000 USD/yr".into()) };
            let hours_per = match t.name.as_str() {
                "h" => 1.0,
                "min" => 1.0 / 60.0,
                "s" => 1.0 / 3600.0,
                "d" | "workday" => hours / 5.0,
                "wk" | "workwk" => hours,
                "mo" | "workmo" => hours * weeks / 12.0,
                "yr" | "workyr" => hours * weeks,
                p => return Err(format!("can't tell how many hours are paid per {p}").into()),
            };
            let hourly = x / hours_per;
            let rows = [("hour", 1.0), ("day", hours / 5.0), ("week", hours), ("month", hours * weeks / 12.0), ("year", hours * weeks)];
            Value::map(rows.into_iter().map(|(k, h)| (k.to_string(), Value::qty(hourly * h, cur.clone()))).collect())
        }
        ("grow", _) => {
            let a = a()?;
            let ((r, rate_period), t) = (a.rate()?, a.time()?);
            let (lump, cur) = match (&a.sums[..], &a.pay) {
                ([], Some(_)) => (0.0, Unit::default()),
                _ => a.sum("starting sum")?,
            };
            let (pay, cur, period) = match a.pay(&cur)? {
                Some((x, cur, period)) => (x, cur, period.scale()),
                None => (0.0, cur, rate_period),
            };
            let i = a.per_period(r, period);
            let n = t / period;
            let g = (1.0 + i).powf(n);
            let fv = lump * g + if i == 0.0 { pay * n } else { pay * (g - 1.0) / i };
            Value::qty(fv, cur)
        }
        ("cagr", [s, e, Qty(t, u)]) if u.dim() == TIME => {
            let (s, e) = pair(s, e)?;
            Value::qty((e / s).powf(YR / u.to_si(*t)) - 1.0, per_yr()?)
        }
        ("doubling", _) => {
            let a = a()?;
            let (r, period) = a.rate()?;
            let secs = period * 2f64.ln() / (1.0 + a.per_period(r, period)).ln();
            Value::qty(secs / YR, unit("yr")?)
        }
        ("apy", _) => {
            let a = a()?;
            Value::qty(a.per_period(a.rate()?.0, YR), per_yr()?)
        }
        ("real_rate", _) => {
            let a = a()?;
            let plain = a.sums.iter().filter(|s| s.1.0.is_empty()).map(|s| s.0);
            let [r, inflation] = a.rates.iter().map(|r| r.0).chain(plain).collect::<Vec<_>>()[..] else { return Err(Fail::BadArgs) };
            Value::qty((1.0 + r) / (1.0 + inflation) - 1.0, per_yr()?)
        }
        ("payment", _) => {
            let a = a()?;
            let ((p, cur), (r, _), t) = (a.sum("loan")?, a.rate()?, a.time()?);
            Value::qty(pmt(p, a.per_period(r, MO), t / MO), cur.mul(&unit("mo")?, -1))
        }
        ("payoff", _) => {
            let a = a()?;
            let ((b, cur), (r, _)) = (a.sum("balance")?, a.rate()?);
            let Some((pay, _, period)) = a.pay(&cur)? else { return Err("needs a payment, like $200/mo".into()) };
            let i = a.per_period(r, period.scale());
            if pay <= b * i {
                return Err("the payment never covers the interest".into());
            }
            let n = if i == 0.0 { b / pay } else { -(1.0 - b * i / pay).ln() / (1.0 + i).ln() };
            Value::Qty((n - 1e-9).ceil(), period)
        }
        ("amortize", _) => {
            let a = a()?;
            a.only_opts(&["extra"])?;
            let ((p, cur), (r, _), t) = (a.sum("loan")?, a.rate()?, a.time()?);
            let i = a.per_period(r, MO);
            let monthly = cur.mul(&unit("mo")?, -1);
            let extra = match a.opts.get("extra") {
                None => 0.0,
                Some(Qty(x, u)) => convert_value(*x, u, &monthly)?,
                Some(v) => num(v).ok_or("extra must be an amount a month, like $200/mo")?,
            };
            let pay = pmt(p, i, t / MO) + extra;
            let (mut rows, mut balance) = (Vec::new(), p);
            // ponytail: the 100k cap stops a runaway loop on odd inputs; no real loan runs 8000 years.
            while balance > p * 1e-9 && rows.len() < 100_000 {
                let interest = balance * i;
                let principal = (pay - interest).min(balance);
                balance -= principal;
                let mut m = IndexMap::from([("n".to_string(), Value::int(rows.len() as i64 + 1))]);
                for (k, v) in [("payment", interest + principal), ("interest", interest), ("principal", principal), ("balance", balance.max(0.0))] {
                    m.insert(k.into(), Value::qty(v, cur.clone()));
                }
                rows.push(Value::map(m));
            }
            Value::list(rows)
        }
        ("share", [sum, parts]) => {
            let (x, cur) = money(sum)?;
            let weights: Vec<f64> = match parts {
                Int(n, _) if *n > 0 => vec![1.0; *n as usize],
                List(l) => l.borrow().iter().map(|w| num(w).filter(|w| *w >= 0.0)).collect::<Option<_>>().ok_or("weights must be numbers, 0 or more")?,
                _ => return Err(Fail::BadArgs),
            };
            let total_w: f64 = weights.iter().sum();
            if total_w <= 0.0 {
                return Err("needs a positive number of parts".into());
            }
            // Largest remainder, in cents: floor every share, then hand the leftover cents to the biggest fractions.
            let q = per_unit(&cur);
            let cents = (x.abs() * q).round();
            let raw: Vec<f64> = weights.iter().map(|w| cents * w / total_w).collect();
            let mut parts: Vec<f64> = raw.iter().map(|r| r.floor()).collect();
            let mut order: Vec<usize> = (0..parts.len()).collect();
            order.sort_by(|&i, &j| (raw[j] - parts[j]).total_cmp(&(raw[i] - parts[i])));
            let left = (cents - parts.iter().sum::<f64>()) as usize;
            for &i in order.iter().take(left) {
                parts[i] += 1.0;
            }
            Value::list(parts.iter().map(|c| Value::qty(x.signum() * c / q, cur.clone())).collect())
        }
        ("settle", [Map(m)]) => {
            let m = m.borrow();
            let Some(first) = m.values().next() else { return Ok(Value::list(vec![])) };
            let (_, cur) = money(first)?;
            let paid = m.iter().map(|(k, v)| Ok((k.clone(), pair(&Value::qty(0.0, cur.clone()), v)?.1))).collect::<Result<Vec<_>, Fail>>()?;
            let fair = paid.iter().map(|p| p.1).sum::<f64>() / paid.len() as f64;
            let by_size = |mut v: Vec<(String, f64)>| {
                v.sort_by(|a, b| b.1.total_cmp(&a.1));
                v
            };
            let mut owed = by_size(paid.iter().map(|(k, x)| (k.clone(), x - fair)).filter(|p| p.1 > 0.0).collect());
            let mut owing = by_size(paid.iter().map(|(k, x)| (k.clone(), fair - x)).filter(|p| p.1 > 0.0).collect());
            let half_cent = 0.5 / per_unit(&cur);
            let (mut out, mut i, mut j) = (Vec::new(), 0, 0);
            while i < owing.len() && j < owed.len() {
                let t = owing[i].1.min(owed[j].1);
                if t >= half_cent {
                    out.push(Value::str(format!("{} pays {} {}", owing[i].0, owed[j].0, Value::qty(t, cur.clone()))));
                }
                owing[i].1 -= t;
                owed[j].1 -= t;
                i += (owing[i].1 < half_cent) as usize;
                j += (owed[j].1 < half_cent) as usize;
            }
            Value::list(out)
        }
        ("change", [a, b]) => {
            let (a, b) = pair(a, b)?;
            Float((b - a) / a)
        }
        ("margin", [cost, price]) => {
            let (c, p) = pair(cost, price)?;
            Float((p - c) / p)
        }
        ("markup", [cost, price]) => {
            let (c, p) = pair(cost, price)?;
            Float((p - c) / c)
        }
        _ => return Err(Fail::BadArgs),
    })
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

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

    #[test]
    fn pay() {
        assert_eq!(show("$25/h to USD/workyr"), "$52,000.00/workyr");
        assert_eq!(show("$1200 / ($40/h) to workday"), "3.75 workday");
        assert_eq!(show("salary($25/h).year"), "$52,000.00");
        assert_eq!(show("salary(78000 USD/yr, {hours: 37.5}).hour"), "$40.00");
        assert_eq!(show("$25/h * 2080 h == 52000 USD"), "true");
        assert!(try_eval("salary($25/h, {hour: 30})").is_err());
    }

    #[test]
    fn growth() {
        assert_eq!(show("grow($10000, 7%/yr, 30 yr)"), "$76,122.55");
        assert_eq!(show("grow($0, 12%/yr, 1 yr, $100/mo)"), "$1,268.25");
        assert_eq!(show(r#"grow(1000, 5%/yr, 1 yr, "continuous")"#), "1051.27");
        assert_eq!(show("cagr($1000, $2000, 10 yr)"), "7.17735%/yr");
        assert_eq!(show(r#"apy(12%/yr, "monthly")"#), "12.6825%/yr");
        assert_eq!(show("doubling(1%/mo)"), "5.80506 yr");
        assert_eq!(show("real_rate(7%, 3%)"), "3.8835%/yr");
    }

    #[test]
    fn loans() {
        assert_eq!(show("payment($400000, 6.5%/yr, 30 yr)"), "$2,528.27/mo");
        assert_eq!(show("payment(1200, 0%/yr, 1 yr)"), "100 1/mo");
        assert_eq!(show("payoff($5000, 22%/yr, $200/mo)"), "34 mo");
        assert_eq!(show("amortize($400000, 6.5%/yr, 30 yr).len"), "360");
        assert_eq!(show("amortize($400000, 6.5%/yr, 30 yr)[-1].balance"), "$0.00");
        assert_eq!(show("amortize($1000, 12%/yr, 3 mo)[0]"), "{n: 1, payment: $340.02, interest: $10.00, principal: $330.02, balance: $669.98}");
        assert_eq!(show("amortize($400000, 6.5%/yr, 30 yr, {extra: $300/mo}).len"), "269");
        assert!(try_eval("payoff($5000, 22%/yr, $50/mo)").is_err());
    }

    #[test]
    fn splitting() {
        assert_eq!(show("share($100, 3)"), "[$33.34, $33.33, $33.33]");
        assert_eq!(show("share($100, [2, 1, 1])"), "[$50.00, $25.00, $25.00]");
        assert_eq!(show("share(-1, 3)"), "[-0.34, -0.33, -0.33]");
        assert_eq!(show("settle({ana: $120, ben: $0, cy: $30})"), r#"["ben pays ana $50.00", "cy pays ana $20.00"]"#);
        assert_eq!(show("change($80, $100)"), "0.25");
        assert_eq!(show("[margin(60, 100), markup(60, 100)]"), "[0.4, 0.666667]");
    }
}
