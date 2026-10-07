//! `help`: rendered from each module's docs; every example shown is evaluated live in a fresh interpreter.

use crate::interp::Interp;
use crate::modules::{Doc, MODULES, Module};

pub fn help(topic: Option<&str>) -> Result<(), String> {
    let Some(topic) = topic else {
        println!("zil help: try help(upper), help(\"strings\"), help(\"km\"). In the REPL: help upper\n");
        let types = |m: &Module| m.guide.iter().find(|s| s.0 == "types").map(|s| s.1.iter().map(|r| r.0).collect::<Vec<_>>().join(" "));
        let names: Vec<_> = MODULES.iter().map(|m| types(m).map_or(m.name.into(), |t| format!("{} ({t})", m.name))).collect();
        let rows: Vec<_> = names.iter().zip(MODULES).map(|(n, m)| format!("{n}  {}", m.example)).collect();
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        for (row, m) in rows.iter().zip(MODULES) {
            println!("{row:<w$}  → {}", eval(m.example));
            let names: Vec<_> = m.fns.iter().take(12).map(|f| f.name).collect();
            let more = if m.fns.len() > names.len() { " ..." } else { "" };
            match names.is_empty() {
                true => println!("  help(\"{}\")", m.name),
                false => println!("  {}{more}", names.join(" ")),
            }
        }
        return Ok(());
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
        return Ok(());
    }
    if let Some(m) = MODULES.iter().find(|m| m.name == topic) {
        page(m);
        return Ok(());
    }
    if MODULES.iter().filter_map(|m| m.topic).any(|f| f(topic)) {
        return Ok(());
    }
    Err(format!("no help for {topic:?}; try help() for an overview"))
}

/// A module's page: what it adds (types, operators, conversions...), then its functions by group.
fn page(m: &Module) {
    println!("{}: {}", m.name, m.about);
    let rows: Vec<_> = m.guide.iter().flat_map(|s| s.1).filter(|r| !r.1.is_empty()).collect();
    let lw = rows.iter().map(|r| r.0.chars().count()).max().unwrap_or(0);
    let ew = rows.iter().map(|r| r.1.chars().count()).max().unwrap_or(0);
    for (heading, rows) in m.guide {
        println!(
            "
{heading}"
        );
        for (label, ex) in *rows {
            match eval(ex) {
                _ if ex.is_empty() => println!("  {label}"),
                v if v == *ex => println!("  {label:<lw$}  {ex}"),
                v => println!("  {label:<lw$}  {ex:<ew$}  → {v}"),
            }
        }
    }
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

/// Print each example with its live result, arrows aligned.
pub fn show(examples: Vec<String>) {
    let w = examples.iter().map(|e| e.chars().count()).max().unwrap_or(0);
    for ex in &examples {
        println!("  {ex:<w$}  → {}", eval(ex));
    }
}

fn eval(src: &str) -> String {
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
        let guide = MODULES.iter().flat_map(|m| m.guide).flat_map(|s| s.1).map(|r| &r.1).filter(|e| !e.is_empty());
        for ex in fns.iter().flat_map(|f| f.examples.iter()).chain(MODULES.iter().map(|m| &m.example)).chain(guide) {
            assert!(!eval(ex).starts_with("error:"), "{ex}: {}", eval(ex));
        }
        for f in &fns {
            for s in f.see {
                assert!(fns.iter().any(|g| g.name == *s), "{}: see also {s} missing", f.name);
            }
        }
        for m in MODULES {
            help(Some(m.name)).unwrap();
        }
        for (kind, _) in DIMS {
            help(Some(kind)).unwrap();
        }
        help(Some("km")).unwrap();
        assert!(help(Some("nope")).is_err());
    }
}
