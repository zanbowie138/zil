//! `zil --docs <dir>`: the mdBook reference, rendered from the same `MODULES` tables as `help`.
//! Example results are evaluated live; a test keeps the committed pages in sync, ignoring results.

use crate::help::{HIGHLIGHTS, eval};
use crate::modules::{Doc, MODULES, Module, Section, goofy_units, units};
use std::fmt::Write;
use std::path::Path;

/// Prefix of the line holding an example's result, so stale checks can drop it.
const RESULT: &str = "# → ";
/// Prefix of the "generated on" note, also dropped by stale checks.
const NOTE: &str = "> Example results generated on ";

/// Writes every page into `dir` (an mdBook `src`).
pub fn write(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    for (name, page) in pages(true) {
        std::fs::write(dir.join(name), page)?;
    }
    Ok(())
}

/// (file name, markdown) for every page; `live: false` leaves out results so pages are deterministic.
fn pages(live: bool) -> Vec<(String, String)> {
    let date = if live { jiff::Zoned::now().date().to_string() } else { String::new() };
    let note = format!("{NOTE}{date}.\n> Ones using `now`, `today` or randomness will differ when you run them.\n");
    let mut summary = String::from("# Summary\n\n[Overview](index.md)\n\n- [Syntax](syntax.md)\n");
    let mut index = format!("# zil\n\nAn expression calculator and scripting language with units, dates, exact fractions and big ints.\n\n{note}");
    sections(&mut index, HIGHLIGHTS, live);
    index.push_str("\n## Modules\n\n| module | about |\n|---|---|\n");
    let mut out = vec![("syntax.md".into(), "{{#include ../../syntax.md}}\n".into())];
    for m in MODULES {
        writeln!(summary, "- [{0}]({0}.md)", m.name).unwrap();
        writeln!(index, "| [{0}]({0}.md) | {1} |", m.name, m.about).unwrap();
        out.push((format!("{}.md", m.name), page(m, &note, live)));
    }
    out.push(("SUMMARY.md".into(), summary));
    out.push(("index.md".into(), index));
    out
}

fn page(m: &Module, note: &str, live: bool) -> String {
    let mut s = format!("# {}\n\n{}\n\n{note}", m.name, m.about);
    sections(&mut s, m.guide, live);
    let table = match m.name {
        "units" => units::TABLE,
        "goofy_units" => goofy_units::TABLE,
        _ => &[],
    };
    let kinds: Vec<_> = units::DIMS.iter().map(|(kind, dim)| (kind, table.iter().filter(|r| r.3 == *dim).map(|r| r.0.replace(' ', ", ")).collect::<Vec<_>>())).filter(|k| !k.1.is_empty()).collect();
    if !kinds.is_empty() {
        s.push_str("\n## Units\n\n| kind | names |\n|---|---|\n");
        for (kind, names) in kinds {
            writeln!(s, "| {kind} | {} |", names.join("; ")).unwrap();
        }
    }
    if !m.fns.is_empty() {
        s.push_str("\n## Functions\n\n| function | description |\n|---|---|\n");
        for f in m.fns {
            writeln!(s, "| [`{}`](#{}) | {} |", f.sig.replace('|', "\\|"), f.name, f.desc.replace('|', "\\|")).unwrap();
        }
        for f in m.fns {
            function(&mut s, f, live);
        }
    }
    if !m.examples.is_empty() {
        s.push_str("\n## More examples\n");
        sections(&mut s, m.examples, live);
    }
    s
}

fn function(s: &mut String, f: &Doc, live: bool) {
    writeln!(s, "\n### {}\n\n`{}`: {}", f.name, f.sig, f.desc).unwrap();
    if !f.examples.is_empty() {
        s.push_str("\n```zil\n");
        for ex in f.examples {
            example(s, ex, live);
        }
        s.push_str("```\n");
    }
    if !f.see.is_empty() {
        let links: Vec<_> = f.see.iter().map(|n| format!("[{n}]({}.md#{n})", MODULES.iter().find(|m| m.fns.iter().any(|g| g.name == *n)).map_or("", |m| m.name))).collect();
        writeln!(s, "\nSee also: {}", links.join(", ")).unwrap();
    }
}

/// Each section as a heading and a code block of `# label`, example, result.
fn sections(s: &mut String, guide: &[Section], live: bool) {
    for (heading, rows) in guide {
        writeln!(s, "\n### {heading}\n\n```zil").unwrap();
        for (label, ex) in *rows {
            writeln!(s, "# {label}").unwrap();
            if !ex.is_empty() {
                example(s, ex, live);
            }
        }
        s.push_str("```\n");
    }
}

fn example(s: &mut String, ex: &str, live: bool) {
    writeln!(s, "{ex}").unwrap();
    let v = if live { eval(ex) } else { return };
    if v != ex {
        writeln!(s, "{RESULT}{}", v.replace('\n', &format!("\n{RESULT}"))).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn book_is_current() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/book/src");
        let strip = |page: &str| -> String {
            let lines = page.lines().filter(|l| !l.starts_with(RESULT));
            lines.map(|l| if l.starts_with(NOTE) { NOTE } else { l }).collect::<Vec<_>>().join("
")
        };
        for (name, page) in pages(false) {
            let disk = std::fs::read_to_string(dir.join(&name)).unwrap_or_default();
            assert!(strip(&disk) == strip(&page), "docs/book/src/{name} is stale: run `cargo run -- --docs docs/book/src`");
        }
    }
}
