//! `help`: rendered from each module's docs; every example shown is evaluated live in a fresh interpreter.

use crate::interp::Interp;
use crate::modules::MODULES;

pub fn help(topic: Option<&str>) -> Result<(), String> {
    let Some(topic) = topic else {
        println!("zil help: try help(upper), help(\"strings\"), help(\"km\"). In the REPL: help upper\n");
        let rows: Vec<_> = MODULES.iter().map(|m| format!("{:<8} {}", m.name, m.example)).collect();
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        for (row, m) in rows.iter().zip(MODULES) {
            println!("{row:<w$}  → {}", eval(m.example));
            let names: Vec<_> = m.fns.iter().take(12).map(|f| f.0).collect();
            let more = if m.fns.len() > names.len() { " ..." } else { "" };
            match names.is_empty() {
                true => println!("         help(\"{}\")", m.name),
                false => println!("         {}{more}", names.join(" ")),
            }
        }
        return Ok(());
    };
    if let Some((name, sig, desc, examples, see)) = MODULES.iter().flat_map(|m| m.fns).find(|f| f.0 == topic) {
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
    if let Some(m) = MODULES.iter().find(|m| m.name == topic && !m.fns.is_empty()) {
        let w = m.fns.iter().map(|f| f.1.chars().count()).max().unwrap_or(0);
        for (_, sig, desc, ..) in m.fns {
            println!("{sig:<w$}  {desc}");
        }
        println!("examples: help(name), e.g. help({})", m.fns[0].0);
        return Ok(());
    }
    if MODULES.iter().filter_map(|m| m.topic).any(|f| f(topic)) {
        return Ok(());
    }
    Err(format!("no help for {topic:?}; try help() for an overview"))
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
        for ex in fns.iter().flat_map(|f| f.3.iter()).chain(MODULES.iter().map(|m| &m.example)) {
            assert!(!eval(ex).starts_with("error:"), "{ex}: {}", eval(ex));
        }
        for f in &fns {
            for s in f.4 {
                assert!(fns.iter().any(|g| g.0 == *s), "{}: see also {s} missing", f.0);
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
