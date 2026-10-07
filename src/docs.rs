//! `zil --docs <dir>`: the mdBook reference, rendered from the same module tree as `help`, one page per module.
//! Example results are evaluated live; a test keeps the committed pages in sync, ignoring results.

use crate::help::{HIGHLIGHTS, eval};
use crate::modules::{ALL, Doc, Module, Section, modules, units};
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
        let file = dir.join(name);
        std::fs::create_dir_all(file.parent().unwrap())?;
        std::fs::write(file, page)?;
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
    for (path, m) in ALL.iter() {
        let depth = path.matches('.').count();
        writeln!(summary, "{}- [{}]({})", "  ".repeat(depth), m.name, file(path)).unwrap();
        writeln!(index, "| [{path}]({}) | {} |", file(path), m.about).unwrap();
        out.push((file(path), page(path, m, &note, live)));
    }
    out.push(("SUMMARY.md".into(), summary));
    out.push(("index.md".into(), index));
    out
}

/// A module's page file: `math.trig` is `math/trig.md`.
fn file(path: &str) -> String {
    format!("{}.md", path.replace('.', "/"))
}

fn page(path: &str, m: &Module, note: &str, live: bool) -> String {
    let mut s = format!("# {path}\n\n{}\n\n{note}", m.about);
    sections(&mut s, m.guide, live);
    let kinds = units::kinds(m.units);
    if !kinds.is_empty() {
        s.push_str("\n## Units\n\n| kind | names |\n|---|---|\n");
        for (kind, rows) in kinds {
            let names: Vec<_> = rows.iter().map(|r| r.0.replace(' ', ", ")).collect();
            writeln!(s, "| {kind} | {} |", names.join("; ")).unwrap();
        }
    }
    if !m.children.is_empty() {
        s.push_str("\n## Submodules\n\n| module | about |\n|---|---|\n");
        for c in m.children {
            writeln!(s, "| [{0}]({1}/{0}.md) | {2} |", c.name, m.name, c.about).unwrap();
        }
    }
    // Links from this page climb back to the book root first.
    let root = "../".repeat(path.matches('.').count());
    if !m.fns.is_empty() {
        s.push_str("\n## Functions\n\n| function | description |\n|---|---|\n");
        for f in m.fns {
            writeln!(s, "| [`{}`](#{}) | {} |", f.sig.replace('|', "\\|"), f.name, f.desc.replace('|', "\\|")).unwrap();
        }
        for f in m.fns {
            function(&mut s, f, &root, live);
        }
    }
    if !m.examples.is_empty() {
        s.push_str("\n## More examples\n");
        sections(&mut s, m.examples, live);
    }
    s
}

fn function(s: &mut String, f: &Doc, root: &str, live: bool) {
    writeln!(s, "\n### {}\n\n`{}`: {}", f.name, f.sig, f.desc).unwrap();
    if !f.examples.is_empty() || !f.shown.is_empty() {
        s.push_str("\n```zil\n");
        for ex in f.examples {
            example(s, ex, live);
        }
        for ex in f.shown {
            writeln!(s, "{ex}").unwrap();
        }
        s.push_str("```\n");
    }
    if !f.see.is_empty() {
        let home = |n: &str| modules().find(|m| m.fns.iter().any(|g| g.name == n)).map_or(String::new(), |m| file(crate::modules::path(m)));
        let links: Vec<_> = f.see.iter().map(|n| format!("[{n}]({root}{}#{n})", home(n))).collect();
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
            lines.map(|l| if l.starts_with(NOTE) { NOTE } else { l }).collect::<Vec<_>>().join(
                "
",
            )
        };
        for (name, page) in pages(false) {
            let disk = std::fs::read_to_string(dir.join(&name)).unwrap_or_default();
            assert!(strip(&disk) == strip(&page), "docs/book/src/{name} is stale: run `cargo run -- --docs docs/book/src`");
        }
    }
}
