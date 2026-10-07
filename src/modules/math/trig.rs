//! Trigonometry: angles in radians or any angle unit.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc, units};
use crate::value::{Value, num};

pub const MODULE: Module = Module {
    name: "trig",
    about: "sine, cosine and friends; angles as units",
    #[rustfmt::skip]
    examples: &[
        ("trig", &[
            ("trig takes angle units", "sin(30 deg)"),
            ("radians back to degrees", "atan2(1, 1) to deg"),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("sin", "sin(x: num|quantity)", "sine of radians or an angle unit", &["sin(30 deg)", "sin(pi / 2)"], &["cos", "tan", "asin"]),
    doc("cos", "cos(x: num|quantity)", "cosine of radians or an angle unit", &["cos(60 deg)"], &["sin", "tan", "acos"]),
    doc("tan", "tan(x: num|quantity)", "tangent of radians or an angle unit", &["tan(45 deg)"], &["sin", "cos", "atan"]),
    doc("asin", "asin(x: num)", "arcsine, as an angle", &["asin(1) to deg"], &["sin"]),
    doc("acos", "acos(x: num)", "arccosine, as an angle", &["acos(0) to deg"], &["cos"]),
    doc("atan", "atan(x: num)", "arctangent, as an angle", &["atan(1) to deg"], &["tan", "atan2"]),
    doc("atan2", "atan2(y: num|quantity, x: num|quantity)", "angle of the point (x, y), as an angle", &["atan2(1, -1) to deg"], &["atan", "hypot"]),
    doc("sinh", "sinh(x: num)", "hyperbolic sine", &["sinh(1)"], &["cosh", "tanh"]),
    doc("cosh", "cosh(x: num)", "hyperbolic cosine", &["cosh(1)"], &["sinh", "tanh"]),
    doc("tanh", "tanh(x: num)", "hyperbolic tangent", &["tanh(1)"], &["sinh", "cosh"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("sinh" | "cosh" | "tanh", [v]) if num(v).is_some() => {
            let x = num(v).unwrap();
            Float(match name {
                "sinh" => x.sinh(),
                "cosh" => x.cosh(),
                _ => x.tanh(),
            })
        }
        ("sin" | "cos" | "tan", [v]) => {
            let rad = match v {
                Qty(x, u) if u.dim() == units::unit("rad").unwrap().dim() => u.to_si(*x),
                _ => num(v).ok_or(Fail::BadArgs)?,
            };
            Float(match name {
                "sin" => rad.sin(),
                "cos" => rad.cos(),
                _ => rad.tan(),
            })
        }
        ("asin" | "acos" | "atan", [v]) if num(v).is_some() => {
            let x = num(v).unwrap();
            let rad = match name {
                "asin" => x.asin(),
                "acos" => x.acos(),
                _ => x.atan(),
            };
            Qty(rad, units::unit("rad").unwrap())
        }
        ("atan2", [y, x]) => {
            let (y, x) = match (y, x) {
                (Qty(y, u), Qty(x, w)) if u.dim() == w.dim() => (u.to_si(*y), w.to_si(*x)),
                _ => (num(y).ok_or(Fail::BadArgs)?, num(x).ok_or(Fail::BadArgs)?),
            };
            Qty(y.atan2(x), units::unit("rad").unwrap())
        }
        _ => return Err(Fail::BadArgs),
    })
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::show;

    #[test]
    fn trig() {
        assert_eq!(show("sin(30 deg)"), "0.5");
        assert_eq!(show("asin(1) to deg"), "90 deg");
        assert_eq!(show("atan2(1, -1) to deg"), "135 deg");
    }
}
