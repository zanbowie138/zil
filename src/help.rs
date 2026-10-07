//! `help`: rendered from each module's docs; every example shown is evaluated live in a fresh interpreter.

use crate::interp::Interp;
use crate::modules::{Doc, MODULES, Module, Section};
use crate::{BOLD, DIM, RESET};

/// Prints help for `topic` (an overview if `None`); false if nothing matched.
pub fn help(topic: Option<&str>) -> bool {
    let Some(topic) = topic else {
        overview();
        return true;
    };
    if let Some((m, f)) = docs().find(|(_, f)| f.name == topic) {
        function(m, f);
        return true;
    }
    if topic == "examples" {
        examples();
        return true;
    }
    if topic == "syntax" {
        sections(SYNTAX);
        return true;
    }
    if let Some(m) = MODULES.iter().find(|m| m.name == topic) {
        page(m);
        return true;
    }
    if MODULES.iter().filter_map(|m| m.topic).any(|f| f(topic)) || type_page(topic) {
        return true;
    }
    let hits = search(topic);
    if hits.is_empty() {
        let near: Vec<_> = near(topic).iter().map(|n| paint(BOLD, n)).collect();
        let hint = if near.is_empty() { String::new() } else { format!("; did you mean {}?", near.join(", ")) };
        println!("no help for {topic:?}{hint}  help() for an overview");
        return false;
    }
    listing(hits);
    true
}

/// `help(value)`: every builtin whose first parameter takes that type; false for a type with none.
pub fn type_page(ty: &str) -> bool {
    let Some((_, params)) = TAKES.iter().find(|t| t.0 == ty) else {
        return false;
    };
    let first = |sig: &str| sig.split_once('(').and_then(|r| r.1.split([',', ')']).next()).unwrap_or("").trim().to_string();
    println!("functions taking a {} first, so x.f(...) works\n", paint(BOLD, ty));
    listing(docs().filter(|(_, f)| params.contains(&first(f.sig).as_str())).collect());
    true
}

/// Types, and the first-parameter names in signatures that take them.
// ponytail: leans on the sigs' naming convention (d, s, x...); add a `takes` field to Doc if it drifts.
const TAKES: &[(&str, &[&str])] = &[
    ("date", &["d"]),
    ("str", &["s", "c", "ip", "block", "path"]),
    ("int", &["x", "n"]),
    ("frac", &["x"]),
    ("float", &["x"]),
    ("quantity", &["x", "qty", "duration", "sum", "loan", "balance", "cost", "rate"]),
    ("list", &["list"]),
    ("map", &["map"]),
];

/// The overview's module groups; every module on exactly one shelf.
const SHELVES: &[(&str, &[&str])] = &[
    ("data", &["strings", "lists", "math", "general"]),
    ("time & measure", &["dates", "units", "money"]),
    ("tools", &["random", "colors", "net", "binary"]),
    ("fun", &["goofy_units", "numerals", "ciphers", "oracles"]),
];

/// Things most languages can't do in one line: heads `help("examples")` and the mdBook index.
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

/// `help("syntax")`: the language at a glance.
#[rustfmt::skip]
const SYNTAX: &[Section] = &[
    ("values", &[
        ("decimals are exact", "0.1 + 0.2"),
        ("int / int is a fraction", "7 / 2"),
        ("floor division", "7 // 2"),
        ("power (^ is xor)", "2 ** 10"),
        ("bases are kept", "0xff + 1"),
        ("units attach to numbers", "60 km/h to m/s"),
        ("percentages", "80 + 15%"),
        ("interpolation", r#""2 + 2 = {2 + 2}""#),
        ("slices, from the end", r#""hello"[-3..]"#),
        ("regex literals", r#""a1b22".find_all(r"\d+")"#),
        ("ranges", "1..=5"),
        ("maps", "{a: 1, b: 2}.b"),
    ]),
    ("calls", &[
        ("x.f(y) is f(x, y)", r#""a-b".split("-")"#),
        ("no args, no parens", r#""hi".upper"#),
        ("pipes", "[3, 1, 2] |> sort"),
        ("_ is the piped value", "3.14159 |> round(_, 2)"),
        ("lambdas", "[1, 2, 3].map(|x| x * 10)"),
        ("to converts", "255 to hex"),
    ]),
    ("statements", &[
        ("assignment", "x = 5; x += 2; x"),
        ("if is an expression", r#"if 3 > 2 { "yes" } else { "no" }"#),
        ("for", "t = 0; for i in 1..=4 { t += i }; t"),
        ("while", "n = 3; while n > 0 { n -= 1 }; n"),
        ("functions", "f = fn(x) { return x * 2 }; f(4)"),
        ("comparisons chain", "x = 5; 0 < x <= 10"),
        ("# comments run to the end of the line; a newline or ; ends a statement", ""),
    ]),
    ("more", &[("full reference, with operator precedence: the Syntax page from zil --docs", "")]),
];

/// `help()`: what zil is, its modules by shelf, and how to dig deeper.
fn overview() {
    println!("{}: an expression calculator and scripting language with units, dates, exact fractions and big ints.", paint(BOLD, "zil"));
    let types = |m: &Module| m.guide.iter().find(|s| s.0 == "types").map(|s| s.1.iter().map(|r| r.0).collect::<Vec<_>>().join(" "));
    let w = MODULES.iter().map(|m| m.name.chars().count()).max().unwrap_or(0);
    for (shelf, names) in SHELVES {
        println!("\n{}", paint(BOLD, shelf));
        for m in names.iter().filter_map(|n| MODULES.iter().find(|m| m.name == *n)) {
            let types = types(m).map_or(String::new(), |t| format!("  {}", paint(DIM, &format!("[{t}]"))));
            println!("  {}{}  {}{types}", m.name, pad(m.name, w), m.about);
        }
    }
    println!("\n{}", paint(BOLD, "more help"));
    let rows = [
        (r#"help("examples")"#, "start here: a few dozen one-liners, module by module"),
        (r#"help("syntax")"#, "the language at a glance"),
        (r#"help("strings")"#, "a module: its types, operators and every function"),
        ("help(upper)", "a function: signature, live examples, related functions"),
        ("help(today)", "a value: every function that takes its type"),
        (r#"help("km")"#, r#"a unit, or a kind of unit like help("length")"#),
        (r#"help("sorting")"#, "search function names and descriptions"),
        ("help upper", "REPL shorthand for help(upper)"),
    ];
    let w = rows.iter().map(|r| r.0.chars().count()).max().unwrap_or(0);
    for (ex, what) in rows {
        println!("  {}{}  {}", code(ex), pad(ex, w), paint(DIM, what));
    }
}

/// `help("examples")`: the highlights, then every module's showcase, aligned per module so one long line doesn't stretch them all.
fn examples() {
    sections(HIGHLIGHTS);
    for m in MODULES {
        sections(m.examples);
    }
    println!("\n{}", paint(DIM, r#"help("dates") etc. for a module's full function list"#));
}

/// Every builtin, with its module.
fn docs() -> impl Iterator<Item = (&'static Module, &'static Doc)> {
    MODULES.iter().flat_map(|m| m.fns.iter().map(move |f| (m, f)))
}

/// A function's page: where it lives, signature, live examples, related functions.
fn function(m: &Module, f: &Doc) {
    let group = m.groups.iter().find(|g| g.1.contains(&f.name)).map_or(String::new(), |g| format!(" › {}", g.0));
    println!("{}", paint(DIM, &format!("{}{group}", m.name)));
    println!("{}  {}", sig(f), f.desc);
    show(f.examples.iter().map(|e| e.to_string()).collect());
    if f.name == "help" {
        println!("  {}  {}", code("help upper"), paint(DIM, "(REPL shorthand)"));
    }
    if !f.see.is_empty() {
        println!("{} {}", paint(DIM, "see also:"), f.see.join(", "));
    }
}

/// Search or type-page hits, grouped under their module.
fn listing(hits: Vec<(&Module, &Doc)>) {
    let w = hits.iter().map(|h| h.1.sig.chars().count()).max().unwrap_or(0);
    let mut last = "";
    for (m, f) in hits {
        if m.name != last {
            println!("{}", paint(BOLD, m.name));
            last = m.name;
        }
        println!("  {}{}  {}", sig(f), pad(f.sig, w), f.desc);
    }
}

/// Builtins whose name, description or help-page group (`trig`, `encode`) mentions `topic`, or its stem.
// ponytail: crude suffix stemming ("sorting" → "sort"); real fuzzy matching if this misses too often.
fn search(topic: &str) -> Vec<(&'static Module, &'static Doc)> {
    let t = topic.to_lowercase();
    let stems: Vec<&str> = [Some(&t[..]), t.strip_suffix("ing"), t.strip_suffix("es"), t.strip_suffix('s')].into_iter().flatten().filter(|s| s.len() >= 2).collect();
    let hit = |m: &Module, f: &Doc| {
        m.groups.iter().any(|g| stems.contains(&g.0) && g.1.contains(&f.name)) || stems.iter().any(|s| f.name.contains(s) || f.desc.to_lowercase().contains(s))
    };
    docs().filter(|(m, f)| hit(m, f)).collect()
}

/// Up to three function, module, unit or topic names a typo or two from `topic`, closest first.
fn near(topic: &str) -> Vec<&'static str> {
    let units = crate::modules::units::TABLE.iter().flat_map(|u| u.0.split(' '));
    let names = docs().map(|(_, f)| f.name).chain(MODULES.iter().map(|m| m.name)).chain(["examples", "syntax"]).chain(units);
    let max = (topic.chars().count() / 3).max(1);
    let mut hits: Vec<_> = names.map(|n| (edits(topic, n), n)).filter(|h| h.0 <= max).collect();
    hits.sort();
    hits.dedup();
    hits.into_iter().take(3).map(|h| h.1).collect()
}

/// Levenshtein distance.
fn edits(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diag = row[0];
        row[0] = i + 1;
        for j in 0..b.len() {
            let next = (diag + usize::from(ca != b[j])).min(row[j] + 1).min(row[j + 1] + 1);
            diag = row[j + 1];
            row[j + 1] = next;
        }
    }
    row[b.len()]
}

/// A module's page: what it adds (types, operators, conversions...), then its functions by group.
fn page(m: &Module) {
    println!("{}: {}", paint(BOLD, m.name), m.about);
    sections(m.guide);
    if let Some(topic) = m.topic {
        topic(m.name);
    }
    if m.fns.is_empty() {
        return;
    }
    println!("\n{}", paint(BOLD, "functions"));
    let all = [("", m.fns.iter().map(|f| f.name).collect::<Vec<_>>())];
    let groups: Vec<_> = m.groups.iter().map(|g| (g.0, g.1.to_vec())).collect();
    let groups = if groups.is_empty() { &all[..] } else { &groups[..] };
    let gw = groups.iter().map(|g| g.0.chars().count()).max().unwrap_or(0);
    for (name, fns) in groups {
        let label = if name.is_empty() { String::new() } else { format!("{}{}  ", paint(DIM, name), pad(name, gw)) };
        println!("  {label}{}", fns.join(" "));
    }
    println!("{}", paint(DIM, &format!("help(name) for details, e.g. help({})", m.fns[0].name)));
}

/// Help sections as aligned `label  example  → result` rows; an empty example prints the label alone.
fn sections(guide: &[Section]) {
    let rows: Vec<_> = guide.iter().flat_map(|s| s.1).filter(|r| !r.1.is_empty()).collect();
    let lw = rows.iter().map(|r| r.0.chars().count()).max().unwrap_or(0);
    let ew = rows.iter().map(|r| r.1.chars().count()).max().unwrap_or(0);
    for (heading, rows) in guide {
        println!("\n{}", paint(BOLD, heading));
        for (label, ex) in *rows {
            if ex.is_empty() {
                println!("  {}", paint(DIM, label));
                continue;
            }
            let (plain, colored) = result(ex);
            let label = format!("{}{}", paint(DIM, label), pad(label, lw));
            if plain == *ex {
                println!("  {label}  {}", code(ex));
            } else {
                println!("  {label}  {}{}  {} {colored}", code(ex), pad(ex, ew), paint(DIM, "→"));
            }
        }
    }
}

/// Print each example with its live result, arrows aligned.
pub fn show(examples: Vec<String>) {
    let w = examples.iter().map(|e| e.chars().count()).max().unwrap_or(0);
    for ex in &examples {
        println!("  {}{}  {} {}", code(ex), pad(ex, w), paint(DIM, "→"), result(ex).1);
    }
}

/// Bold when color is on, for help text printed by modules' `topic` hooks.
pub fn heading(s: &str) -> String {
    paint(BOLD, s)
}

pub fn eval(src: &str) -> String {
    result(src).0
}

/// `src`'s result, plain and colored by type like a REPL result.
fn result(src: &str) -> (String, String) {
    match crate::run(&mut Interp::new(), src) {
        Ok(v) => {
            let s = format!("{v:?}");
            (s.clone(), paint(crate::tint(&v), &s))
        }
        Err(e) => {
            let s = format!("error: {}", e.msg);
            (s.clone(), s)
        }
    }
}

/// A signature with its function name in bold.
fn sig(f: &Doc) -> String {
    f.sig.strip_prefix(f.name).map_or(f.sig.to_string(), |rest| format!("{}{rest}", paint(BOLD, f.name)))
}

fn paint(color: &str, s: &str) -> String {
    if color.is_empty() || !crate::color_on() { s.to_string() } else { format!("{color}{s}{RESET}") }
}

/// Code, syntax-highlighted when color is on.
fn code(s: &str) -> String {
    if crate::color_on() { crate::highlight(s) } else { s.to_string() }
}

/// Spaces padding uncolored `s` to width `w`; `{:<w$}` would count escape codes.
fn pad(s: &str, w: usize) -> String {
    " ".repeat(w.saturating_sub(s.chars().count()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::units::DIMS;

    #[test]
    fn docs_examples_run() {
        let fns: Vec<_> = MODULES.iter().flat_map(|m| m.fns).collect();
        let guide = MODULES.iter().flat_map(|m| m.guide.iter().chain(m.examples)).chain(HIGHLIGHTS).chain(SYNTAX).flat_map(|s| s.1).map(|r| &r.1).filter(|e| !e.is_empty());
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
            assert_eq!(SHELVES.iter().filter(|s| s.1.contains(&m.name)).count(), 1, "{} on one shelf", m.name);
        }
        for (kind, _) in DIMS {
            assert!(help(Some(kind)));
        }
        assert!(help(Some("km")));
        assert!(help(Some("syntax")));
        assert!(help(Some("quantity")));
        assert!(!help(Some("nope")));
    }

    #[test]
    fn search_finds_by_description_group_and_stem() {
        let names = |t| search(t).iter().map(|h| h.1.name).collect::<Vec<_>>();
        assert!(names("sorting").contains(&"sort"));
        assert!(names("Uppercase").contains(&"upper"));
        assert!(names("trig").contains(&"atan2"));
        assert!(names("hash").contains(&"md5"));
        assert!(names("zzz").is_empty());
    }

    #[test]
    fn near_suggests_typos() {
        assert_eq!(edits("kitten", "sitting"), 3);
        assert_eq!(near("uper")[0], "upper");
        assert!(near("syntx").contains(&"syntax"));
        assert!(near("qqqqqq").is_empty());
    }
}
