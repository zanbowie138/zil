//! Chance: random numbers, picks and shuffles, dice and coins, probabilities in plain words, and oracles of questionable reliability.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, Section, doc};
use crate::value::{Value, num};
use indexmap::IndexMap;
use std::hash::{DefaultHasher, Hash, Hasher};

pub const MODULE: Module = Module {
    name: "random",
    about: "random numbers, picks and shuffles; dice and coins; the birthday paradox and odds in plain words; fortune cookies, a magic 8-ball and excuses",
    examples: EXAMPLES,
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("numbers and lists", &["rand", "choice", "shuffle"]),
        ("dice and coins", &["roll", "coin"]),
        ("probability", &["birthday_paradox", "odds"]),
        ("oracles", &["fortune", "eight_ball", "yes_or_no", "excuse"]),
    ],
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const EXAMPLES: &[Section] = &[
    ("random", &[
        ("roll a die", "rand(1, 6)"),
        ("float in a range", "rand(1.0, 2.0)"),
        ("pick one", r#"["rock", "paper", "scissors"].choice"#),
        ("shuffle", "(1..=5).shuffle"),
        ("a D&D stat: 4d6, keep the best 3", r#"roll("4d6kh3")"#),
        ("attack with advantage", r#"roll("2d20kh1+5").total"#),
        ("best of five", "coin(5)"),
        ("a party of 23", "birthday_paradox(23)"),
        ("how unlikely is the jackpot?", "odds(1 / 292201338)"),
        ("ask the ball", r#"eight_ball("should I get a dog?")"#),
        ("why you're late", "excuse()"),
    ]),
];

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("rand", "rand() / rand(a: num, b: num)", "float in [0, 1), or a number from a to b inclusive", &["rand()", "rand(1, 6)"], &["choice", "shuffle"]),
    doc("choice", "choice(xs: list)", "random item", &[r#"["rock", "paper", "scissors"].choice"#], &["rand", "shuffle"]),
    doc("shuffle", "shuffle(xs: list)", "shuffled copy", &["(1..6).shuffle"], &["choice"]),
    doc("roll", "roll(dice?: str)", "roll dice like \"3d6+2\", \"d20\", \"4d6kh3\" (keep highest 3) or \"2d20kl1\"; returns {total, rolls}. Default 1d6", &[r#"roll("3d6+2")"#, r#"roll("d%").total"#], &["rand", "coin"]),
    doc("birthday_paradox", "birthday_paradox(people: int, days?: int)", "the chance at least two of n people share a birthday (or any of `days` equally likely values)", &["birthday_paradox(23)", "birthday_paradox(70)", "birthday_paradox(10, 100)"], &["odds", "choose"]),
    doc("odds", "odds(p: num)", "a probability as \"1 in N\", next to a familiar event about as likely", &["odds(1 / 1000)", "odds(0.5)", "odds(2 ** -64)"], &["birthday_paradox"]),
    doc("fortune", "fortune()", "a fortune cookie", &["fortune()"], &["eight_ball"]),
    doc("eight_ball", "eight_ball(question?: str)", "a magic 8-ball answer; the same question gets the same answer all day", &[r#"eight_ball("is it Friday?")"#], &["yes_or_no", "fortune"]),
    doc("coin", "coin(n?: int)", "heads or tails, or a list of n flips", &["coin()", "coin(3)"], &["rand"]),
    doc("yes_or_no", "yes_or_no(question?: str)", "yes or no; leans yes on Fridays", &["yes_or_no()"], &["eight_ball"]),
    doc("excuse", "excuse()", "why it doesn't work", &["excuse()"], &["fortune"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    let pick = |list: &[&'static str]| Value::str(list[fastrand::usize(..list.len())]);
    Ok(match (name, args) {
        ("rand", []) => Float(fastrand::f64()),
        ("rand", [Int(a, _), Int(b, _)]) if a <= b => Value::int(fastrand::i64(*a..=*b)),
        ("rand", [a, b]) if num(a).is_some() && num(b).is_some() => {
            let (a, b) = (num(a).unwrap(), num(b).unwrap());
            Float(a + fastrand::f64() * (b - a))
        }
        ("choice", [List(l)]) => {
            let l = l.borrow();
            if l.is_empty() {
                return Err("empty list".into());
            }
            l[fastrand::usize(..l.len())].clone()
        }
        ("shuffle", [List(l)]) => {
            let mut v = l.borrow().clone();
            fastrand::shuffle(&mut v);
            Value::list(v)
        }
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
        ("fortune", []) => pick(FORTUNES),
        ("eight_ball", []) => pick(EIGHT_BALL),
        ("eight_ball", [Str(q)]) => {
            let mut h = DefaultHasher::new();
            (q.trim().to_lowercase(), jiff::Zoned::now().date()).hash(&mut h);
            Value::str(EIGHT_BALL[h.finish() as usize % EIGHT_BALL.len()])
        }
        ("coin", []) => pick(&["heads", "tails"]),
        ("coin", [Int(n, _)]) if (0..=10_000).contains(n) => Value::list((0..*n).map(|_| pick(&["heads", "tails"])).collect()),
        ("yes_or_no", [] | [Str(_)]) => {
            let friday = jiff::Zoned::now().weekday() == jiff::civil::Weekday::Friday;
            Value::str(if fastrand::f64() < if friday { 0.8 } else { 0.5 } { "yes" } else { "no" })
        }
        ("excuse", []) => pick(EXCUSES),
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

const EIGHT_BALL: &[&str] = &[
    "It is certain.",
    "It is decidedly so.",
    "Without a doubt.",
    "Yes, definitely.",
    "You may rely on it.",
    "As I see it, yes.",
    "Most likely.",
    "Outlook good.",
    "Yes.",
    "Signs point to yes.",
    "Reply hazy, try again.",
    "Ask again later.",
    "Better not tell you now.",
    "Cannot predict now.",
    "Concentrate and ask again.",
    "Don't count on it.",
    "My reply is no.",
    "My sources say no.",
    "Outlook not so good.",
    "Very doubtful.",
];

const FORTUNES: &[&str] = &[
    "You will soon find money in a coat you forgot you owned.",
    "A nap is in your future. Do not fight it.",
    "The leftovers in the fridge are older than you think.",
    "You will walk into a room and forget why. Twice.",
    "Your houseplant believes in you. Water it anyway.",
    "A pigeon will judge you this week. Hold your head high.",
    "You will say \"you too\" to a waiter who said \"enjoy your meal\".",
    "Good news is on its way. So is spam.",
    "The socks you lost are living a better life without you.",
    "You will have a brilliant idea in the shower and lose it in the towel.",
    "Do not trust the last slice. It was left for a reason.",
    "An exciting opportunity awaits you behind the couch cushions.",
    "You will be asked to help someone move. Have an excuse ready.",
    "Your lucky number is 7. Or 4. Results may vary.",
    "A cat will ignore you today. Take it as a compliment.",
    "You will meet a tall, dark stranger. It is your shadow.",
    "The early bird gets the worm. You are free to sleep in.",
    "Help! I am trapped in a fortune cookie factory.",
    "That thing you've been putting off? Still there.",
    "Ignore the previous fortune.",
    "A great adventure awaits, right after this snack.",
    "You will finally fold that laundry. Not today, though.",
    "Somewhere, a dog is very happy to see someone. Be that someone.",
    "You are about to step on a Lego.",
    "Fortune not found. Please try again after coffee.",
    "Your future is bright. Wear sunscreen.",
    "Soon you will be hungry. This prediction is never wrong.",
    "Your microwave will beep at the worst possible moment.",
    "Today you will pull a door that says push.",
    "A small act of kindness will come back to you, possibly as a casserole.",
    "Beware of anyone who says \"quick question\".",
    "You will be right about something tiny and remember it forever.",
    "Someone you know is pretending to have read the book.",
    "Wealth is coming, mostly in the form of loyalty points.",
    "A package is in your future. It's the thing you forgot you ordered.",
    "Life is short. Eat dessert first.",
    "You will wave back at someone who was waving at the person behind you.",
    "The answer you seek is in the group chat you muted.",
    "A long-lost friend will reach out. They want to borrow something.",
    "Your karaoke moment is coming. Choose wisely.",
];

const EXCUSES: &[&str] = &[
    "My alarm didn't go off.",
    "The dog ate it.",
    "Traffic was unbelievable.",
    "Mercury is in retrograde.",
    "I thought it was tomorrow.",
    "My phone died, and a part of me died with it.",
    "I was abducted by aliens. They send their regards.",
    "A goose blocked the path and would not negotiate.",
    "I got stuck talking to a neighbor about their hedge.",
    "I was going to, but then I sat down.",
    "The cat was sitting on me and I couldn't move.",
    "I never got the message.",
    "It's been a really long week, and it's only Tuesday.",
    "I was busy being fabulous.",
    "There was a spider. I had to move out.",
    "I locked my keys in the car. With the car running.",
    "My horoscope told me to stay in bed.",
    "I was there in spirit.",
    "Autocorrect changed my plans.",
    "I was waiting for the right moment. It never came.",
    "Someone ate my lunch and I needed time to grieve.",
    "The weather looked suspicious.",
    "I got distracted by a really good sandwich.",
    "My twin did it.",
    "I was told there would be snacks.",
    "My socks didn't match and I couldn't go on.",
    "I was on hold for three hours.",
    "The GPS took me somewhere exciting.",
    "A squirrel stared me down and I lost my nerve.",
    "I was practicing self-care.",
    "The instructions were in a language I don't speak: flat-pack furniture.",
    "My plants needed emotional support.",
    "I tripped over nothing and needed a moment.",
    "Daylight saving time.",
    "I accidentally watched an entire season.",
    "It was like that when I got here.",
    "My grandma called, and nobody hangs up on grandma.",
    "I had a wardrobe malfunction of the soul.",
    "The bus left early. I saw it. It saw me.",
    "I'm not late, everyone else is early.",
];

#[cfg(test)]
mod tests {
    use crate::interp::tests::{eval, show, try_eval};
    use crate::value::Value;

    #[test]
    fn random() {
        assert!(matches!(eval("rand(1, 6)"), Value::Int(1..=6, _)));
        assert_eq!(eval("[1, 2, 3].shuffle.sort").to_string(), "[1, 2, 3]");
        assert!(try_eval("choice([])").is_err());
    }

    #[test]
    fn fortune() {
        assert_eq!(show(r#"eight_ball("same?")"#), show(r#"eight_ball(" SAME? ")"#));
        assert_eq!(show("coin(4).len"), "4");
        assert!(["yes", "no"].contains(&show("yes_or_no()").as_str()));
        assert!(try_eval("coin(-1)").is_err());
    }

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
    }
}
