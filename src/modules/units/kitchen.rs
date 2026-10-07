//! Kitchen and screen conversions: ingredient densities (cups to grams), decibels, CSS pixels.

use super::{LEN, Row, terms};
use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::{Value, num};

pub const MODULE: Module = Module {
    name: "kitchen",
    about: "cups to grams with ingredient densities, decibels, and CSS pixels (px)",
    #[rustfmt::skip]
    examples: &[
        ("kitchen", &[
            ("a cup of flour", r#"1 cup * density("flour") to g"#),
            ("250 g of sugar in cups", r#"250 g / density("sugar") to cup"#),
            ("a stick of butter", r#"0.5 cup * density("butter") to g"#),
            ("twice the power", "db(2)"),
            ("+3 dB louder means", "from_db(3)"),
            ("a 1080p screen at 96 dpi", "1080 px to in"),
        ]),
    ],
    fns: FNS,
    call,
    units: TABLE,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("density", "density(ingredient: str)", "a typical density in g/mL, so volume * density is mass; baking ingredients are spooned and leveled", &[r#"density("honey")"#, r#"2 tbsp * density("butter") to g"#], &["db"]),
    doc("db", "db(ratio: num, kind?: str)", "a power ratio in decibels (10 log10); kind \"amplitude\" for voltage or pressure ratios (20 log10)", &["db(2)", "db(1000)", r#"db(2, "amplitude")"#], &["from_db", "log"]),
    doc("from_db", "from_db(db: num, kind?: str)", "decibels back to a power ratio, or an amplitude ratio with \"amplitude\"", &["from_db(3)", "from_db(-6, \"amplitude\")"], &["db"]),
];

/// CSS reference pixel: 1/96 inch.
pub const TABLE: &[Row] = &[("px pixel pixels", 0.0254 / 96.0, 0.0, LEN, "pixel: the CSS reference pixel, 1/96 inch")];

// ponytail: one typical density per ingredient; brands and packing vary by ±10%.
#[rustfmt::skip]
const DENSITIES: &[(&str, f64)] = &[
    ("water", 1.0), ("milk", 1.03), ("cream heavy_cream", 0.99), ("yogurt", 1.03), ("buttermilk", 1.03),
    ("butter", 0.96), ("oil olive_oil vegetable_oil", 0.92), ("honey", 1.42), ("maple_syrup syrup", 1.32), ("molasses", 1.4),
    ("flour all_purpose_flour", 0.53), ("bread_flour", 0.55), ("whole_wheat_flour", 0.51), ("cake_flour", 0.48), ("almond_flour", 0.41),
    ("sugar granulated_sugar", 0.85), ("brown_sugar", 0.93), ("powdered_sugar icing_sugar", 0.51),
    ("cocoa cocoa_powder", 0.42), ("cornstarch", 0.54), ("baking_soda", 0.97), ("baking_powder", 0.81), ("salt table_salt", 1.2), ("kosher_salt", 0.6),
    ("rice", 0.85), ("oats rolled_oats", 0.38), ("chocolate_chips", 0.72), ("peanut_butter", 1.08), ("rice_flour", 0.67),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    let amplitude = |rest: &[Value]| match rest {
        [] => Ok(false),
        [Str(k)] if &**k == "amplitude" => Ok(true),
        [Str(k)] if &**k == "power" => Ok(false),
        _ => Err(Fail::Arg(1, "kind must be \"power\" or \"amplitude\"".into())),
    };
    Ok(match (name, args) {
        ("density", [Str(s)]) => {
            let key = s.trim().to_lowercase().replace([' ', '-'], "_");
            let key = key.trim_end_matches('s');
            let hit = DENSITIES.iter().find(|d| d.0.split(' ').any(|n| n.trim_end_matches('s') == key));
            let (_, g_ml) = hit.ok_or_else(|| {
                let names: Vec<_> = DENSITIES.iter().map(|d| d.0.split(' ').next().unwrap()).collect();
                Fail::Arg(0, format!("no density for {s:?}\nnote: known: {}", names.join(", ")))
            })?;
            Value::qty(*g_ml, terms("g mL^-1")?)
        }
        ("db", [x, rest @ ..]) if rest.len() <= 1 && num(x).is_some() => {
            let x = num(x).unwrap();
            if x <= 0.0 {
                return Err(Fail::Arg(0, format!("the ratio must be positive, got {x}")));
            }
            Float(if amplitude(rest)? { 20.0 } else { 10.0 } * x.log10())
        }
        ("from_db", [d, rest @ ..]) if rest.len() <= 1 && num(d).is_some() => Float(10f64.powf(num(d).unwrap() / if amplitude(rest)? { 20.0 } else { 10.0 })),
        _ => return Err(Fail::BadArgs),
    })
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn kitchen() {
        assert_eq!(show(r#"(1 cup * density("flour") to g).round"#), "125 g");
        assert_eq!(show(r#"(1 cup * density("Brown Sugar") to g).round"#), "220 g");
        assert_eq!(show(r#"(1 cup * density("chocolate chip") to g).round"#), "170 g");
        assert_eq!(show(r#"(250 g / density("sugar") to cup).round(2)"#), "1.24 cup");
        assert_eq!(show(r#"[db(2).round(2), db(100), db(10, "amplitude"), from_db(20), from_db(-20, "amplitude")]"#), "[3.01, 20, 20, 100, 0.1]");
        assert_eq!(show("[96 px to in, 1 in to px]"), "[1 in, 96 px]");
        assert!(try_eval(r#"density("unobtainium")"#).is_err());
        assert!(try_eval("db(0)").is_err());
        assert!(try_eval(r#"db(2, "loud")"#).is_err());
    }
}
