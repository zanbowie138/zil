//! Maps: keys and values.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;

pub const MODULE: Module = Module {
    name: "maps",
    about: "keys and values of maps",
    #[rustfmt::skip]
    examples: &[
        ("maps", &[
            ("sum a map", "{a: 1, b: 2}.values.sum"),
            ("as [key, value] pairs", "{a: 1, b: 2}.list"),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("keys", "keys(map)", "list of map keys", &["{a: 1, b: 2}.keys"], &["values"]),
    doc("values", "values(map)", "list of map values", &["{a: 1, b: 2}.values"], &["keys"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    Ok(match (name, args) {
        ("keys", [Value::Map(m)]) => Value::list(m.borrow().keys().map(|k| Value::str(k.as_str())).collect()),
        ("values", [Value::Map(m)]) => Value::list(m.borrow().values().cloned().collect()),
        _ => return Err(Fail::BadArgs),
    })
}
