//! `help`: rendered from each module's docs; every example shown is evaluated live in a fresh interpreter.

use crate::interp::Interp;
use crate::modules::{Doc, MODULES, Module, Section};

/// Prints help for `topic` (an overview if `None`); false if nothing matched.
pub fn help(topic: Option<&str>) -> bool {
    let Some(topic) = topic else {
        overview();
        return true;
    };
    if let Some(Doc { name, sig, desc, examples, see }) = MODULES.iter().flat_map(|m| m.fns).find(|f| f.name == topic) {
        println!("{sig}  {desc}");
        show(examples.iter().map(|e| e.to_string()).collect());
        if name == &"help" {
            println!("  help upper      (REPL shorthand)");
        }
        if !see.is_empty() {
            println!("see also: {}", see.join(", "));
        }
        return true;
    }
    if topic == "examples" {
        examples();
        return true;
    }
    if let Some(m) = MODULES.iter().find(|m| m.name == topic) {
        page(m);
        return true;
    }
    if MODULES.iter().filter_map(|m| m.topic).any(|f| f(topic)) {
        return true;
    }
    let hits = search(topic);
    if hits.is_empty() {
        println!("no help for {topic:?}; try help() for an overview");
        return false;
    }
    let w = hits.iter().map(|f| f.sig.chars().count()).max().unwrap_or(0);
    for f in hits {
        println!("{:<w$}  {}", f.sig, f.desc);
    }
    true
}

/// The `help()` primer: things most languages can't do in one line. More per module in `help("examples")`.
#[rustfmt::skip]
pub const HIGHLIGHTS: &[Section] = &[(
    "a taste",
    &[
        ("day of the week", r#"date("2026-12-25").weekday"#),
        ("4th Thursday of November", r#"date(2026, 11, 1).nth_weekday(4, "thu")"#),
        ("3 business days later", r#"date("2026-12-24").add_workdays(3)"#),
        ("time zones", r#"date("2026-12-25 18:30") to "Asia/Tokyo""#),
        ("units combine and convert", "100 km / 2 h to mph"),
        ("split across units", "1.8 m to ft in"),
        ("exact decimals", "0.1 + 0.2 == 0.3"),
        ("big ints", "2 ** 100"),
        ("bits", "0xf0 to bits"),
        ("percentages", "80 + 15%"),
        ("numbers out of text", r#""a1b22c333".nums.sum"#),
        (r"syntax: x.f(y) is f(x, y)   |x| x * 2 is a lambda   xs |> sum pipes   # comments", ""),
    ],
)];

/// `help()`: what zil is, a taste of what it does, its modules, and how to dig deeper.
fn overview() {
    println!("zil: an expression calculator and scripting language with units, dates, exact fractions and big ints.");
    sections(HIGHLIGHTS);
    println!("\nmodules");
    let types = |m: &Module| m.guide.iter().find(|s| s.0 == "types").map(|s| s.1.iter().map(|r| r.0).collect::<Vec<_>>().join(" "));
    let w = MODULES.iter().map(|m| m.name.chars().count()).max().unwrap_or(0);
    for m in MODULES {
        let types = types(m).map_or(String::new(), |t| format!("  [{t}]"));
        println!("  {:<w$}  {}{types}", m.name, m.about);
    }
    println!(
        "
more help
  help(\"examples\")  a few dozen more, module by module
  help(\"strings\")   a module: its types, operators and every function
  help(upper)       a function: signature, live examples, related functions
  help(\"km\")        a unit, or a kind of unit like help(\"length\")
  help(\"sorting\")   search function names and descriptions
  help upper        REPL shorthand for help(upper)"
    );
}

/// `help("examples")`: every module's showcase, aligned per module so one long line doesn't stretch them all.
fn examples() {
    for m in MODULES {
        sections(m.examples);
    }
    println!("\nhelp(\"dates\") etc. for a module's full function list");
}

/// Builtins whose name, description or help-page group (`trig`, `encode`) mentions `topic`, or its stem.
// ponytail: crude suffix stemming ("sorting" → "sort"); real fuzzy matching if this misses too often.
fn search(topic: &str) -> Vec<&'static Doc> {
    let t = topic.to_lowercase();
    let stems: Vec<&str> = [Some(&t[..]), t.strip_suffix("ing"), t.strip_suffix("es"), t.strip_suffix('s')].into_iter().flatten().filter(|s| s.len() >= 2).collect();
    let hit = |m: &Module, f: &Doc| {
        m.groups.iter().any(|g| stems.contains(&g.0) && g.1.contains(&f.name)) || stems.iter().any(|s| f.name.contains(s) || f.desc.to_lowercase().contains(s))
    };
    MODULES.iter().flat_map(|m| m.fns.iter().map(move |f| (m, f))).filter(|(m, f)| hit(m, f)).map(|(_, f)| f).collect()
}

/// A module's page: what it adds (types, operators, conversions...), then its functions by group.
fn page(m: &Module) {
    println!("{}: {}", m.name, m.about);
    sections(m.guide);
    if let Some(topic) = m.topic {
        topic(m.name);
    }
    if m.fns.is_empty() {
        return;
    }
    println!(
        "
functions"
    );
    let all = [("", m.fns.iter().map(|f| f.name).collect::<Vec<_>>())];
    let groups: Vec<_> = m.groups.iter().map(|g| (g.0, g.1.to_vec())).collect();
    let groups = if groups.is_empty() { &all[..] } else { &groups[..] };
    let gw = groups.iter().map(|g| g.0.chars().count()).max().unwrap_or(0);
    for (name, fns) in groups {
        println!("  {}", format!("{name:<gw$}  {}", fns.join(" ")).trim_start());
    }
    println!("help(name) for details, e.g. help({})", m.fns[0].name);
}

/// Help sections as aligned `label  example  → result` rows; an empty example prints the label alone.
fn sections(guide: &[Section]) {
    let rows: Vec<_> = guide.iter().flat_map(|s| s.1).filter(|r| !r.1.is_empty()).collect();
    let lw = rows.iter().map(|r| r.0.chars().count()).max().unwrap_or(0);
    let ew = rows.iter().map(|r| r.1.chars().count()).max().unwrap_or(0);
    for (heading, rows) in guide {
        println!("
{heading}");
        for (label, ex) in *rows {
            match eval(ex) {
                _ if ex.is_empty() => println!("  {label}"),
                v if v == *ex => println!("  {label:<lw$}  {ex}"),
                v => println!("  {label:<lw$}  {ex:<ew$}  → {v}"),
            }
        }
    }
}

/// Print each example with its live result, arrows aligned.
pub fn show(examples: Vec<String>) {
    let w = examples.iter().map(|e| e.chars().count()).max().unwrap_or(0);
    for ex in &examples {
        println!("  {ex:<w$}  → {}", eval(ex));
    }
}

pub fn eval(src: &str) -> String {
    match crate::run(&mut Interp::new(), src) {
        Ok(v) => format!("{v:?}"),
        Err(e) => format!("error: {}", e.msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::units::DIMS;

    #[test]
    fn docs_examples_run() {
        let fns: Vec<_> = MODULES.iter().flat_map(|m| m.fns).collect();
        let guide = MODULES.iter().flat_map(|m| m.guide.iter().chain(m.examples)).chain(HIGHLIGHTS).flat_map(|s| s.1).map(|r| &r.1).filter(|e| !e.is_empty());
        for ex in fns.iter().flat_map(|f| f.examples.iter()).chain(guide) {
            assert!(!eval(ex).starts_with("error:"), "{ex}: {}", eval(ex));
        }
        for f in &fns {
            for s in f.see {
                assert!(fns.iter().any(|g| g.name == *s), "{}: see also {s} missing", f.name);
            }
        }
        for m in MODULES {
            assert!(help(Some(m.name)));
        }
        for (kind, _) in DIMS {
            assert!(help(Some(kind)));
        }
        assert!(help(Some("km")));
        assert!(!help(Some("nope")));
    }

    #[test]
    fn search_finds_by_description_group_and_stem() {
        let names = |t| search(t).iter().map(|f| f.name).collect::<Vec<_>>();
        assert!(names("sorting").contains(&"sort"));
        assert!(names("Uppercase").contains(&"upper"));
        assert!(names("trig").contains(&"atan2"));
        assert!(names("hash").contains(&"md5"));
        assert!(names("zzz").is_empty());
    }
}
