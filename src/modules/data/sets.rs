//! Sets: unique values in insertion order, with set algebra as operators.

use crate::ast::BinOp;
use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Claim, Doc, Fail, Module, doc, short};
use crate::value::Value;
use indexmap::IndexSet;

pub const MODULE: Module = Module {
    name: "sets",
    about: "unique values in insertion order: union, intersection, difference",
    #[rustfmt::skip]
    examples: &[
        ("sets", &[
            ("distinct words", r#""the cat and the hat".split.set"#),
            ("in both", "set(1, 2, 3) & set(2, 3, 4)"),
            ("in either, not both", "set(1, 2, 3) ^ set(2, 3, 4)"),
            ("order doesn't matter", "set(1, 2) == set(2, 1)"),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[
        ("types", &[("set", "set(1, 2, 3)")]),
        ("operators", &[("a | b  union", "set(1, 2) | set(2, 3)"), ("a & b  intersection", "set(1, 2) & set(2, 3)"),
            ("a - b  difference", "set(1, 2) - set(2, 3)"), ("a ^ b  symmetric difference", "set(1, 2) ^ set(2, 3)"), ("x in s", "2 in set(1, 2)")]),
        ("holds", &[("int, str, bool and nil only", "")]),
        ("pretty", &[
            ("long: one item per line, indented, items pretty", ""),
            ("short: one line, items short", r#"pretty(set(1500, "a"), "short")"#),
        ]),
    ],
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("make", &["set"]),
        ("compare", &["is_subset", "is_superset", "is_disjoint"]),
        ("change", &["add", "remove"]),
    ],
    call,
    targets: &[("set", "set")],
    binary: Some(binary),
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("set", "set(xs: list|str|set) / set(a?: any, ...)", "a set of a list's items (or a string's characters), or of the arguments", &["[3, 1, 3, 2].set", "set(1, 2)", r#""hello" to set"#], &["unique", "list"]),
    doc("is_subset", "is_subset(a: set, b: set)", "true if every item of a is in b", &["set(1, 2).is_subset(set(1, 2, 3))"], &["is_superset"]),
    doc("is_superset", "is_superset(a: set, b: set)", "true if a has every item of b", &["set(1, 2, 3).is_superset(set(1, 5))"], &["is_subset"]),
    doc("is_disjoint", "is_disjoint(a: set, b: set)", "true if a and b share nothing", &["set(1, 2).is_disjoint(set(3))"], &["is_subset"]),
    doc("add", "add(s: set, v: any)", "add v in place and return the set", &["set(1).add(2)"], &["remove", "push"]),
    doc("remove", "remove(s: set, v: any)", "remove v in place (if there) and return the set", &["set(1, 2).remove(1)"], &["add", "del"]),
];

/// A set of `items`, blaming argument `arg` for the first one a set can't hold.
fn collect(items: impl IntoIterator<Item = Value>, arg: usize) -> Result<IndexSet<Value>, Fail> {
    items.into_iter().map(|v| if v.hashable() { Ok(v) } else { Err(Fail::Arg(arg, unhashable(&v))) }).collect()
}

fn unhashable(v: &Value) -> String {
    format!("sets hold int, str, bool or nil, got {}", short(v))
}

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("set", [List(l)]) => Value::set(collect(l.borrow().clone(), 0)?),
        ("set", [Str(s)]) => Value::set(s.chars().map(|c| Value::str(c.to_string())).collect()),
        ("set", [Set(s)]) => Value::set(s.borrow().clone()),
        ("set", items) => Value::set(collect(items.to_vec(), 0)?),
        ("is_subset", [Set(a), Set(b)]) => Bool(a.borrow().is_subset(&b.borrow())),
        ("is_superset", [Set(a), Set(b)]) => Bool(a.borrow().is_superset(&b.borrow())),
        ("is_disjoint", [Set(a), Set(b)]) => Bool(a.borrow().is_disjoint(&b.borrow())),
        ("add", [Set(_), v]) if !v.hashable() => return Err(Fail::Arg(1, unhashable(v))),
        ("add", [Set(s), v]) => {
            s.borrow_mut().insert(v.clone());
            Set(s.clone())
        }
        ("remove", [Set(s), v]) => {
            s.borrow_mut().shift_remove(v);
            Set(s.clone())
        }
        _ => return Err(Fail::BadArgs),
    })
}

/// `|` union, `&` intersection, `-` difference, `^` symmetric difference.
fn binary(op: BinOp, a: &Value, b: &Value) -> Claim {
    let (Value::Set(x), Value::Set(y)) = (a, b) else { return None };
    let (x, y) = (x.borrow(), y.borrow());
    Some(Ok(Value::set(match op {
        BinOp::BitOr => x.union(&y).cloned().collect(),
        BinOp::BitAnd => x.intersection(&y).cloned().collect(),
        BinOp::Sub => x.difference(&y).cloned().collect(),
        BinOp::BitXor => x.symmetric_difference(&y).cloned().collect(),
        _ => return None,
    })))
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn sets() {
        assert_eq!(show("[[3, 1, 3, 2].set, set(1, 2), set(), [1, 2] to set, \"aba\".set]"), r#"[set(3, 1, 2), set(1, 2), set(), set(1, 2), set("a", "b")]"#);
        assert_eq!(show("a = set(1, 2, 3); b = set(2, 3, 4); [a | b, a & b, a - b, a ^ b]"), "[set(1, 2, 3, 4), set(2, 3), set(1), set(1, 4)]");
        assert_eq!(show("[set(1, 2) == set(2, 1), set(1) == set(1, 2), 2 in set(1, 2), 0xff in set(255)]"), "[true, false, true, true]");
        assert_eq!(show("[set(1).is_subset(set(1, 2)), set(1).is_superset(set(1, 2)), set(1).is_disjoint(set(2))]"), "[true, false, true]");
        assert_eq!(show("s = set(1); s.add(2); s.add(1); s.remove(1); s.remove(9); s"), "set(2)");
        assert_eq!(show("s = set(3, 1, 2); [s.len, s.contains(1), s.sort, s.list, s.sort_desc]"), "[3, true, [1, 2, 3], [3, 1, 2], [3, 2, 1]]");
        assert_eq!(show("t = 0; for x in set(1, 2, 2) { t += x }; t"), "3");
        assert_eq!(show(r#"s = set(1, "a", nil, true); str(s).parse == s"#), "true");
        for bad in ["set([1], 2)", "set(1).add([2])", "set(1.5)", "[[1]].set"] {
            assert!(try_eval(bad).is_err(), "{bad}");
        }
    }
}
