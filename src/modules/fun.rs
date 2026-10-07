//! Just for fun: oracles of questionable reliability (fortune cookies, a magic 8-ball, coins, excuses)
//! and Roman numerals; placeholder text, mangled text and games below.

pub mod games;
pub mod mangle;
pub mod placeholder;

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;
use std::hash::{DefaultHasher, Hash, Hasher};

pub const MODULE: Module = Module {
    name: "fun",
    about: "fortune cookies, a magic 8-ball, coin flips, excuses, Roman numerals; placeholder text, mangled text, dice and zodiac below",
    #[rustfmt::skip]
    examples: &[
        ("fortune", &[
            ("ask the ball", r#"eight_ball("should I get a dog?")"#),
            ("best of five", "coin(5)"),
            ("why you're late", "excuse()"),
            ("should I ship it?", "yes_or_no()"),
        ]),
        ("roman", &[
            ("this year", "2026 to roman"),
            ("Super Bowl math", r#"roman("LX") + 1 to roman"#),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[("conversions", &[("to roman", "1999 to roman")])],
    fns: FNS,
    call,
    targets: &[("roman", "roman")],
    children: &[placeholder::MODULE, mangle::MODULE, games::MODULE],
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("fortune", "fortune()", "a fortune cookie", &["fortune()"], &["eight_ball"]),
    doc("eight_ball", "eight_ball(question?: str)", "a magic 8-ball answer; the same question gets the same answer all day", &[r#"eight_ball("is it Friday?")"#], &["yes_or_no", "fortune"]),
    doc("coin", "coin(n?: int)", "heads or tails, or a list of n flips", &["coin()", "coin(3)"], &["rand"]),
    doc("yes_or_no", "yes_or_no(question?: str)", "yes or no; leans yes on Fridays", &["yes_or_no()"], &["eight_ball"]),
    doc("excuse", "excuse()", "why it doesn't work", &["excuse()"], &["fortune"]),
    doc("roman", "roman(v: int|str)", "an int 1-3999 as a Roman numeral, or a numeral back to an int; same as `n to roman`", &["roman(2026)", r#""MCMXCIX".roman"#], &[]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    let pick = |list: &[&'static str]| Value::str(list[fastrand::usize(..list.len())]);
    Ok(match (name, args) {
        ("roman", _) => return roman(args),
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
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn fortune() {
        assert_eq!(show(r#"eight_ball("same?")"#), show(r#"eight_ball(" SAME? ")"#));
        assert_eq!(show("coin(4).len"), "4");
        assert!(["yes", "no"].contains(&show("yes_or_no()").as_str()));
        assert!(try_eval("coin(-1)").is_err());
    }
}

const DIGITS: [(i64, &str); 13] =
    [(1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"), (50, "L"), (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")];

/// `roman(n)` or `roman("XIV")`.
fn roman(args: &[Value]) -> Call {
    match args {
        [Value::Int(n, _)] if (1..4000).contains(n) => Ok(Value::str(to_roman(*n))),
        [Value::Int(n, _)] => Err(format!("`{n}` has no Roman numeral\nnote: Roman numerals go from 1 to 3999").into()),
        [Value::Str(s)] => {
            let s = s.trim().to_uppercase();
            // Read greedily, then check by writing it back: rejects IIII, IC, VX and friends.
            let (mut n, mut rest) = (0, s.as_str());
            for (v, d) in DIGITS {
                while let Some(r) = rest.strip_prefix(d) {
                    n += v;
                    rest = r;
                }
            }
            if !rest.is_empty() || n == 0 || to_roman(n) != s {
                return Err(format!("{s:?} is not a Roman numeral").into());
            }
            Ok(Value::int(n))
        }
        _ => Err(Fail::BadArgs),
    }
}

fn to_roman(mut n: i64) -> String {
    let mut s = String::new();
    for (v, d) in DIGITS {
        while n >= v {
            s += d;
            n -= v;
        }
    }
    s
}

#[cfg(test)]
mod roman_tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn roman() {
        assert_eq!(show("2026 to roman"), "MMXXVI");
        assert_eq!(show("3999.roman"), "MMMCMXCIX");
        assert_eq!(show(r#""mcmxcix".roman"#), "1999");
        assert!((1..4000).all(|n| show(&format!("{n}.roman.roman")) == n.to_string()));
        assert!(try_eval(r#""IIII".roman"#).is_err());
        assert!(try_eval(r#""IC".roman"#).is_err());
        assert!(try_eval("0.roman").is_err());
    }
}
