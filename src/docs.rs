//! `zil --docs <dir>`: the mdBook reference, rendered from the same module tree as `help`, one page per module.
//! Example results are evaluated live, so the pages are build output: CI regenerates them on deploy.

use crate::help::{ADVANCED, ADVANCED_UNRUN, HIGHLIGHTS, eval};
use crate::modules::{ALL, Doc, Module, Section, modules, units};
use std::fmt::Write;
use std::path::Path;

/// Prefix of the line holding an example's result.
const RESULT: &str = "# → ";
/// Prefix of the "generated on" note.
const NOTE: &str = "> Example results generated on ";

/// Writes every page into `dir` (an mdBook `src`).
pub fn write(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    for (name, page) in pages() {
        let file = dir.join(name);
        std::fs::create_dir_all(file.parent().unwrap())?;
        std::fs::write(file, page)?;
    }
    Ok(())
}

/// (file name, markdown) for every page.
fn pages() -> Vec<(String, String)> {
    let date = jiff::Zoned::now().date();
    let note = format!("{NOTE}{date}.\n> Ones using `now`, `today` or randomness will differ when you run them.\n");
    let mut summary = String::from("# Summary\n\n[Overview](index.md)\n\n- [Syntax](syntax.md)\n- [Advanced examples](advanced.md)\n");
    let mut index = format!("# zil\n\nAn expression calculator and scripting language with units, dates, exact fractions and big ints.\n\n{note}");
    sections(&mut index, HIGHLIGHTS);
    index.push_str("\n## Modules\n\n| module | about |\n|---|---|\n");
    let mut out = vec![("syntax.md".into(), "{{#include ../../syntax.md}}\n".into()), ("advanced.md".into(), advanced(&note))];
    for (path, m) in ALL.iter() {
        let depth = path.matches('.').count();
        writeln!(summary, "{}- [{}]({})", "  ".repeat(depth), m.name, file(path)).unwrap();
        writeln!(index, "| [{path}]({}) | {} |", file(path), m.about).unwrap();
        out.push((file(path), page(path, m, &note)));
    }
    out.push(("SUMMARY.md".into(), summary));
    out.push(("index.md".into(), index));
    out
}

/// Recipes combining several modules: a heading each, the modules it uses, then the script with its result.
fn advanced(note: &str) -> String {
    let mut s = format!("# Advanced examples\n\nLonger recipes that combine several modules.\n\n{note}");
    let unrun = std::iter::once((ADVANCED_UNRUN, false));
    for ((theme, recipes), run) in ADVANCED.iter().map(|t| (*t, true)).chain(unrun) {
        writeln!(s, "\n## {theme}").unwrap();
        for (title, uses, src) in recipes {
            writeln!(s, "\n### {title}\n\n*{uses}*\n\n```zil").unwrap();
            if run {
                example(&mut s, src);
            } else {
                writeln!(s, "{src}").unwrap();
            }
            s.push_str("```\n");
        }
    }
    s
}

/// A module's page file: `math.trig` is `math/trig.md`.
fn file(path: &str) -> String {
    format!("{}.md", path.replace('.', "/"))
}

fn page(path: &str, m: &Module, note: &str) -> String {
    let mut s = format!("# {path}\n\n{}\n\n{note}", m.about);
    sections(&mut s, m.guide);
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
            function(&mut s, f, &root);
        }
    }
    if !m.examples.is_empty() {
        s.push_str("\n## More examples\n");
        sections(&mut s, m.examples);
    }
    s
}

fn function(s: &mut String, f: &Doc, root: &str) {
    writeln!(s, "\n### {}\n\n`{}`: {}", f.name, f.sig, f.desc).unwrap();
    if !f.examples.is_empty() || !f.shown.is_empty() {
        s.push_str("\n```zil\n");
        for ex in f.examples {
            example(s, ex);
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
fn sections(s: &mut String, guide: &[Section]) {
    for (heading, rows) in guide {
        writeln!(s, "\n### {heading}\n\n```zil").unwrap();
        for (label, ex) in *rows {
            writeln!(s, "# {label}").unwrap();
            if !ex.is_empty() {
                example(s, ex);
            }
        }
        s.push_str("```\n");
    }
}

fn example(s: &mut String, ex: &str) {
    writeln!(s, "{ex}").unwrap();
    let v = eval(ex);
    if v != ex {
        writeln!(s, "{RESULT}{}", v.replace('\n', &format!("\n{RESULT}"))).unwrap();
    }
}
