//! Goofy units: bananas for scale, smoots, fortnights, and friends. Real units that happen to be silly too.

use super::units::{DATA, DIMS, Dim, d, unit};
use super::{Call, Doc, Fail, Module, doc};
use crate::interp::Interp;
use crate::lexer::Span;
use crate::value::Value;

pub const MODULE: Module = Module {
    name: "goofy_units",
    about: "bananas for scale, smoots, fortnights; every item also works as item_for_scale",
    #[rustfmt::skip]
    examples: &[
        ("goofy_units", &[
            ("height in bananas", "1.8 m to banana_for_scale"),
            ("the classic speed unit", "1 mph to furlong/fortnight"),
            ("Harvard Bridge", "364.4 smoot to m"),
            ("pick a fitting item", "8848 m.for_scale"),
            ("a lecture", "50 min to microcentury"),
            ("lunch energy", "1 big_mac to kWh"),
        ]),
    ],
    fns: FNS,
    call,
    topic: Some(topic),
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("for_scale", "for_scale(qty)", "the quantity in whichever goofy unit gives the most relatable count", &["1.8 m.for_scale", "70 kg.for_scale", "2 h.for_scale"], &[]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    match (name, args) {
        ("for_scale", [Value::Qty(x, u)]) => {
            let si = u.to_si(*x);
            // ponytail: "relatable" = count nearest 3 on a log scale; tune the target if picks feel off.
            let best = TABLE
                .iter()
                .filter(|row| row.3 == u.dim())
                .map(|row| (row.0.split(' ').next().unwrap(), si / row.1))
                .min_by(|a, b| (a.1.abs().log10() - 0.5).abs().total_cmp(&(b.1.abs().log10() - 0.5).abs()));
            let (name, n) = best.ok_or_else(|| format!("no goofy unit for {u}"))?;
            Ok(Value::Qty(n, unit(name)?))
        }
        _ => Err(Fail::BadArgs),
    }
}

/// `help("goofy_units")`: every goofy unit by kind.
fn topic(topic: &str) -> bool {
    if topic != "goofy_units" && topic != "goofy" {
        return false;
    }
    for (kind, dim) in DIMS {
        let names: Vec<_> = TABLE.iter().filter(|r| r.3 == *dim).map(|r| r.0.split(' ').next().unwrap()).collect();
        if !names.is_empty() {
            println!("  {kind:<14} {}", names.join(" "));
        }
    }
    true
}

const LEN: Dim = d(1, 0, 0, 0);
const MASS: Dim = d(0, 1, 0, 0);
const TIME: Dim = d(0, 0, 1, 0);
const AREA: Dim = d(2, 0, 0, 0);
const VOLUME: Dim = d(3, 0, 0, 0);
const SPEED: Dim = d(1, 0, -1, 0);
const ENERGY: Dim = d(2, 1, -2, 0);
const POWER: Dim = d(2, 1, -3, 0);

/// Same shape as `units::TABLE`. Sizes are typical, not official: your banana may vary.
#[rustfmt::skip]
pub const TABLE: &[(&str, f64, f64, Dim)] = &[
    ("banana bananas", 0.178, 0.0, LEN),
    ("credit_card credit_cards", 0.0856, 0.0, LEN),
    ("attoparsec attoparsecs", 0.030856775814913673, 0.0, LEN),
    ("beard_second beard_seconds", 5e-9, 0.0, LEN),
    ("smoot smoots", 1.7018, 0.0, LEN),
    ("altuve altuves", 1.65, 0.0, LEN),
    ("cubit cubits", 0.4572, 0.0, LEN),
    ("hand hands", 0.1016, 0.0, LEN),
    ("giraffe giraffes", 5.5, 0.0, LEN),
    ("school_bus school_buses", 12.2, 0.0, LEN),
    ("blue_whale blue_whales", 25.0, 0.0, LEN),
    ("football_field football_fields", 91.44, 0.0, LEN),
    ("statue_of_liberty", 93.0, 0.0, LEN),
    ("furlong furlongs", 201.168, 0.0, LEN),
    ("eiffel_tower eiffel_towers", 330.0, 0.0, LEN),
    ("sheppey sheppeys", 1408.176, 0.0, LEN),
    ("league leagues", 4828.032, 0.0, LEN),
    ("everest everests", 8848.86, 0.0, LEN),
    ("marathon marathons", 42195.0, 0.0, LEN),

    ("paperclip paperclips", 0.001, 0.0, MASS),
    ("bowling_ball bowling_balls", 7.26, 0.0, MASS),
    ("corgi corgis", 12.0, 0.0, MASS),
    ("slug slugs", 14.593903, 0.0, MASS),
    ("firkin firkins", 40.8233133, 0.0, MASS),
    ("grand_piano grand_pianos", 480.0, 0.0, MASS),
    ("honda_civic honda_civics", 1300.0, 0.0, MASS),
    ("elephant elephants", 6000.0, 0.0, MASS),

    ("shake shakes", 1e-8, 0.0, TIME),
    ("jiffy jiffies", 1.0 / 60.0, 0.0, TIME),
    ("nanocentury nanocenturies", 3.15576, 0.0, TIME),
    ("moment moments", 90.0, 0.0, TIME),
    ("microcentury microcenturies", 3155.76, 0.0, TIME),
    ("scaramucci scaramuccis", 11.0 * 86400.0, 0.0, TIME),
    ("fortnight fortnights", 14.0 * 86400.0, 0.0, TIME),
    ("dog_year dog_years", 31556952.0 / 7.0, 0.0, TIME),
    ("friedman friedmans", 31556952.0 / 2.0, 0.0, TIME),

    ("barn barns", 1e-28, 0.0, AREA),
    ("parking_space parking_spaces", 12.5, 0.0, AREA),
    ("tennis_court tennis_courts", 260.87, 0.0, AREA),
    ("rhode_island", 4.001e9, 0.0, AREA),
    ("wales", 2.0779e10, 0.0, AREA),

    ("barrel barrels bbl", 0.158987294928, 0.0, VOLUME),
    ("hogshead hogsheads", 0.238480942392, 0.0, VOLUME),
    ("bathtub bathtubs", 0.3, 0.0, VOLUME),
    ("olympic_pool olympic_pools", 2500.0, 0.0, VOLUME),

    ("snail snails", 0.001, 0.0, SPEED),
    ("mach", 343.0, 0.0, SPEED),

    ("big_mac big_macs", 563.0 * 4184.0, 0.0, ENERGY),
    ("tnt ton_tnt", 4.184e9, 0.0, ENERGY),
    ("hiroshima hiroshimas", 6.3e13, 0.0, ENERGY),

    ("donkeypower", 250.0, 0.0, POWER),
    ("toaster toasters", 1100.0, 0.0, POWER),

    ("nibble nibbles", 4.0, 0.0, DATA),
    ("floppy floppies", 1474560.0 * 8.0, 0.0, DATA),
    ("cdrom cdroms", 700.0 * 1048576.0 * 8.0, 0.0, DATA),
    ("dvd dvds", 4.7e9 * 8.0, 0.0, DATA),
    ("library_of_congress", 1e13 * 8.0, 0.0, DATA),
];

#[cfg(test)]
mod tests {
    use crate::interp::tests::show;

    #[test]
    fn goofy() {
        assert_eq!(show("1.78 m to banana_for_scale"), "10 banana");
        assert_eq!(show("2 banana to cm"), "35.6 cm");
        assert_eq!(show("1 fortnight to d"), "14 d");
        assert_eq!(show("1 m.for_scale"), "2.18723 cubit");
        assert_eq!(show("70 kg.for_scale"), "4.79652 slug");
        assert_eq!(show("1 furlong/fortnight to mm/min"), "9.97857 mm/min");
    }
}
