//! Placeholder text.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;

pub const MODULE: Module = Module {
    name: "placeholder",
    about: "lorem ipsum placeholder text",
    #[rustfmt::skip]
    examples: &[
        ("placeholder", &[
            ("a heading", "lorem(3)"),
            ("how long is it?", "lorem().split.len"),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("lorem", "lorem(words?: int)", "the classic lorem ipsum paragraph, or its first n words (repeating past the end)", &["lorem(5)"], &["fortune"]),
];

const LOREM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. \
Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. \
Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    Ok(match (name, args) {
        ("lorem", []) => Value::str(LOREM),
        ("lorem", [Value::Int(n, _)]) if (1..=100_000).contains(n) => {
            let words: Vec<_> = LOREM.split(' ').cycle().take(*n as usize).collect();
            Value::str(format!("{}.", words.join(" ").trim_end_matches([',', '.'])))
        }
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
}
