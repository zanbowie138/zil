//! Placeholder text and names.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;

pub const MODULE: Module = Module {
    name: "placeholder",
    about: "lorem ipsum placeholder text, random team and person names",
    #[rustfmt::skip]
    examples: &[
        ("placeholder", &[
            ("a heading", "lorem(3)"),
            ("how long is it?", "lorem().split.len"),
            ("name the squad", "team_name()"),
            ("a test user", "person_name()"),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("lorem", "lorem(words?: int)", "the classic lorem ipsum paragraph, or its first n words (repeating past the end)", &["lorem(5)"], &["fortune"]),
    doc("team_name", "team_name()", "a random team name, like \"The Crimson Otters\"", &["team_name()"], &["person_name"]),
    doc("person_name", "person_name()", "a random first and last name", &["person_name()"], &["team_name"]),
];

const LOREM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. \
Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. \
Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

const TEAM_ADJECTIVES: &[&str] = &[
    "Crimson",
    "Mighty",
    "Flying",
    "Electric",
    "Golden",
    "Rogue",
    "Silent",
    "Thundering",
    "Cosmic",
    "Iron",
    "Wild",
    "Midnight",
    "Atomic",
    "Fearless",
    "Turbo",
    "Sneaky",
    "Caffeinated",
    "Undefeated",
    "Rusty",
    "Lazy",
];
const TEAM_NOUNS: &[&str] = &[
    "Otters",
    "Falcons",
    "Wolves",
    "Penguins",
    "Dragons",
    "Badgers",
    "Llamas",
    "Comets",
    "Pirates",
    "Ninjas",
    "Hedgehogs",
    "Krakens",
    "Squirrels",
    "Titans",
    "Ducks",
    "Honey Badgers",
    "Crabs",
    "Yetis",
    "Narwhals",
    "Goblins",
];
const FIRST_NAMES: &[&str] = &[
    "Alex", "Sam", "Jordan", "Taylor", "Morgan", "Casey", "Riley", "Jamie", "Avery", "Quinn", "Maria", "James", "Priya", "Wei", "Fatima", "Diego", "Aisha",
    "Liam", "Yuki", "Olga", "Kofi", "Elena", "Noah", "Sofia", "Omar", "Hana", "Lucas", "Amara", "Ivan", "Mei",
];
const LAST_NAMES: &[&str] = &[
    "Smith", "Garcia", "Nguyen", "Kim", "Patel", "Müller", "Rossi", "Silva", "Cohen", "Okafor", "Tanaka", "Ivanova", "Johnson", "Lopez", "Chen", "Haddad",
    "Novak", "Andersen", "Murphy", "Kowalski", "Dubois", "Santos", "Ahmed", "Brown", "Park", "Yilmaz", "Larsen", "Mensah",
];

fn pick(list: &[&'static str]) -> &'static str {
    list[fastrand::usize(..list.len())]
}

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    Ok(match (name, args) {
        ("lorem", []) => Value::str(LOREM),
        ("lorem", [Value::Int(n, _)]) if (1..=100_000).contains(n) => {
            let words: Vec<_> = LOREM.split(' ').cycle().take(*n as usize).collect();
            Value::str(format!("{}.", words.join(" ").trim_end_matches([',', '.'])))
        }
        ("team_name", []) => Value::str(format!("The {} {}", pick(TEAM_ADJECTIVES), pick(TEAM_NOUNS))),
        ("person_name", []) => Value::str(format!("{} {}", pick(FIRST_NAMES), pick(LAST_NAMES))),
        ("lorem", [Value::Int(n, _)]) => return Err(Fail::Arg(0, format!("word count must be 1 to 100000, got {n}"))),
        _ => return Err(Fail::BadArgs),
    })
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn lorem() {
        assert_eq!(show("lorem(5)"), "Lorem ipsum dolor sit amet.");
        assert_eq!(show("lorem(8)"), "Lorem ipsum dolor sit amet, consectetur adipiscing elit.");
        assert_eq!(show("lorem(70).split.len"), "70");
        assert!(try_eval("lorem(0)").is_err());
    }

    #[test]
    fn names() {
        assert!(show("team_name()").starts_with("The "));
        assert_eq!(show("person_name().split.len"), "2");
    }
}
