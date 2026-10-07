//! `help`: every example shown is evaluated live in a fresh interpreter.

use crate::interp::Interp;
use crate::units::{DIMS, TABLE};

/// Name, signature, summary, examples, see also.
type Fn = (&'static str, &'static str, &'static str, &'static [&'static str], &'static [&'static str]);

#[rustfmt::skip]
const FNS: &[Fn] = &[
    // general
    ("print", "print(a, b, ...)", "print values separated by spaces", &[], &["str"]),
    ("type", "type(v)", "the type name of a value", &["type(5 km)", r#"type("hi")"#, "type([1])"], &["str", "int", "float"]),
    ("str", "str(v)", "convert to a string (full float precision)", &["str(1/3)", "str(5 km)"], &["int", "float"]),
    ("int", "int(v, base?)", "convert to an integer, parsing strings in an optional base", &["int(3.9)", r#""ff".int(16)"#, r#""0b101".int"#], &["float", "str"]),
    ("float", "float(v)", "convert to a float; drops a quantity's unit", &[r#"float("2.5")"#, "float(5 km)"], &["int", "str"]),
    ("bool", "bool(v)", "truthiness: false only for nil and false", &["bool(0)", "nil to bool"], &["str"]),
    ("hex", "hex(v, bits?)", "same as `v to hex` / `v to hex(bits)`; strings become hex bytes", &["hex(255)", "hex(-1, 16)", r#"hex("hi")"#], &["bin", "base", "int"]),
    ("bin", "bin(v, bits?)", "same as `v to bin` / `v to bin(bits)`", &["bin(10)", "bin(5, 8)"], &["hex", "oct"]),
    ("oct", "oct(v, bits?)", "same as `v to oct`", &["oct(8)"], &["hex", "bin"]),
    ("dec", "dec(v, bits?)", "same as `v to dec`; with bits, reads two's complement as unsigned", &["dec(0xff)", "dec(-1, 8)"], &["hex", "int"]),
    ("base", "base(v, b)", "same as `v to base(b)`, any base 2-36", &["base(35, 36)", "base(10, 3)"], &["hex", "digits"]),
    ("list", "list(v)", "convert to a list: characters, a copy, or [key, value] pairs", &[r#"list("abc")"#, r#""abc" to list"#, "{a: 1, b: 2}.list"], &["chars", "parse"]),
    ("parse", "parse(s)", "read a zil literal (number, string, list, map, quantity); never runs code", &[r#""[1, 2.5, 0xff]".parse"#, r#""5 km".parse to m"#], &["str", "nums"]),
    ("len", "len(v)", "length of a string, list or map", &[r#""héllo".len"#, "[1, 2, 3].len", "{a: 1}.len"], &[]),
    ("read_file", "read_file(path)", "file contents as a string", &[], &["write_file", "lines"]),
    ("write_file", "write_file(path, v)", "write v to a file as text", &[], &["read_file"]),
    ("help", "help(topic?)", "this help; topic is a function, unit or unit kind", &[], &[]),
    // strings
    ("upper", "upper(s)", "uppercase a string", &[r#""hello".upper"#, r#"upper("zil")"#], &["lower", "capitalize"]),
    ("lower", "lower(s)", "lowercase a string", &[r#""HeLLo".lower"#], &["upper", "capitalize"]),
    ("trim", "trim(s)", "strip leading and trailing whitespace", &[r#""  hi  ".trim"#], &["split"]),
    ("capitalize", "capitalize(s)", "uppercase the first character", &[r#""hello world".capitalize"#], &["upper"]),
    ("reverse", "reverse(v)", "reverse a string or list", &[r#""abc".reverse"#, "[1, 2, 3].reverse"], &["sort"]),
    ("split", "split(s, sep?)", "split on sep (string or regex); whitespace if omitted", &[r#""a b  c".split"#, r#""x,y;z".split(r"[,;]")"#], &["join", "lines", "chars"]),
    ("lines", "lines(s)", "split into lines", &[r#""a\nb".lines"#], &["split"]),
    ("chars", "chars(s)", "list of characters", &[r#""abc".chars"#], &["split"]),
    ("join", "join(list, sep?)", "join items into a string", &[r#"["a", "b"].join("-")"#, "[1, 2].join"], &["split"]),
    ("replace", "replace(s, pat, with)", "replace all matches; regex replacements can use $1", &[r#""a.b".replace(".", "-")"#, r#""a  b   c".replace(r"\s+", " ")"#], &["find_all", "match"]),
    ("contains", "contains(v, x)", "substring/regex in a string, item in a list, key in a map", &[r#""price: $12".contains(r"\$\d+")"#, "[1, 2].contains(2)"], &["find", "starts_with"]),
    ("starts_with", "starts_with(s, prefix)", "whether s starts with prefix", &[r#""abc".starts_with("a")"#], &["ends_with", "contains"]),
    ("ends_with", "ends_with(s, suffix)", "whether s ends with suffix", &[r#""file.zil".ends_with(".zil")"#], &["starts_with"]),
    ("find", "find(v, x)", "index of the first match, or nil", &[r#""hello".find("l")"#, "[5, 6].find(6)", r#""abc".find("z")"#], &["contains", "count"]),
    ("count", "count(v, x)", "number of matches in a string or list", &[r#""banana".count("a")"#, "[1, 2, 1].count(1)"], &["find"]),
    ("match", "match(s, regex)", "first match, its capture groups, or nil", &[r#""2026-10-06".match(r"(\d+)-(\d+)")"#, r#""id 42".match(r"\d+")"#], &["find_all", "replace"]),
    ("find_all", "find_all(s, pat)", "list of all matches", &[r#""a1b22c333".find_all(r"\d+")"#], &["match", "count"]),
    ("repeat", "repeat(s, n)", "repeat a string n times", &[r#""ab".repeat(3)"#], &[]),
    ("base64", "base64(s)", "base64-encode a string; same as `s to base64`", &[r#"base64("hi there")"#, r#""hi" to base64"#], &["encode", "decode"]),
    ("encode", "encode(s, fmt)", "encode as \"base64\", \"url\" or \"hex\"", &[r#""hi there".encode("base64")"#, r#""a b&c".encode("url")"#], &["decode"]),
    ("decode", "decode(s, fmt)", "decode \"base64\", \"url\" or \"hex\"", &[r#""aGk=".decode("base64")"#, r#""6869".decode("hex")"#], &["encode"]),
    ("sha256", "sha256(s)", "hex SHA-256 digest", &[r#""hello".sha256[..16]"#], &["md5"]),
    ("md5", "md5(s)", "hex MD5 digest", &[r#""hello".md5"#], &["sha256"]),
    ("ord", "ord(c)", "Unicode code point of a single character", &[r#""A".ord"#, r#""A".ord to hex"#], &["chr", "bytes"]),
    ("chr", "chr(n)", "character for a Unicode code point", &["97.chr", "(65..70).map(chr).join"], &["ord"]),
    ("nums", "nums(s)", "every number in a string", &[r#""x=3, y=-2.5; 1e3".nums"#], &["find_all", "parse"]),
    ("bytes", "bytes(s)", "list of the string's UTF-8 bytes", &[r#""hé".bytes"#], &["from_bytes", "ord"]),
    ("from_bytes", "from_bytes(list)", "string from a list of UTF-8 bytes", &["[104, 105].from_bytes"], &["bytes", "chr"]),
    // lists & maps
    ("range", "range(n) / range(a, b)", "integers in [0, n) or [a, b); same as a..b", &["range(4)", "range(2, 5)"], &["map"]),
    ("push", "push(list, v)", "append v in place and return the list", &["[1, 2].push(3)"], &[]),
    ("map", "map(list, f)", "apply f to every item", &[r"[1, 2, 3].map(\x -> x * 10)"], &["filter", "reduce"]),
    ("filter", "filter(list, f)", "keep items where f is truthy", &[r"(1..10).filter(\x -> x % 3 == 0)"], &["map", "reduce"]),
    ("reduce", "reduce(list, init, f)", "fold with f(acc, item)", &[r"[1, 2, 3].reduce(10, \acc, x -> acc + x)"], &["sum", "map"]),
    ("sum", "sum(list)", "add up a list; works with units", &["[1, 2, 3].sum", "[1 m, 50 cm].sum"], &["avg", "reduce"]),
    ("avg", "avg(list)", "mean of a list", &["[1, 2, 4].avg", "[2 h, 30 min].avg"], &["sum"]),
    ("min", "min(list) / min(a, b, ...)", "smallest value", &["min(3, 9, 4)", "[2 km, 1 mi].min"], &["max", "sort"]),
    ("max", "max(list) / max(a, b, ...)", "largest value", &["max(3, 9, 4)", r#"["b", "a"].max"#], &["min", "sort"]),
    ("sort", "sort(list, key?)", "sorted copy, optionally by key function", &["[3, 1, 2].sort", r#"["ccc", "a", "bb"].sort(\w -> w.len)"#], &["reverse", "unique"]),
    ("unique", "unique(list)", "drop duplicates, keeping first occurrences", &["[1, 2, 1, 3].unique"], &["sort", "count"]),
    ("first", "first(list)", "first item, or nil", &["[7, 8].first"], &["last"]),
    ("last", "last(list)", "last item, or nil", &["[7, 8].last"], &["first"]),
    ("keys", "keys(map)", "list of map keys", &["{a: 1, b: 2}.keys"], &["values"]),
    ("values", "values(map)", "list of map values", &["{a: 1, b: 2}.values"], &["keys"]),
    // math
    ("digits", "digits(n)", "list of digits in the number's own base", &["1234.digits", "0b1011.digits"], &["from_digits"]),
    ("from_digits", "from_digits(list, base?)", "build a number from digits, kept in that base", &["[1, 2, 3].from_digits", "[1, 0, 1, 1].from_digits(2)"], &["digits"]),
    ("sqrt", "sqrt(x)", "square root", &["sqrt(2)"], &[]),
    ("abs", "abs(x)", "absolute value; keeps units", &["abs(-3)", "abs(-2 km)"], &["round"]),
    ("round", "round(x, digits?)", "round to nearest; keeps units", &["round(pi, 2)", "round(2.5 km)"], &["floor", "ceil"]),
    ("floor", "floor(x)", "round down", &["floor(2.7)"], &["ceil", "round"]),
    ("ceil", "ceil(x)", "round up", &["ceil(2.1)"], &["floor", "round"]),
    ("ln", "ln(x)", "natural log", &["ln(e)"], &["log"]),
    ("log", "log(x, base?)", "log base 10, or another base", &["log(1000)", "log(8, 2)"], &["ln"]),
    ("sin", "sin(x)", "sine of radians or an angle unit", &["sin(30 deg)", "sin(pi / 2)"], &["cos", "tan", "asin"]),
    ("cos", "cos(x)", "cosine of radians or an angle unit", &["cos(60 deg)"], &["sin", "tan", "acos"]),
    ("tan", "tan(x)", "tangent of radians or an angle unit", &["tan(45 deg)"], &["sin", "cos", "atan"]),
    ("asin", "asin(x)", "arcsine, as an angle", &["asin(1) to deg"], &["sin"]),
    ("acos", "acos(x)", "arccosine, as an angle", &["acos(0) to deg"], &["cos"]),
    ("atan", "atan(x)", "arctangent, as an angle", &["atan(1) to deg"], &["tan"]),
    // dates
    ("date", "date(s) / date(s, fmt) / date(y, m, d, h?, min?, s?) / date(unix)", "parse ISO, US (12/25/2026), written (Dec 25 2026) or natural language dates", &[r#"date("2026-12-25")"#, r#"date("next friday at 5pm")"#, r#"date("3 days ago")"#, r#"date("25.12.2026", "%d.%m.%Y")"#, "date(2026, 12, 25, 18, 30)", "date(0)"], &["format", "with", "start_of"]),
    ("year", "year(d)", "year of a date", &["now.year"], &["month", "day"]),
    ("month", "month(d)", "month of a date (1-12)", &["now.month"], &["year", "day"]),
    ("day", "day(d)", "day of the month", &["now.day"], &["month", "weekday"]),
    ("hour", "hour(d)", "hour of a date", &["now.hour"], &["minute", "second"]),
    ("minute", "minute(d)", "minute of a date", &["now.minute"], &["hour", "second"]),
    ("second", "second(d)", "second of a date", &["now.second"], &["hour", "minute"]),
    ("weekday", "weekday(d)", "day name", &[r#"date("2026-12-25").weekday"#], &["day", "format"]),
    ("format", "format(d, fmt)", "format a date with strftime codes", &[r#"now.format("%B %d, %Y")"#, r#"now.format("%H:%M")"#], &["date"]),
    ("with", "with(d, fields)", "same date with fields replaced: year month day hour minute second", &["today.with({day: 1})", "now.with({hour: 9, minute: 0})"], &["start_of", "date"]),
    ("weekday_num", "weekday_num(d)", "weekday as a number, Monday = 1 ... Sunday = 7", &["today.weekday_num"], &["weekday"]),
    ("day_of_year", "day_of_year(d)", "day of the year, 1-366", &[r#"date("2026-12-31").day_of_year"#], &["iso_week"]),
    ("iso_week", "iso_week(d)", "ISO 8601 week number (weeks start Monday)", &[r#"date("2026-12-31").iso_week"#], &["day_of_year", "quarter"]),
    ("quarter", "quarter(d)", "quarter of the year, 1-4", &["today.quarter"], &["iso_week"]),
    ("leap_year", "leap_year(d or year)", "whether the year is a leap year", &["leap_year(2028)", "today.leap_year"], &["days_in_year"]),
    ("days_in_month", "days_in_month(d)", "number of days in the date's month", &[r#"date("2028-02-10").days_in_month"#], &["days_in_year"]),
    ("days_in_year", "days_in_year(d)", "365 or 366", &["today.days_in_year"], &["leap_year"]),
    ("is_weekend", "is_weekend(d)", "Saturday or Sunday", &[r#"date("2026-10-10").is_weekend"#], &["is_weekday", "add_workdays"]),
    ("is_weekday", "is_weekday(d)", "Monday through Friday", &["today.is_weekday"], &["is_weekend"]),
    ("is_today", "is_today(d)", "same calendar day as now", &["now.is_today", "tomorrow.is_today"], &["is_past"]),
    ("is_past", "is_past(d)", "before now", &["yesterday.is_past"], &["is_future", "is_today"]),
    ("is_future", "is_future(d)", "after now", &["tomorrow.is_future"], &["is_past"]),
    ("start_of", "start_of(d, period)", "start of the second/minute/hour/day/week/month/quarter/year", &[r#"now.start_of("week")"#, r#"now.start_of("quarter")"#], &["end_of", "with"]),
    ("end_of", "end_of(d, period)", "last moment of the period", &[r#"now.end_of("month")"#, r#"(today.end_of("year") - now).parts"#], &["start_of"]),
    ("next", "next(d, weekday)", "the next given weekday strictly after d", &[r#"today.next("friday")"#, r#"now.next("mon")"#], &["prev", "nth_weekday"]),
    ("prev", "prev(d, weekday)", "the last given weekday strictly before d", &[r#"today.prev("sunday")"#], &["next"]),
    ("nth_weekday", "nth_weekday(d, n, weekday)", "nth weekday of d's month; negative counts from the end", &[r#"date(2026, 11, 1).nth_weekday(4, "thu")"#, r#"date(2026, 5, 1).nth_weekday(-1, "mon")"#], &["next"]),
    ("add_workdays", "add_workdays(d, n)", "move n Monday-Friday days (no holidays); negative goes back", &["today.add_workdays(10)"], &["workdays", "is_weekend"]),
    ("workdays", "workdays(a, b)", "Monday-Friday days from a up to (not including) b", &[r#"workdays(today, date("2026-12-25"))"#], &["add_workdays"]),
    ("age", "age(d)", "whole years since d", &[r#"date("1990-06-15").age"#], &["diff"]),
    ("diff", "diff(a, b)", "calendar difference from a to b", &[r#"diff(date("2025-08-03"), date("2026-10-06 04:00"))"#], &["age", "relative", "parts"]),
    ("relative", "relative(d)", "\"in 3 days\", \"2 hours ago\"", &[r#"date("2026-12-25").relative"#, "(now - 3 h).relative"], &["diff", "parts"]),
    ("parts", "parts(duration)", "duration in up to three of d, h, min, s; or `to d h min` for chosen units", &[r#"(date("2026-12-25") - date("2026-10-06 14:24")).parts"#, "5000 s.parts"], &["relative", "diff"]),
    ("calendar", "calendar(d) / calendar(year, month)", "month grid, weeks starting Monday", &["calendar(2026, 12)"], &["date"]),
    // random
    ("rand", "rand() / rand(a, b)", "float in [0, 1), or a number from a to b inclusive", &["rand()", "rand(1, 6)"], &["choice", "shuffle"]),
    ("choice", "choice(list)", "random item", &[r#"["rock", "paper", "scissors"].choice"#], &["rand", "shuffle"]),
    ("shuffle", "shuffle(list)", "shuffled copy", &["(1..6).shuffle"], &["choice"]),
    ("uuid", "uuid()", "random v4 UUID", &["uuid()"], &["rand"]),
];

/// Overview: category, example, functions, first function of its section in FNS ("" = none).
#[rustfmt::skip]
const CATEGORIES: &[(&str, &str, &str, &str)] = &[
    ("strings", r#""a b".split.join("-")"#, "upper lower trim split replace match find_all nums base64 encode ord chr bytes ...", "upper"),
    ("lists", r"[3, 1, 2].sort.map(\x -> x * 2)", "map filter reduce sum avg min max sort unique ...", "range"),
    ("math", "round(sqrt(2), 3)", "digits sqrt abs round floor ceil ln log sin cos tan ...", "digits"),
    ("dates", r#"date("2026-12-25").weekday"#, "date with start_of next workdays diff relative calendar ..., now today", "date"),
    ("random", "rand(1, 6)", "rand choice shuffle uuid", "rand"),
    ("units", "5 km to mi", r#"help("length"), help("km"), help("units")"#, ""),
    ("general", "type(5 km)", "print str int float bool list hex bin base parse len read_file write_file", "print"),
];

pub fn help(topic: Option<&str>) -> Result<(), String> {
    let Some(topic) = topic else {
        println!("zil help: try help(upper), help(\"strings\"), help(\"km\"). In the REPL: help upper\n");
        let rows: Vec<_> = CATEGORIES.iter().map(|(c, ex, ..)| format!("{c:<8} {ex}")).collect();
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        for (row, (_, ex, names, _)) in rows.iter().zip(CATEGORIES) {
            println!("{row:<w$}  → {}", eval(ex));
            println!("         {names}");
        }
        return Ok(());
    };
    if let Some((name, sig, desc, examples, see)) = FNS.iter().find(|f| f.0 == topic) {
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
    if let Some(fns) = category(topic) {
        let w = fns.iter().map(|f| f.1.chars().count()).max().unwrap_or(0);
        for (_, sig, desc, ..) in fns {
            println!("{sig:<w$}  {desc}");
        }
        println!("examples: help(name), e.g. help({})", fns[0].0);
        return Ok(());
    }
    if topic == "units" {
        for (kind, _) in DIMS {
            println!("{kind:<12} {}", units_of(kind).join(" "));
        }
        println!("{:<12} 3-letter codes (USD EUR GBP ...), live rates fetched on first use", "currency");
        return Ok(());
    }
    if matches!(topic, "currency" | "money") {
        println!("currency: 3-letter codes like USD, EUR, GBP, JPY. Rates come from frankfurter.dev,");
        println!("are fetched on first use (with a prompt) and cached for a day.");
        println!("  100 USD to EUR");
        return Ok(());
    }
    if let Some(kind) = DIMS.iter().map(|d| d.0).find(|k| *k == topic) {
        let units = units_of(kind);
        println!("{kind} units: {}", units.join(" "));
        show(vec![format!("1 {} to {}", units[1], units[0]), format!("1 {} to {}", units[0], units[units.len() - 1])]);
        return Ok(());
    }
    if let Some((names, _, _, dim)) = TABLE.iter().find(|row| row.0.split(' ').any(|n| n == topic)) {
        let mut names = names.split(' ');
        let name = names.next().unwrap();
        let kind = DIMS.iter().find(|d| d.1 == *dim).unwrap().0;
        let aliases: Vec<_> = names.collect();
        let aka = if aliases.is_empty() { String::new() } else { format!("  (also {})", aliases.join(", ")) };
        println!("{name}: {kind}{aka}");
        let other = units_of(kind).into_iter().find(|u| *u != name).unwrap();
        show(vec![format!("1 {name} to {other}"), format!("1 {other} to {name}")]);
        println!("all {kind} units: help(\"{kind}\")");
        return Ok(());
    }
    Err(format!("no help for {topic:?}; try help() for an overview"))
}

/// The FNS rows from a category's first function up to the next category's.
fn category(topic: &str) -> Option<&'static [Fn]> {
    let start = |first: &str| FNS.iter().position(|f| f.0 == first);
    let i = start(CATEGORIES.iter().find(|c| c.0 == topic)?.3)?;
    let end = CATEGORIES.iter().filter_map(|c| start(c.3)).filter(|&s| s > i).min().unwrap_or(FNS.len());
    Some(&FNS[i..end])
}

fn units_of(kind: &str) -> Vec<&'static str> {
    let dim = DIMS.iter().find(|d| d.0 == kind).unwrap().1;
    TABLE.iter().filter(|row| row.3 == dim).map(|row| row.0.split(' ').next().unwrap()).collect()
}

/// Print each example with its live result, arrows aligned.
fn show(examples: Vec<String>) {
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

    #[test]
    fn every_builtin_documented_and_examples_run() {
        for name in crate::builtins::NAMES {
            assert!(FNS.iter().any(|f| f.0 == *name), "no help entry for {name}");
        }
        let examples = FNS.iter().flat_map(|f| f.3.iter()).chain(CATEGORIES.iter().map(|c| &c.1));
        for ex in examples {
            assert!(!eval(ex).starts_with("error:"), "{ex}: {}", eval(ex));
        }
        for f in FNS {
            for s in f.4 {
                assert!(FNS.iter().any(|g| g.0 == *s), "{}: see also {s} missing", f.0);
            }
        }
        for (kind, _) in DIMS {
            assert!(units_of(kind).len() >= 2, "{kind}");
            help(Some(kind)).unwrap();
        }
        let covered: usize = CATEGORIES.iter().filter_map(|c| category(c.0)).map(<[_]>::len).sum();
        assert_eq!(covered, FNS.len(), "every function belongs to exactly one category");
        for (name, ..) in CATEGORIES {
            help(Some(name)).unwrap();
        }
        assert!(help(Some("nope")).is_err());
    }
}
