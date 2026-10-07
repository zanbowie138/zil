//! `help`: rendered from each module's docs; every example shown is evaluated live in a fresh interpreter.

use crate::interp::Interp;
use crate::modules::{self, ALL, Doc, Module, Section, modules, units};
use crate::{DIM, RESET};
use std::cell::{Cell, RefCell};

/// Help colors: headings, module paths, function and unit names, parameter types.
pub const HEADING: &str = "\x1b[1;33m";
pub const MODPATH: &str = "\x1b[1;34m";
const SUBMODULE: &str = "\x1b[34m";
pub const NAME: &str = "\x1b[1;36m";
const TYPE: &str = "\x1b[32m";

thread_local! {
    /// What the page renderers below write to, read back by `capture`.
    static OUT: RefCell<String> = const { RefCell::new(String::new()) };
    static COLOR: Cell<bool> = const { Cell::new(false) };
    /// Set by `live`: help is going straight to a terminal, so color it if `.0` and wrap it to width `.1`.
    static TERM: Cell<Option<(bool, usize)>> = const { Cell::new(None) };
}

/// Runs `f` with every help page it renders shaped for a terminal: `Some((color, width))`.
pub fn live<T>(term: Option<(bool, usize)>, f: impl FnOnce() -> T) -> T {
    TERM.set(term);
    let r = f();
    TERM.set(None);
    r
}

/// `println!` into the help buffer.
macro_rules! out {
    ($($t:tt)*) => {
        $crate::help::push(format!($($t)*))
    };
}
pub(crate) use out;

pub fn push(line: String) {
    OUT.with_borrow_mut(|o| {
        o.push_str(&line);
        o.push('\n');
    });
}

/// Runs a page renderer, returning its text; `Err` holds the miss note if it returned false.
pub fn capture(color: bool, render: impl FnOnce() -> bool) -> Result<String, String> {
    COLOR.set(TERM.get().map_or(color, |t| t.0));
    OUT.take();
    let found = render();
    let mut text = OUT.take().trim_end().to_string();
    if let Some((_, width)) = TERM.get() {
        text = text.lines().map(|l| wrap(l, width)).collect::<Vec<_>>().join("\n");
    }
    if found { Ok(text) } else { Err(text) }
}

/// `line` fit to `width` by breaking its last column (after the last run of 2+ spaces) at spaces, continuing under
/// that column, or under a `→ result`'s result. Earlier columns (labels, code, signatures) are never broken; when the
/// last column starts with under 20 to work with, it moves to the next line, under the second column or the indent + 4.
/// Escape codes take no width.
fn wrap(line: &str, width: usize) -> String {
    let esc = regex::Regex::new("\x1b\\[[0-9;]*m").unwrap();
    let vis = |s: &str| esc.replace_all(s, "").chars().count();
    if vis(line) <= width {
        return line.to_string();
    }
    // Words with their start column and whether 2+ spaces come before them (a new column). Escape codes hold no
    // spaces, so splitting on spaces keeps each one inside a word.
    let (mut words, mut at, mut gap) = (Vec::new(), 0, 0);
    for (i, p) in line.split(' ').enumerate() {
        if i > 0 {
            (at, gap) = (at + 1, gap + 1);
        }
        if !p.is_empty() {
            words.push((p, at, gap >= 2 && !words.is_empty()));
            (at, gap) = (at + vis(p), 0);
        }
    }
    let lead = words.first().map_or(0, |w| w.1);
    let last = words.iter().rposition(|w| w.2).unwrap_or(0);
    let second = words.iter().find(|w| w.2).map_or(lead, |w| w.1);
    let roomy = |c: usize| width.saturating_sub(c) >= 20;
    let (mut out, mut at, mut hang) = (String::new(), 0, 0);
    for (i, &(p, start, _)) in words.iter().enumerate() {
        let spaces = if i == 0 { start } else { start - words[i - 1].1 - vis(words[i - 1].0) };
        let n = vis(p);
        let newline = if i == last && last > 0 && !roomy(start) {
            Some(if second < start && roomy(second) { second } else { lead + 4 })
        } else {
            (i > last && at + spaces + n > width).then_some(hang)
        };
        match newline {
            Some(c) => {
                out.push('\n');
                out.push_str(&" ".repeat(c));
                at = c;
            }
            None => {
                out.push_str(&" ".repeat(spaces));
                at += spaces;
            }
        }
        if i == last {
            hang = at + if esc.replace_all(p, "") == "→" { 2 } else { 0 };
        }
        out.push_str(p);
        at += n;
    }
    out
}

/// Help for `topic` (an overview if `None`), with ANSI colors if `color`.
pub fn help(topic: Option<&str>, color: bool) -> Result<String, String> {
    capture(color, || render(topic))
}

/// Writes help for `topic`; false if nothing matched.
fn render(topic: Option<&str>) -> bool {
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
    if let Some(m) = modules::find(topic) {
        page(m);
        return true;
    }
    if modules().filter_map(|m| m.topic).any(|f| f(topic)) || type_page(topic) {
        return true;
    }
    let hits = search(topic);
    if hits.is_empty() {
        let near: Vec<_> = near(topic).iter().map(|n| paint(NAME, n)).collect();
        let hint = if near.is_empty() { String::new() } else { format!("; did you mean {}?", near.join(", ")) };
        out!("no help for {topic:?}{hint}  help() for an overview");
        return false;
    }
    listing(hits);
    true
}

/// `help(value)`: every builtin whose first parameter takes that type; false for a type with none.
pub fn type_page(ty: &str) -> bool {
    // A typed first parameter that names `ty` (or `num`, for numbers); `any` would list everything.
    let takes = |f: &Doc| {
        let num = matches!(ty, "int" | "frac" | "float");
        modules::forms(f.sig).is_ok_and(|fs| fs.iter().any(|g| g.params.first().is_some_and(|p| p.0.iter().any(|t| *t == ty || (num && *t == "num")))))
    };
    let hits: Vec<_> = docs().filter(|(_, f)| takes(f)).collect();
    if hits.is_empty() {
        return false;
    }
    out!("functions taking a {} first, so x.f(...) works\n", paint(TYPE, ty));
    listing(hits);
    true
}

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
        ("destructuring, _ skips", "[a, [b, _]] = [1, [2, 3]]; a + b"),
        ("while", "n = 3; while n > 0 { n -= 1 }; n"),
        ("functions", "f = fn(x) { return x * 2 }; f(4)"),
        ("comparisons chain", "x = 5; 0 < x <= 10"),
        ("# comments run to the end of the line; a newline or ; ends a statement", ""),
    ]),
    ("more", &[("full reference, with operator precedence: the Syntax page from zil --docs", "")]),
];

/// `help()`: what zil is, its module tree, and how to dig deeper.
fn overview() {
    out!("{}: an expression calculator and scripting language with units, dates, exact fractions and big ints.", paint(MODPATH, "zil"));
    out!("\n{}", paint(HEADING, "modules"));
    let types = |m: &Module| m.guide.iter().find(|s| s.0 == "types").map(|s| s.1.iter().map(|r| r.0).collect::<Vec<_>>().join(" "));
    // Indented by depth: two spaces per dot in the path.
    let rows: Vec<_> = ALL.iter().map(|(path, m)| (format!("{}{}", "  ".repeat(path.matches('.').count()), m.name), *m)).collect();
    let w = rows.iter().map(|r| r.0.chars().count()).max().unwrap_or(0);
    for (name, m) in rows {
        let types = types(m).map_or(String::new(), |t| format!(" {}", paint(DIM, &format!("[{t}]"))));
        let label = name.replace(m.name, &paint(if m.children.is_empty() { SUBMODULE } else { MODPATH }, m.name));
        out!("  {label}{}  {}{types}", pad(&name, w), m.about);
    }
    out!("\n{}", paint(HEADING, "more help"));
    let rows = [
        (r#"help("examples")"#, "start here: a few dozen one-liners, module by module"),
        (r#"help("syntax")"#, "the language at a glance"),
        (r#"help("text")"#, "a module: its types, operators and every function"),
        (r#"help("math.trig")"#, "a submodule, by path or just help(\"trig\")"),
        ("help(upper)", "a function: signature, live examples, related functions"),
        ("help(today)", "a value: every function that takes its type"),
        (r#"help("km")"#, r#"a unit, or a kind of unit like help("length")"#),
        (r#"help("sorting")"#, "search function names and descriptions"),
        ("help upper", "REPL shorthand for help(upper)"),
        ("clear", "clear the screen (or Ctrl+L)"),
        ("exit", "leave the REPL (or quit, Ctrl+D)"),
    ];
    let w = rows.iter().map(|r| r.0.chars().count()).max().unwrap_or(0);
    for (ex, what) in rows {
        out!("  {}{}  {}", code(ex), pad(ex, w), paint(DIM, what));
    }
}

/// `help("examples")`: the highlights, then every module's showcase, aligned per module so one long line doesn't stretch them all.
fn examples() {
    sections(HIGHLIGHTS);
    for m in modules() {
        sections(m.examples);
    }
    out!("\n{}", paint(DIM, r#"help("time") etc. for a module's full function list"#));
}

/// Every builtin, with its module.
fn docs() -> impl Iterator<Item = (&'static Module, &'static Doc)> {
    modules().flat_map(|m| m.fns.iter().map(move |f| (m, f)))
}

/// A function's page: where it lives, signature, live examples, related functions.
fn function(m: &Module, f: &Doc) {
    let group = m.groups.iter().find(|g| g.1.contains(&f.name)).map_or(String::new(), |g| format!(" › {}", g.0));
    out!("{}{}", paint(MODPATH, modules::path(m)), paint(DIM, &group));
    out!("{}  {}", sig(f), f.desc);
    show(f.examples.iter().map(|e| e.to_string()).collect());
    for ex in f.shown {
        out!("  {}", code(ex));
    }
    if f.name == "help" {
        out!("  {}  {}", code("help upper"), paint(DIM, "(REPL shorthand)"));
    }
    if !f.see.is_empty() {
        out!("{} {}", paint(DIM, "see also:"), names(f.see, ", "));
    }
}

/// Search or type-page hits, grouped under their module.
fn listing(hits: Vec<(&Module, &Doc)>) {
    let w = hits.iter().map(|h| h.1.sig.chars().count()).max().unwrap_or(0);
    let mut last = "";
    for (m, f) in hits {
        if m.name != last {
            out!("{}", paint(MODPATH, modules::path(m)));
            last = m.name;
        }
        out!("  {}{}  {}", sig(f), pad(f.sig, w), f.desc);
    }
}

/// Builtins whose name, description, module (`trig`) or help-page group (`spread`) mentions `topic`, or its stem.
// ponytail: crude suffix stemming ("sorting" → "sort"); real fuzzy matching if this misses too often.
fn search(topic: &str) -> Vec<(&'static Module, &'static Doc)> {
    let t = topic.to_lowercase();
    let stems: Vec<&str> =
        [Some(&t[..]), t.strip_suffix("ing"), t.strip_suffix("es"), t.strip_suffix('s')].into_iter().flatten().filter(|s| s.len() >= 2).collect();
    let hit = |m: &Module, f: &Doc| {
        stems.contains(&m.name)
            || m.groups.iter().any(|g| stems.contains(&g.0) && g.1.contains(&f.name))
            || stems.iter().any(|s| f.name.contains(s) || f.desc.to_lowercase().contains(s))
    };
    docs().filter(|(m, f)| hit(m, f)).collect()
}

/// Up to three function, module, unit or topic names a typo or two from `topic`, closest first.
fn near(topic: &str) -> Vec<&'static str> {
    let units = crate::modules::units::TABLE.iter().flat_map(|u| u.0.split(' '));
    let names = docs().map(|(_, f)| f.name).chain(ALL.iter().flat_map(|(path, m)| [path.as_str(), m.name])).chain(["examples", "syntax"]).chain(units);
    crate::error::near(topic, names)
}

/// A module's page: what it adds (types, operators, conversions, units...), its submodules, then its functions by group.
fn page(m: &Module) {
    out!("{}: {}", paint(MODPATH, modules::path(m)), m.about);
    sections(m.guide);
    let kinds = units::kinds(m.units);
    if !kinds.is_empty() {
        out!("\n{}", paint(HEADING, "units"));
        let w = kinds.iter().map(|k| k.0.chars().count()).max().unwrap_or(0);
        for (kind, rows) in kinds {
            let units: Vec<_> = rows.iter().map(|r| r.0.split(' ').next().unwrap()).collect();
            out!("  {}{}  {}", paint(DIM, kind), pad(kind, w), names(&units, " "));
        }
    }
    if let Some(topic) = m.topic {
        topic(m.name);
    }
    if !m.children.is_empty() {
        out!("\n{}", paint(HEADING, "submodules"));
        let w = m.children.iter().map(|c| c.name.chars().count()).max().unwrap_or(0);
        for c in m.children {
            out!("  {}{}  {}", paint(SUBMODULE, c.name), pad(c.name, w), c.about);
        }
    }
    if m.fns.is_empty() {
        return;
    }
    out!("\n{}", paint(HEADING, "functions"));
    let all = [("", m.fns.iter().map(|f| f.name).collect::<Vec<_>>())];
    let groups: Vec<_> = m.groups.iter().map(|g| (g.0, g.1.to_vec())).collect();
    let groups = if groups.is_empty() { &all[..] } else { &groups[..] };
    let gw = groups.iter().map(|g| g.0.chars().count()).max().unwrap_or(0);
    for (name, fns) in groups {
        let label = if name.is_empty() { String::new() } else { format!("{}{}  ", paint(DIM, name), pad(name, gw)) };
        out!("  {label}{}", names(fns, " "));
    }
    out!("{}", paint(DIM, &format!("help(name) for details, e.g. help({})", m.fns[0].name)));
}

/// Help sections as aligned `label  example  → result` rows; an empty example prints the label alone.
fn sections(guide: &[Section]) {
    let rows: Vec<_> = guide.iter().flat_map(|s| s.1).filter(|r| !r.1.is_empty()).collect();
    let lw = rows.iter().map(|r| r.0.chars().count()).max().unwrap_or(0);
    let ew = rows.iter().map(|r| r.1.chars().count()).max().unwrap_or(0);
    for (heading, rows) in guide {
        out!("\n{}", paint(HEADING, heading));
        for (label, ex) in *rows {
            if ex.is_empty() {
                out!("  {}", paint(DIM, label));
                continue;
            }
            let (plain, colored) = result(ex);
            let label = format!("{}{}", paint(DIM, label), pad(label, lw));
            if plain == *ex {
                out!("  {label}  {}", code(ex));
            } else {
                out!("  {label}  {}{}  {} {colored}", code(ex), pad(ex, ew), paint(DIM, "→"));
            }
        }
    }
}

/// Print each example with its live result, arrows aligned.
pub fn show(examples: Vec<String>) {
    let w = examples.iter().map(|e| e.chars().count()).max().unwrap_or(0);
    for ex in &examples {
        out!("  {}{}  {} {}", code(ex), pad(ex, w), paint(DIM, "→"), result(ex).1);
    }
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

/// A signature with each form's function name and parameter types colored.
fn sig(f: &Doc) -> String {
    let types = regex::Regex::new(r": ([^,)]+)").unwrap();
    let form = |form: &str| match form.strip_prefix(f.name) {
        Some(rest) => format!("{}{}", paint(NAME, f.name), types.replace_all(rest, |c: &regex::Captures| format!(": {}", paint(TYPE, &c[1])))),
        None => form.to_string(),
    };
    f.sig.split(" / ").map(form).collect::<Vec<_>>().join(" / ")
}

/// Function or unit names, colored and joined.
pub fn names(names: &[&str], sep: &str) -> String {
    names.iter().map(|n| paint(NAME, n)).collect::<Vec<_>>().join(sep)
}

pub fn paint(color: &str, s: &str) -> String {
    if color.is_empty() || !COLOR.get() { s.to_string() } else { format!("{color}{s}{RESET}") }
}

/// Code, syntax-highlighted when color is on.
pub fn code(s: &str) -> String {
    if COLOR.get() { crate::highlight(s) } else { s.to_string() }
}

/// Spaces padding uncolored `s` to width `w`; `{:<w$}` would count escape codes.
pub fn pad(s: &str, w: usize) -> String {
    " ".repeat(w.saturating_sub(s.chars().count()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::units::DIMS;

    #[test]
    fn docs_examples_run() {
        let fns: Vec<_> = modules().flat_map(|m| m.fns).collect();
        let guide =
            modules().flat_map(|m| m.guide.iter().chain(m.examples)).chain(HIGHLIGHTS).chain(SYNTAX).flat_map(|s| s.1).map(|r| &r.1).filter(|e| !e.is_empty());
        for ex in fns.iter().flat_map(|f| f.examples.iter()).chain(guide) {
            assert!(!eval(ex).starts_with("error:"), "{ex}: {}", eval(ex));
        }
        let found = |t| help(Some(t), false).is_ok();
        for (path, m) in ALL.iter() {
            assert!(found(path));
            assert!(found(m.name));
        }
        for (kind, _) in DIMS {
            assert!(found(kind));
        }
        for row in units::rows() {
            let name = row.0.split(' ').next().unwrap();
            assert!(!help(Some(name), false).unwrap().contains("error:"), "help({name:?})");
        }
        assert!(help(Some("upper"), true).unwrap().contains(NAME));
        assert!(!help(Some("km"), false).unwrap().contains('\x1b'));
        assert!(found("syntax"));
        assert!(found("quantity"));
        assert!(!found("nope"));
        assert!(help(Some("upper"), false).unwrap().contains("upper(s: str)"));
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
    fn wrap_keeps_columns() {
        assert_eq!(wrap("  ab  one two three four five six seven", 30), "  ab  one two three four five\n      six seven");
        // Too little room past the column: it moves under the indent + 4, and code before it is never split.
        assert_eq!(wrap("  a_very_long_signature(x: str)  one two three", 40), "  a_very_long_signature(x: str)\n      one two three");
        assert_eq!(wrap("  f(x)  → aaa bbb ccc", 14), "  f(x)\n      → aaa\n        bbb\n        ccc");
        // Colors don't count toward the width.
        let red = |s: &str| format!("\x1b[31m{s}\x1b[0m");
        assert_eq!(
            wrap(&format!("  {}  one two three four five six seven", red("ab")), 30),
            format!("  {}  one two three four five\n      six seven", red("ab"))
        );
        assert_eq!(wrap("short", 30), "short");
    }

    #[test]
    fn near_suggests_typos() {
        assert_eq!(near("uper")[0], "upper");
        assert!(near("syntx").contains(&"syntax"));
        assert!(near("qqqqqq").is_empty());
    }
}
