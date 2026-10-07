//! Games of chance and the stars: dice notation, the birthday paradox, odds, zodiac signs.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::{Value, num};
use indexmap::IndexMap;

pub const MODULE: Module = Module {
    name: "games",
    about: "dice notation, the birthday paradox, odds in plain words, Western and Chinese zodiac",
    #[rustfmt::skip]
    examples: &[
        ("games", &[
            ("a D&D stat: 4d6, keep the best 3", r#"roll("4d6kh3")"#),
            ("attack with advantage", r#"roll("2d20kh1+5").total"#),
            ("a party of 23", "birthday_paradox(23)"),
            ("how unlikely is the jackpot?", "odds(1 / 292201338)"),
            ("your sign", r#"[zodiac(date("1990-08-10")), chinese_zodiac(1990)]"#),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("roll", "roll(dice?: str)", "roll dice like \"3d6+2\", \"d20\", \"4d6kh3\" (keep highest 3) or \"2d20kl1\"; returns {total, rolls}. Default 1d6", &[r#"roll("3d6+2")"#, r#"roll("d%").total"#], &["rand", "coin"]),
    doc("birthday_paradox", "birthday_paradox(people: int, days?: int)", "the chance at least two of n people share a birthday (or any of `days` equally likely values)", &["birthday_paradox(23)", "birthday_paradox(70)", "birthday_paradox(10, 100)"], &["odds", "choose"]),
    doc("odds", "odds(p: num)", "a probability as \"1 in N\", next to a familiar event about as likely", &["odds(1 / 1000)", "odds(0.5)", "odds(2 ** -64)"], &["birthday_paradox"]),
    doc("zodiac", "zodiac(day?: date)", "the Western zodiac sign for a date (default today)", &[r#"zodiac(date("2000-01-01"))"#], &["chinese_zodiac"]),
    doc("chinese_zodiac", "chinese_zodiac(year: int|date)", "the Chinese zodiac element and animal; by Gregorian year, so January dates before Lunar New Year come out a year late", &["chinese_zodiac(2026)", "chinese_zodiac(1984)"], &["zodiac"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("roll", []) => roll("1d6")?,
        ("roll", [Str(s)]) => roll(s).map_err(|e| Fail::Arg(0, e))?,
        ("birthday_paradox", [Int(n, _), rest @ ..]) if rest.len() <= 1 => {
            let days = match rest {
                [] => 365,
                [Int(d, _)] if *d >= 1 => *d,
                _ => return Err(Fail::Arg(1, "days must be a positive int".into())),
            };
            if *n < 0 {
                return Err(Fail::Arg(0, format!("people must be 0 or more, got {n}")));
            }
            let none_shared: f64 = (0..*n.min(&(days + 1))).map(|i| (days - i) as f64 / days as f64).product();
            Float(1.0 - none_shared)
        }
        ("odds", [p]) => {
            let p = num(p).filter(|p| *p > 0.0 && *p <= 1.0).ok_or(Fail::Arg(0, "expected a probability above 0, up to 1".into()))?;
            let n = 1.0 / p;
            let (what, m) = EVENTS.iter().min_by(|a, b| ((a.1 / n).ln().abs()).total_cmp(&(b.1 / n).ln().abs())).unwrap();
            Value::str(format!("1 in {}: about as likely as {what} (1 in {})", big_num(n), big_num(*m)))
        }
        ("zodiac", [] | [Date(_)]) => {
            let d = match args {
                [Date(z)] => z.date(),
                _ => jiff::Zoned::now().date(),
            };
            let md = d.month() as i32 * 100 + d.day() as i32;
            // Each sign starts on the given month/day; Capricorn wraps over New Year.
            let sign = SIGNS.iter().rev().find(|s| md >= s.0).unwrap_or(&SIGNS[SIGNS.len() - 1]);
            Value::str(sign.1)
        }
        ("chinese_zodiac", [Int(y, _)]) => Value::str(chinese(*y)),
        ("chinese_zodiac", [Date(z)]) => Value::str(chinese(z.year() as i64)),
        _ => return Err(Fail::BadArgs),
    })
}

/// "4d6kh3+2-1d4": dice terms and constants, each with a sign.
fn roll(spec: &str) -> Result<Value, String> {
    // Spaces may surround + and -, but "2d6 3" must not read as 2d63.
    let s = regex::Regex::new(r"\s*([+-])\s*").unwrap().replace_all(spec.trim(), "$1").to_lowercase();
    let term = regex::Regex::new(r"^([+-]?)(?:(\d*)d(\d+|%)(?:k([hl]?)(\d+))?|(\d+))").unwrap();
    let bad = || format!("can't read {spec:?} as dice\nnote: like \"3d6+2\", \"d20\", \"4d6kh3\" or \"2d20kl1\"");
    let (mut rest, mut total, mut rolls) = (s.as_str(), 0i64, vec![]);
    if rest.is_empty() {
        return Err(bad());
    }
    while !rest.is_empty() {
        let c = term.captures(rest).filter(|c| !c[0].is_empty()).ok_or_else(bad)?;
        let sign = if &c[1] == "-" { -1 } else { 1 };
        if c.get(1).unwrap().as_str().is_empty() && rest.len() != s.len() {
            return Err(bad());
        }
        if let Some(k) = c.get(6) {
            total += sign * k.as_str().parse::<i64>().map_err(|_| bad())?;
        } else {
            let count: usize = if c[2].is_empty() { 1 } else { c[2].parse().map_err(|_| bad())? };
            let sides: i64 = if &c[3] == "%" { 100 } else { c[3].parse().map_err(|_| bad())? };
            if !(1..=1000).contains(&count) || !(1..=1_000_000).contains(&sides) {
                return Err("dice count must be 1-1000 and sides 1-1000000".into());
            }
            let mut dice: Vec<i64> = (0..count).map(|_| fastrand::i64(1..=sides)).collect();
            rolls.extend(dice.iter().map(|d| Value::int(*d)));
            if let Some(keep) = c.get(5) {
                let keep: usize = keep.as_str().parse().map_err(|_| bad())?;
                dice.sort_unstable();
                if &c[4] != "l" {
                    dice.reverse();
                }
                dice.truncate(keep);
            }
            total += sign * dice.iter().sum::<i64>();
        }
        rest = &rest[c[0].len()..];
    }
    let mut m = IndexMap::new();
    m.insert("total".to_string(), Value::int(total));
    m.insert("rolls".to_string(), Value::list(rolls));
    Ok(Value::map(m))
}

/// 1234567 → "1,234,567"; huge ones in words or scientific.
fn big_num(n: f64) -> String {
    if n < 1e15 {
        let s = format!("{:.0}", n);
        let mut out = String::new();
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i) % 3 == 0 {
                out.push(',');
            }
            out.push(c);
        }
        out
    } else {
        format!("{n:.2e}").replace("e", " × 10^")
    }
}

/// Commonly cited odds, rounded.
#[rustfmt::skip]
const EVENTS: &[(&str, f64)] = &[
    ("a coin landing heads", 2.0),
    ("rolling a six", 6.0),
    ("being dealt a pocket pair in Texas hold'em", 17.0),
    ("rolling double sixes", 36.0),
    ("identical twins", 250.0),
    ("flipping ten heads in a row", 1024.0),
    ("finding a four-leaf clover on the first try", 10_000.0),
    ("a hole in one for an amateur golfer", 12_500.0),
    ("being dealt a royal flush", 649_740.0),
    ("being struck by lightning this year in the US", 1_222_000.0),
    ("flipping 30 heads in a row", 1_073_741_824.0),
    ("winning the Powerball jackpot", 292_201_338.0),
    ("a perfect March Madness bracket", 9.2e18),
    ("guessing a random 128-bit key", 3.4e38),
    ("shuffling a deck into a given order", 8.07e67),
];

const SIGNS: [(i32, &str); 12] = [
    (120, "♒ Aquarius"),
    (219, "♓ Pisces"),
    (321, "♈ Aries"),
    (420, "♉ Taurus"),
    (521, "♊ Gemini"),
    (621, "♋ Cancer"),
    (723, "♌ Leo"),
    (823, "♍ Virgo"),
    (923, "♎ Libra"),
    (1023, "♏ Scorpio"),
    (1122, "♐ Sagittarius"),
    (1222, "♑ Capricorn"),
];

fn chinese(y: i64) -> String {
    const ANIMALS: [&str; 12] = ["Rat", "Ox", "Tiger", "Rabbit", "Dragon", "Snake", "Horse", "Goat", "Monkey", "Rooster", "Dog", "Pig"];
    const ELEMENTS: [&str; 5] = ["Wood", "Fire", "Earth", "Metal", "Water"];
    // 1984 was a Wood Rat, the start of a 60-year cycle.
    let k = (y - 1984).rem_euclid(60);
    format!("{} {}", ELEMENTS[(k / 2 % 5) as usize], ANIMALS[(k % 12) as usize])
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{eval, show, try_eval};
    use crate::value::Value;

    #[test]
    fn games() {
        for _ in 0..50 {
            assert!(matches!(eval(r#"roll("3d6+2").total"#), Value::Int(5..=20, _)));
            assert!(matches!(eval(r#"roll("4d6kh3").total"#), Value::Int(3..=18, _)));
            assert!(matches!(eval(r#"roll("2d20kl1-1").total"#), Value::Int(0..=19, _)));
            assert!(matches!(eval(r#"roll("d%").total"#), Value::Int(1..=100, _)));
        }
        assert_eq!(show(r#"[roll("4d6kh3").rolls.len, roll("5").total, roll().rolls.len]"#), "[4, 5, 1]");
        for bad in [r#"roll("")"#, r#"roll("3x6")"#, r#"roll("2d6 3")"#, r#"roll("0d6")"#, r#"roll("2d6++3")"#] {
            assert!(try_eval(r#"roll("2d6 + 3")"#).is_ok());
            assert!(try_eval(bad).is_err(), "{bad}");
        }
        assert_eq!(show("[birthday_paradox(23).round(3), birthday_paradox(1), birthday_paradox(366)]"), "[0.507, 0, 1]");
        assert_eq!(show("odds(1/1000)"), "1 in 1,000: about as likely as flipping ten heads in a row (1 in 1,024)");
        assert!(show("odds(2 ** -128)").contains("128-bit"));
        assert_eq!(
            show(r#"[zodiac(date("2000-01-01")), zodiac(date("2000-01-20")), zodiac(date("2000-03-20")), zodiac(date("2000-12-31"))]"#),
            r#"["♑ Capricorn", "♒ Aquarius", "♓ Pisces", "♑ Capricorn"]"#
        );
        assert_eq!(
            show("[chinese_zodiac(2026), chinese_zodiac(1984), chinese_zodiac(2000), chinese_zodiac(1900)]"),
            r#"["Fire Horse", "Wood Rat", "Metal Dragon", "Metal Rat"]"#
        );
    }
}
