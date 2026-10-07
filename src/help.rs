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
/// Titles of `help("advanced")` recipes, set apart from the code under them.
const RECIPE: &str = "\x1b[1;35m";

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
    if topic == "advanced" {
        advanced();
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
        ("4th Thursday of November", r#"date(2026, 11, 1).nth_weekday(4, "thu")"#),
        ("cities are time zones", r#"date("2026-12-25 18:30") to "Sao Paulo""#),
        ("units combine and convert", "100 km / 2 h to mph"),
        ("cups to grams", r#"1 cup * density("flour") to g"#),
        ("money", "$25/h to USD/workyr"),
        ("error bars", "5 ± 0.1 m * 2"),
        ("exact decimals", "0.1 + 0.2 == 0.3"),
        ("factoring big ints", "factors(2 ** 32 + 1)"),
        ("calculus", "integrate(sin, 0, pi)"),
        ("geography", r#"great_circle("Paris", "Tokyo")"#),
        ("fuzzy matching", r#""recieve".closest(["deceive", "receive", "recipe"])"#),
        ("subnets", r#"in_cidr("10.0.3.7", "10.0.0.0/22")"#),
        ("odds in plain words", "odds(2 ** -64)"),
        ("charts", "(0..20).map(|x| sin(x / 3)).sparkline"),
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

/// A worked example drawing on several modules: (what it does, the modules it uses, a script whose result is shown).
pub type Recipe = (&'static str, &'static str, &'static str);

/// `help("advanced")`: recipes, by theme.
#[rustfmt::skip]
pub const ADVANCED: &[(&str, &[Recipe])] = &[
    ("travel and sky", &[
        ("a flight from Seattle to Tokyo: time in the air and local arrival", "geo · units · time.zones", r#"leave = date("2026-12-20T13:00-08:00") to "Seattle"
flight = great_circle("Seattle", "Tokyo") / 900 km/h
{in_the_air: flight.parts, landing: leave + flight to "Tokyo"}"#),
        ("UTC hours when it's 9 to 5 in New York, London and Berlin at once", "time.zones · data.lists", r#"office = |t| 9 <= t.hour < 17
day = date("2026-10-12T00:00Z") to UTC
hours = (0..24).map(|h| day + h * 1 h)
hours.filter(|t| office(t to "New York") && office(t to "London") && office(t to "Berlin")).map(|t| t.format("%H:%M"))"#),
        ("hours of daylight on the 21st of each month, equator to arctic", "time.sky · geo · art.renderers · art.frames", r#"places = ["Singapore", "Cairo", "Paris", "Oslo", "Reykjavik"]
rows = places.map(|p| (1..=12).map(|m| day_length(p, date(2026, m, 21)) / 1 h))
places.map(|p| p.pad(10)).join("\n").beside(rows.heatmap)"#),
        ("where the sun sets within the next hour", "geo · time.sky · time", r#"cities().filter(|c| now <= sunset(c.name, today) < now + 1 h).map(|c| c.name).take(6)"#),
        ("the fastest a ping from New York could ever be: there and back through fiber at 2/3 c", "geo · units.constants", r#"["London", "Tokyo", "Sydney"].map(|p| 2 * great_circle("New York", p) / (2/3 * c) to ms)"#),
        ("dig straight down from Madrid: where do you come out?", "geo", r#"m = city("Madrid")
out = nearest(-m.lat, m.lon - 180)
{city: out.name, country: out.country, apart: great_circle("Madrid", out.name) to km}"#),
        ("the further north, the shorter December days: latitude vs. daylight across every city", "geo · time.sky · math.stats", r#"cs = cities()
corr(cs.map(|c| c.lat), cs.map(|c| day_length(c.name, date(2026, 12, 21)) / 1 h))"#),
        ("the day Oslo gains daylight fastest, and how fast", "time.sky · math.stats · time", r#"start = date(2026, 1, 1)
gains = (0..365).map(|i| day_length("Oslo", start + i * 1 d)).deltas
{day: start + (gains.find(gains.max) + 1) * 1 d, gained: gains.max to min, fastest_loss: gains.min to min}"#),
    ]),
    ("money and science", &[
        ("a $400k mortgage from January 2027: cost, crossover, interest by year", "units.money · time · data.lists · data.charts", r#"plan = amortize($400000, 6.5%/yr, 30 yr)
flip = plan.filter(|m| m.principal > m.interest).first
yearly = plan.chunks(12).map(|y| y.map(|m| m.interest).sum / $1)
{monthly: plan[0].payment, interest: plan.map(|m| m.interest).sum, crossover: date(2027, 1, 1) + (flip.n - 1) * 1 mo, by_year: yearly.sparkline}"#),
        ("a ski trip: who owes whom", "units.money · data.maps", r#"paid = {ana: $640 + $85.50, ben: 3 * $42, cy: $0, dee: $310}
{each: share(paid.values.sum, 4)[0], settle: settle(paid)}"#),
        ("weigh the Earth with a pendulum; errors carry through every step", "math.uncertainty · units.constants", r#"L = 1.000 ± 0.005 m
T = (20.07 ± 0.1 s) / 10    # timed 10 swings
g = 4 * pi ** 2 * L / T ** 2 to m/s^2
M = g * (6371 km) ** 2 / G to kg
{g: g, off_by: percent(g.value / g0 - 1, 2), earth: M}"#),
        ("a ball thrown at 20 m/s, 35° up: landing, peak and arc", "math.calculus · math.trig · data.charts", r#"v = 20; a = 35 deg; g = 9.81
y = |t| v * sin(a) * t - g / 2 * t ** 2
land = root(y, 0.1, 10)
peak = root(|t| deriv(y, t), 0, land)
{seconds: round(land, 2), meters: round(v * cos(a) * land, 1), peak: round(y(peak), 2), arc: (0..=30).map(|i| y(land * i / 30)).sparkline}"#),
        ("fractions closing in on pi: convergents of its continued fraction, and how far off each is", "math.numtheory · data.lists", r#"terms = pi.cfrac(5)
fold = |ts| ts.reverse.drop(1).reduce(ts.last, |acc, t| t + 1 / acc)
(1..=5).map(|n| fold(terms.take(n))).map(|c| [c to frac, sci(abs(c - pi), 1)])"#),
        ("a cookie recipe for 24, scaled to 60 and weighed instead of scooped", "units.kitchen · data.maps", r#"cups = {flour: 2.25 cup, sugar: 0.75 cup, butter: 1 cup, honey: 2 tbsp}
cups.entries.map(|[k, v]| [k, round(v * 60 / 24 * density(k) to g)]).from_entries"#),
        ("a Fermi estimate with honest error bars: piano tuners in Chicago", "math.uncertainty · units", r#"pianos = (2.7e6 ± 0.1e6) / (2.5 ± 0.5) * (5% ± 2%)    # people / household * share with a piano
per_tuner = (4 ± 1) / d * 250 d / yr
pianos / yr / per_tuner"#),
        ("three receipts, three unknown prices: coffee, bagel, juice", "math.linalg · units.money", r#"# rows: [coffees, bagels, juices] on each receipt
solve([[2, 1, 0], [1, 2, 1], [0, 1, 2]], [$9.50, $11.50, $8.50])"#),
    ]),
    ("text, data and dev", &[
        ("how far apart were the errors in a log, and what were they?", "text · time · math.stats · data.lists", r#"log = "08:59:58 INFO start
09:00:03 ERROR db timeout
09:00:41 INFO retry
09:02:15 ERROR db timeout
09:07:50 ERROR disk full"
errors = log.grep("ERROR")
times = errors.map(|l| date("2026-10-06 " + l.match(r"^\S+")))
{gaps: times.deltas.map(|d| d.parts), kinds: errors.map(|l| l.split("ERROR ")[1]).count_by(|k| k)}"#),
        ("crack a Caesar cipher: try every shift, keep the one with the most real words", "text.ciphers · text.layout · data", r#"secret = "wkh vhfuhw sdvvzrug lv vzruglvk"
common = ["the", "is", "and", "a", "of", "to", "secret", "password", "in"]
score = |s| s.words.filter(|w| w in common).len
(0..26).map(|k| secret.caesar(-k)).sort(score).last"#),
        ("a running log as CSV; h:m:s times become durations by a dot product", "fs · data.tables · math.linalg · units", r#"runs = "date,km,time\n2026-09-01,5,0:27:30\n2026-09-04,10,0:58:10\n2026-09-08,5,0:26:05\n2026-09-12,21.1,2:05:40".from_csv
dur = |t| dot(t.split(":").map(int), [1 h, 1 min, 1 s])
pretty(runs.map(|r| {date: r.date, km: r.km, per_km: dur(r.time) / r.km to min}).table.sort_by("per_km"))"#),
        ("carve a /22 into /24s and see which one each server lands in", "dev.net · data.tables", r#"servers = ["10.0.1.20", "10.0.3.7", "10.0.3.250", "10.0.0.9"]
pretty(subnets("10.0.0.0/22", 24).map(|b| {
  block: b, hosts: cidr(b).hosts, servers: servers.filter(|s| in_cidr(s, b)).join(" "),
}).table)"#),
        ("six hues, each as light as it can be while still readable on white", "dev.colors · data.lists", r#"readable = |h| (20..=70).map(|l| hsl(h, 75, l)).filter(|c| contrast(c, "white") >= 4.5).last
(0..6).map(|i| readable(i * 60)).map(|c| "{swatch(c)} {round(contrast(c, "white"), 2)}").join("\n")"#),
        ("when were this token and this ID minted, in Tokyo time?", "dev · time.zones", r#"token = jwt("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZGEiLCJpYXQiOjE1MTYyMzkwMjJ9.c2ln")
id = uuid_info("01890a5d-ac96-774b-bcce-b302099a8057")
[token.payload.iat, id.timestamp].map(|d| [d to "Tokyo", d.relative])"#),
        ("anagram groups: words that sort to the same letters", "text.layout · data.lists", r#"words = "listen silent enlist google tinsel inlets banana stone tones notes onset".words
words.group_by(|w| w.chars.sort.join).values.filter(|g| g.len > 1)"#),
        ("a spellchecker in one line: each word's closest match in a dictionary", "text.compare · text.layout", r#"dict = "the quick brown fox jumps over lazy dog".words
"teh qiuck bronw fox jumsp ovr the lazzy dgo".words.map(|w| w.closest(dict)).join(" ")"#),
        ("newest release per major version, by semver precedence rather than string order", "dev · data.lists · data.maps", r#"tags = ["1.9.3", "1.10.0", "2.0.0-rc.1", "2.0.0", "1.2.11", "2.1.0-beta"]
tags.group_by(|t| semver(t).major).map_values(|vs| vs.sort(semver).last)"#),
        ("how many files until two CRC-32s probably collide?", "fun.games · math.calculus", r#"half = root(|k| birthday_paradox(round(k), 2 ** 32) - 0.5, 1, 1e6)
{even_odds_at: round(half), at_10k_files: odds(birthday_paradox(10000, 2 ** 32))}"#),
        ("proof of work: the first nonce whose SHA-256 starts with 000", "text.hash · math", r#"n = 0
while !"zil:{n}".sha256.starts_with("000") { n += 1 }
{nonce: n, hash: "zil:{n}".sha256[..16], expected_tries: 16 ** 3}"#),
    ]),
    ("puzzles and play", &[
        ("prime dates: days of 2027 whose YYYYMMDD is prime", "math.numtheory · time · data.lists", r#"days = (0..365).map(|i| date(2027, 1, 1) + i * 1 d)
primes = days.filter(|d| is_prime(int(d.format("%Y%m%d"))))
{count: primes.len, first: primes.take(3).map(|d| d.format("%b %d")), by_month: primes.count_by(|d| d.format("%b"))}"#),
        ("numbers that are palindromes in both decimal and binary", "math.bases · data.lists", r#"pal = |n| n.digits == n.digits.reverse
(1..1000).filter(|n| pal(n) && pal(n to bin))"#),
        ("which points are in the Mandelbrot set? escape step, or stays", "math.complex · data.maps", r#"escapes = fn(c) { z = 0; for n in 1..=50 { z = z * z + c; if abs(z) > 2 { return n } }; return "stays" }
[0, -1, 1, 1i, 0.3 + 0.5i, -0.75 + 0.1i].map(|c| [str(c), escapes(c)]).from_entries"#),
        ("the exact chance of a poker full house", "math.numtheory · fun.games", r#"p = 13 * choose(4, 3) * 12 * choose(4, 2) / choose(52, 5)
[p to frac, odds(p)]"#),
    ]),
];

/// Recipes that touch files, the shell or the network: shown, never run (help would change things or hang offline).
#[rustfmt::skip]
pub const ADVANCED_UNRUN: (&str, &[Recipe]) = ("files, shell and network (shown, not run)", &[
    ("disk usage by file type, as a bar chart", "fs · data.lists · data.maps · data.charts", r#"files = glob("**/*").filter(|f| f.type == "file")
files.group_by(|f| f.name.ext).map_values(|fs| fs.map(|f| f.size).sum to KB).bars"#),
    ("which weekdays you commit on", "sys · time · data.lists · data.charts", r#"sh("git log --format=%ad --date=short").lines.count_by(|d| date(d).weekday).bars"#),
    ("PATH entries that point nowhere", "sys · fs", r#"sep = if env("OS") == "Windows_NT" { ";" } else { ":" }
env("PATH").split(sep).filter(|p| !exists(p))"#),
    ("files with the most TODOs", "fs · text · data.tables", r#"todos = glob("src/**/*.rs").map(|f| {file: f.name, todos: read_file(f.name).grep("TODO").len})
todos.filter(|r| r.todos > 0).table.sort_by_desc("todos")"#),
    ("sort Downloads into a folder per month", "fs · time", r#"for f in ls("Downloads").where(|f| f.type == "file") {
  dir = path_join("Downloads", f.modified.format("%Y-%m"))
  mkdir(dir)
  mv(path_join("Downloads", f.name), path_join(dir, f.name))
}"#),
    ("a weight log: zil weigh.zil 71.2 adds today's entry and charts them all", "sys · fs · time · data.charts", r#"if !exists("weight.csv") { write_file("weight.csv", "date,kg") }
append_file("weight.csv", "{today.format("%Y-%m-%d")},{args()[0]}")
read_file("weight.csv").from_csv.kg.sparkline"#),
    ("describe any CSV column: zil stats.zil data.csv price", "sys · fs · math.stats", r#"[file, col] = args()
read_file(file).from_csv.map(|r| r[col]).describe"#),
    ("GitHub stars, straight from the API", "sys · fs", r#"["rust-lang/rust", "zanbowie138/zil"].map(|r| fetch("https://api.github.com/repos/{r}").from_json.stargazers_count)"#),
    ("the weather in Oslo, in Fahrenheit", "sys · fs · units", r#"now_oslo = fetch("https://wttr.in/Oslo?format=j1").from_json.current_condition[0]
{temp: int(now_oslo.temp_C) * 1 C to F, feels_like: int(now_oslo.FeelsLikeC) * 1 C to F, sky: now_oslo.weatherDesc[0].value}"#),
    ("a 9-day trip at $150 a day, in local money (live rates)", "sys · units.money · geo", r#"allow_network_access()
{japan: $150 * 9 to "Japan", vietnam: $150 * 9 to "Vietnam", mexico: $150 * 9 to "Mexico"}"#),
    ("one phrase, three languages", "text.translation · sys", r#"allow_network_access()
["es", "ja", "de"].map(|l| "where is the train station?".translate(l))"#),
]);

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
        (r#"help("advanced")"#, "longer recipes that combine several modules"),
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

/// `help("advanced")`: each recipe's title and modules, its script, then its live result.
fn advanced() {
    let unrun = std::iter::once((ADVANCED_UNRUN, false));
    for ((theme, recipes), run) in ADVANCED.iter().map(|t| (*t, true)).chain(unrun) {
        out!("\n{}", paint(HEADING, theme));
        for (title, uses, src) in recipes {
            out!("\n  {}  {}", paint(RECIPE, title), paint(DIM, &format!("({uses})")));
            for line in src.lines() {
                out!("    {}", code(line));
            }
            if run {
                out!("    {} {}", paint(DIM, "→"), below(&result(src).1));
            }
        }
    }
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
    let names =
        docs().map(|(_, f)| f.name).chain(ALL.iter().flat_map(|(path, m)| [path.as_str(), m.name])).chain(["examples", "syntax", "advanced"]).chain(units);
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
                out!("  {label}  {}{}  {} {}", code(ex), pad(ex, ew), paint(DIM, "→"), below(&colored));
            }
        }
    }
}

/// Print each example with its live result, arrows aligned.
pub fn show(examples: Vec<String>) {
    let w = examples.iter().map(|e| e.chars().count()).max().unwrap_or(0);
    for ex in &examples {
        out!("  {}{}  {} {}", code(ex), pad(ex, w), paint(DIM, "→"), below(&result(ex).1));
    }
}

pub fn eval(src: &str) -> String {
    result(src).0
}

/// A multi-line result on its own indented lines, so art keeps its shape.
fn below(r: &str) -> String {
    if r.contains('\n') { r.lines().map(|l| format!("\n      {l}")).collect() } else { r.to_string() }
}

/// `src`'s result, plain and colored by type like a REPL result.
/// Text art (multi-line or colored strings) comes back bare, as the REPL prints it.
fn result(src: &str) -> (String, String) {
    match crate::run(&mut Interp::new(), src) {
        Ok(crate::value::Value::Str(s)) if s.contains(['\n', '\x1b']) => {
            let plain = crate::modules::art::strip_ansi(&s);
            let colored = if COLOR.get() { s.to_string() } else { plain.clone() };
            (plain, colored)
        }
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
        let recipes = ADVANCED.iter().flat_map(|t| t.1).map(|r| &r.2);
        for ex in fns.iter().flat_map(|f| f.examples.iter()).chain(guide).chain(recipes) {
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
        assert!(found("advanced"));
        for (_, _, src) in ADVANCED_UNRUN.1 {
            assert!(crate::parser::parse(src).is_ok(), "{src}");
        }
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
