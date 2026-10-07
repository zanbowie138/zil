//! Oracles of questionable reliability: fortune cookies, a magic 8-ball, coins, excuses.

use super::{Call, Doc, Fail, Module, doc};
use crate::interp::Interp;
use crate::lexer::Span;
use crate::value::Value;
use std::hash::{DefaultHasher, Hash, Hasher};

pub const MODULE: Module = Module {
    name: "fortune",
    about: "fortune cookies, a magic 8-ball, coin flips, excuses",
    #[rustfmt::skip]
    examples: &[
        ("fortune", &[
            ("ask the ball", r#"eight_ball("will it compile?")"#),
            ("best of five", "coin(5)"),
            ("standup", "excuse()"),
            ("should I ship it?", "yes_or_no()"),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("fortune", "fortune()", "a fortune cookie", &["fortune()"], &["eight_ball"]),
    doc("eight_ball", "eight_ball(question?)", "a magic 8-ball answer; the same question gets the same answer all day", &[r#"eight_ball("is it Friday?")"#], &["yes_or_no", "fortune"]),
    doc("coin", "coin(n?)", "heads or tails, or a list of n flips", &["coin()", "coin(3)"], &["rand"]),
    doc("yes_or_no", "yes_or_no(question?)", "yes or no; leans yes on Fridays", &["yes_or_no()"], &["eight_ball"]),
    doc("excuse", "excuse()", "why it doesn't work", &["excuse()"], &["fortune"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    let pick = |list: &[&'static str]| Value::str(list[fastrand::usize(..list.len())]);
    Ok(match (name, args) {
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
    "A refactor is in your future. It will touch more files than you think.",
    "The bug you seek is in the code you trust.",
    "You will soon delete a lot of code, and be happier for it.",
    "Today is a good day to read the error message.",
    "An off-by-one error is closer than it appears.",
    "Your tests pass. Do not ask why.",
    "Someone will thank you for a comment you wrote long ago.",
    "The cache is lying to you.",
    "You will find what you lost in the last place you look. Try git reflog.",
    "A wise developer once said: it works on my machine.",
    "Your next estimate will be off by a factor of pi.",
    "Good things come to those who rebase.",
    "Rest. The code will still be broken tomorrow.",
    "It was DNS.",
];

const EXCUSES: &[&str] = &[
    "It works on my machine.",
    "That's a known issue upstream.",
    "Must be a caching problem.",
    "Cosmic rays flipped a bit.",
    "It was fine in staging.",
    "The tests were flaky.",
    "Someone changed the config.",
    "That's not a bug, it's undocumented behavior.",
    "It must be a timezone thing.",
    "The compiler is being weird today.",
    "Mercury is in retrograde.",
    "I was told the requirements would not change.",
    "The intern had root access.",
    "It's a race condition. Probably.",
    "It was DNS.",
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
