//! Text, mangled: SpOnGeBoB case, l33t, uwu, Pig Latin, zalgo, upside down, NATO spelling, emoji.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;

pub const MODULE: Module = Module {
    name: "mangle",
    about: "mock case, l33t, uwu, Pig Latin, zalgo, upside-down text, NATO spelling and emoji",
    #[rustfmt::skip]
    examples: &[
        ("mangle", &[
            ("reply to a bad take", r#""tabs are better than spaces".mock"#),
            ("spell it over the phone", r#""zil 2".nato"#),
            ("table flip", r#""(╯°□°)╯ " + "zil".flip"#),
            ("a commit message", r#"":rocket: ship it :tada:".emojify"#),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("mock", "mock(s: str)", "aLtErNaTiNg CaSe, for quoting someone sarcastically", &[r#""this is fine".mock"#], &["upper", "leet"]),
    doc("leet", "leet(s: str)", "l33t sp34k", &[r#""elite hacker".leet"#], &["mock"]),
    doc("uwu", "uwu(s: str)", "uwu-ify text", &[r#""hello friend, I really love this".uwu"#], &["mock"]),
    doc("pig_latin", "pig_latin(s: str)", "translate to Pig Latin, keeping capitals and punctuation", &[r#""Hello, string theory!".pig_latin"#], &["nato"]),
    doc("zalgo", "zalgo(s: str, marks?: int)", "Z̷a̸l̵g̶o̴ text: pile marks (default 3) onto every character", &[r#""he comes".zalgo"#, r#""ok".zalgo(1)"#], &["flip"]),
    doc("flip", "flip(s: str)", "turn text upside down", &[r#""hello world".flip"#], &["reverse", "zalgo"]),
    doc("nato", "nato(s: str)", "spell with the NATO phonetic alphabet", &[r#""SOS".nato"#, r#""b2b".nato"#], &["morse"]),
    doc("emoji", "emoji(name: str)", "an emoji by name, or nil", &[r#"emoji("taco")"#, r#"emoji("fire")"#], &["emojify"]),
    doc("emojify", "emojify(s: str)", "replace :name: codes with emoji; unknown codes stay", &[r#"":fire: deploy on friday :skull:".emojify"#], &["emoji"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("mock", [Str(s)]) => {
            let mut up = false;
            Value::str(
                s.chars()
                    .map(|c| {
                        if !c.is_alphabetic() {
                            return c.to_string();
                        }
                        up = !up;
                        if up { c.to_lowercase().to_string() } else { c.to_uppercase().to_string() }
                    })
                    .collect::<String>(),
            )
        }
        ("leet", [Str(s)]) => Value::str(
            s.chars()
                .map(|c| match c.to_ascii_lowercase() {
                    'a' => '4',
                    'e' => '3',
                    'i' => '1',
                    'o' => '0',
                    's' => '5',
                    't' => '7',
                    'b' => '8',
                    'g' => '9',
                    _ => c,
                })
                .collect::<String>(),
        ),
        ("uwu", [Str(s)]) => {
            let s = regex::Regex::new(r"[rl]").unwrap().replace_all(s, "w");
            let s = regex::Regex::new(r"[RL]").unwrap().replace_all(&s, "W");
            let s = regex::Regex::new(r"([nN])([aeiou])").unwrap().replace_all(&s, "${1}y$2");
            let s = s.replace("ove", "uv");
            let face = ["uwu", "owo", ">w<", "^w^"][s.len() % 4];
            Value::str(format!("{s} {face}"))
        }
        ("pig_latin", [Str(s)]) => {
            let re = regex::Regex::new(r"[A-Za-z]+").unwrap();
            Value::str(re.replace_all(s, |c: &regex::Captures| pig(&c[0])).into_owned())
        }
        ("zalgo", [Str(s), rest @ ..]) if rest.len() <= 1 => {
            let n = match rest {
                [] => 3,
                [Int(n, _)] if (0..=50).contains(n) => *n as usize,
                _ => return Err(Fail::Arg(1, "marks must be an int from 0 to 50".into())),
            };
            let mut out = String::new();
            for c in s.chars() {
                out.push(c);
                if !c.is_whitespace() {
                    out.extend((0..n).map(|_| char::from_u32(fastrand::u32(0x300..0x370)).unwrap()));
                }
            }
            Value::str(out)
        }
        ("flip", [Str(s)]) => Value::str(
            s.chars()
                .rev()
                .map(|c| {
                    FLIP.iter()
                        .find_map(|(a, b)| {
                            if *a == c {
                                Some(*b)
                            } else if *b == c {
                                Some(*a)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(c)
                })
                .collect::<String>(),
        ),
        ("nato", [Str(s)]) => {
            let words: Vec<&str> = s
                .chars()
                .filter_map(|c| match c.to_ascii_lowercase() {
                    c @ 'a'..='z' => Some(NATO[c as usize - 'a' as usize]),
                    c @ '0'..='9' => Some(DIGITS[c as usize - '0' as usize]),
                    ' ' => Some("/"),
                    _ => None,
                })
                .collect();
            Value::str(words.join(" "))
        }
        ("emoji", [Str(n)]) => emoji(n).map_or(Nil, Value::str),
        ("emojify", [Str(s)]) => {
            let re = regex::Regex::new(r":([a-z0-9_+-]+):").unwrap();
            Value::str(re.replace_all(s, |c: &regex::Captures| emoji(&c[1]).map_or_else(|| c[0].to_string(), String::from)).into_owned())
        }
        _ => return Err(Fail::BadArgs),
    })
}

/// One word: vowels add "way", consonant clusters move to the end before "ay"; a leading capital stays leading.
fn pig(w: &str) -> String {
    let lower = w.to_lowercase();
    let vowel = |c: char| "aeiou".contains(c);
    let split = match lower.find(vowel) {
        Some(0) => return format!("{w}way"),
        // "qu" stays together: "quiet" → "ietquay".
        Some(i) if lower[..i].ends_with('q') && lower[i..].starts_with('u') => i + 1,
        Some(i) => i,
        None => return format!("{w}ay"),
    };
    let out = format!("{}{}ay", &lower[split..], &lower[..split]);
    if w.chars().next().is_some_and(char::is_uppercase) {
        let mut c = out.chars();
        c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
    } else {
        out
    }
}

fn emoji(name: &str) -> Option<&'static str> {
    let name = name.to_lowercase().replace([' ', '-'], "_");
    EMOJI.iter().find(|e| e.0.split(' ').any(|n| n == name)).map(|e| e.1)
}

const NATO: [&str; 26] = [
    "Alfa", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel", "India", "Juliett", "Kilo", "Lima", "Mike", "November", "Oscar", "Papa", "Quebec",
    "Romeo", "Sierra", "Tango", "Uniform", "Victor", "Whiskey", "X-ray", "Yankee", "Zulu",
];
const DIGITS: [&str; 10] = ["Zero", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine"];

#[rustfmt::skip]
const FLIP: &[(char, char)] = &[
    ('a', 'ɐ'), ('b', 'q'), ('c', 'ɔ'), ('d', 'p'), ('e', 'ǝ'), ('f', 'ɟ'), ('g', 'ƃ'), ('h', 'ɥ'), ('i', 'ᴉ'), ('j', 'ɾ'), ('k', 'ʞ'), ('l', 'l'), ('m', 'ɯ'),
    ('n', 'u'), ('r', 'ɹ'), ('t', 'ʇ'), ('v', 'ʌ'), ('w', 'ʍ'), ('y', 'ʎ'), ('A', '∀'), ('C', 'Ɔ'), ('E', 'Ǝ'), ('F', 'Ⅎ'), ('G', '⅁'), ('J', 'ſ'), ('L', '˥'),
    ('M', 'W'), ('P', 'Ԁ'), ('T', '⊥'), ('U', '∩'), ('V', 'Λ'), ('Y', '⅄'), ('1', 'Ɩ'), ('2', 'ᄅ'), ('3', 'Ɛ'), ('4', 'ㄣ'), ('5', 'ϛ'), ('6', '9'), ('7', 'ㄥ'),
    ('.', '˙'), (',', '\''), ('?', '¿'), ('!', '¡'), ('(', ')'), ('[', ']'), ('{', '}'), ('<', '>'), ('_', '‾'), ('&', '⅋'),
];

#[rustfmt::skip]
const EMOJI: &[(&str, &str)] = &[
    ("smile", "😄"), ("grin", "😁"), ("joy laughing", "😂"), ("rofl", "🤣"), ("wink", "😉"), ("blush", "😊"), ("heart_eyes", "😍"), ("kiss", "😘"),
    ("thinking", "🤔"), ("neutral", "😐"), ("eyeroll", "🙄"), ("smirk", "😏"), ("sweat_smile", "😅"), ("cry", "😢"), ("sob", "😭"), ("angry", "😠"),
    ("rage", "😡"), ("scream", "😱"), ("sunglasses cool", "😎"), ("nerd", "🤓"), ("sleeping", "😴"), ("upside_down", "🙃"), ("shush", "🤫"), ("mindblown exploding_head", "🤯"),
    ("party partying", "🥳"), ("clown", "🤡"), ("skull dead", "💀"), ("ghost", "👻"), ("alien", "👽"), ("robot", "🤖"), ("poop", "💩"), ("devil", "😈"),
    ("thumbsup +1 yes", "👍"), ("thumbsdown -1 no", "👎"), ("clap", "👏"), ("wave", "👋"), ("pray thanks", "🙏"), ("muscle strong", "💪"), ("ok_hand", "👌"),
    ("point_up", "☝️"), ("eyes", "👀"), ("brain", "🧠"), ("facepalm", "🤦"), ("shrug", "🤷"),
    ("heart", "❤️"), ("broken_heart", "💔"), ("sparkles", "✨"), ("star", "⭐"), ("fire", "🔥"), ("100", "💯"), ("boom", "💥"), ("zap lightning", "⚡"),
    ("rainbow", "🌈"), ("sun sunny", "☀️"), ("cloud", "☁️"), ("rain", "🌧️"), ("snow snowflake", "❄️"), ("moon", "🌙"), ("earth globe", "🌍"),
    ("rocket", "🚀"), ("tada", "🎉"), ("confetti", "🎊"), ("gift", "🎁"), ("trophy", "🏆"), ("medal", "🏅"), ("crown", "👑"), ("gem", "💎"), ("moneybag money", "💰"),
    ("bulb idea", "💡"), ("bug", "🐛"), ("wrench", "🔧"), ("hammer", "🔨"), ("gear", "⚙️"), ("lock", "🔒"), ("key", "🔑"), ("bell", "🔔"), ("hourglass", "⏳"),
    ("warning", "⚠️"), ("x cross", "❌"), ("check white_check_mark", "✅"), ("question", "❓"), ("exclamation", "❗"), ("stop", "🛑"), ("construction", "🚧"),
    ("computer laptop", "💻"), ("phone", "📱"), ("email", "📧"), ("memo", "📝"), ("book", "📖"), ("calendar", "📅"), ("chart", "📈"), ("package", "📦"),
    ("coffee", "☕"), ("tea", "🍵"), ("beer", "🍺"), ("wine", "🍷"), ("pizza", "🍕"), ("taco", "🌮"), ("burger hamburger", "🍔"), ("fries", "🍟"),
    ("sushi", "🍣"), ("ramen", "🍜"), ("cake", "🎂"), ("cookie", "🍪"), ("doughnut donut", "🍩"), ("apple", "🍎"), ("banana", "🍌"), ("avocado", "🥑"),
    ("cat", "🐱"), ("dog", "🐶"), ("fox", "🦊"), ("panda", "🐼"), ("penguin", "🐧"), ("crab rust ferris", "🦀"), ("snake python", "🐍"), ("whale", "🐳"),
    ("unicorn", "🦄"), ("dragon", "🐉"), ("turtle", "🐢"), ("octopus", "🐙"), ("bee", "🐝"), ("owl", "🦉"), ("frog", "🐸"), ("monkey", "🐒"),
    ("tree", "🌳"), ("cactus", "🌵"), ("rose", "🌹"), ("sunflower", "🌻"), ("mushroom", "🍄"), ("music", "🎵"), ("guitar", "🎸"), ("video_game", "🎮"),
    ("dice die", "🎲"), ("soccer", "⚽"), ("basketball", "🏀"), ("car", "🚗"), ("bike", "🚲"), ("airplane plane", "✈️"), ("ship", "🚢"), ("house home", "🏠"),
];

#[cfg(test)]
mod tests {
    use crate::interp::tests::show;

    #[test]
    fn mangle() {
        assert_eq!(show(r#""this is fine".mock"#), "tHiS iS fInE");
        assert_eq!(show(r#""leet sauce".leet"#), "l337 54uc3");
        assert!(show(r#""hello, really".uwu"#).starts_with("hewwo, weawwy"));
        assert_eq!(show(r#""Hello, string theory! Eat quiet rhythm".pig_latin"#), "Ellohay, ingstray eorythay! Eatway ietquay rhythmay");
        assert_eq!(show(r#"["ab".zalgo(2).len, "a b".zalgo(1).len, "x".zalgo(0)]"#), r#"[6, 5, "x"]"#);
        assert_eq!(show(r#"["hello!".flip, "hello!".flip.flip]"#), r#"["¡ollǝɥ", "hello!"]"#);
        assert_eq!(show(r#""Hi 5".nato"#), "Hotel India / Five");
        assert_eq!(show(r#"[emoji("Taco"), emoji("rust"), emoji("nope")]"#), r#"["🌮", "🦀", nil]"#);
        assert_eq!(show(r#"":fire: :nope: ok".emojify"#), "🔥 :nope: ok");
    }
}
