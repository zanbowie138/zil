# zil syntax

zil is a calculator-style language for quick conversions and string work.
Everything is built in; there are no imports.

```
zil                  # REPL (history, multi-line, `_` = last result)
zil -e '5 km to mi'  # evaluate and print
zil script.zil       # run a file
echo hi | zil -e 'input.upper'
zil -e "help(upper)" # live examples; help() for an overview, help("km") for units
```

## Basics

```zil
# comments run to end of line
rate = 0.08          # assignment creates or updates a variable
price = 20
price * (1 + rate)   # 21.6
price += 5           # also -= *= /= //= %= **= &= |= ^= <<= >>=
```

Statements end at a newline. A line starting with `|>` or `.` continues the
previous one, as does a line ending in an operator. Newlines are free inside
`()`, `[]` and `{}`.

## Numbers

```zil
7 / 2        # 3.5   an exact fraction; 6 / 3 is the int 2
1 / 3 * 3    # 1     (fractions don't round: no 0.999999)
1/3 + 1/6 to frac    # 1/2   `to frac` shows it as a fraction, and so do results from it
0.75 to frac # 3/4   floats become the simplest fraction within 1e-12
2 ** 100     # 1267650600228229401496703205376
7 // 2       # 3     floor division
-7 % 3       # 2     remainder is never negative for a positive divisor
2 ** 10      # 1024  (`^` is xor; see Bitwise)
1_000_000    # underscores are ignored
1.5e3        # 1500
```

Integers have no size limit (results over about a million digits are refused).
Dividing ints gives an exact fraction (type `"frac"`) that displays like a float:
`1/3` shows `0.333333`, and `str(x)` gives full precision. Mixing a fraction with a
float gives a float. Floats display with 6 significant digits.

### Bases

```zil
0xff  0b1010  0o17   # literals in base 16, 2, 8
36#z  3#120          # any base 2-36: base#digits
x = 0b1010
x + 1                # 0b1011  numbers remember the base they were written in
255 to hex           # 0xff
0x1f to dec          # 31
35 to base(36)       # 36#z
0xff + 1             # 0x100  (the left operand's base wins, unless it is decimal)
"ff".int(16)         # 0xff   parse a string in a base
"0b101".int          # 0b101  prefixes are understood
-1 to hex(32)        # 0xffffffff  a bit width shows two's complement, zero-padded
5 to bin(8)          # 0b00000101
-1 to dec(8)         # 255    (works with hex, bin, oct, dec; must fit in the width)
"hi" to hex          # "6869" a string's UTF-8 bytes as hex
"hi" to base64       # "aGk="  same as .encode("base64")
```

`36#z` has no spaces; a comment right after a number needs one (`5 # note`, not `5#note`).

### Bitwise

```zil
0b1100 & 0b1010      # 8      and
0b1100 | 0b1010      # 14     or
0b1100 ^ 0b1010      # 6      xor
~0x0f to hex(8)      # 0xf0   not
1 << 4               # 16     (`<<` drops high bits like C; shift must be 0-63)
-16 >> 2             # -4     arithmetic shift
```

Integers only. Like Python, these bind looser than `+` but tighter than
comparisons: `x & 0xff == 0` is `(x & 0xff) == 0`.

### Characters and bytes

```zil
"A".ord              # 65     Unicode code point
97.chr               # "a"
"hé".bytes           # [104, 195, 169]  UTF-8 bytes
[104, 105].from_bytes  # "hi"
```

## Units

A unit written right after a number literal attaches to it. `to` converts.

```zil
5 km to mi             # 3.10686 mi
72 F to C              # 22.2222 C
2 m + 30 cm            # 2.3 m   (result uses the left unit)
60 km/h to m/s         # 16.6667 m/s
3 km / 20 min to kph   # 9 kph
2 m * 3 m              # 6 m^2
1.5 GB to MiB          # 1430.51 MiB
1 GiB / 100 Mbps to s  # 85.8993 s
100 USD to EUR         # live ECB rates
```

- `to` with several units splits the value: `5.5 ft to ft in` → `"5 ft 6 in"` (a string).
- `km/h`, `m^2` and `m/s^2` are one unit only when written **without spaces**.
  `60 km / h` divides by whatever `h` is. Inside a unit `^` is a power; elsewhere it is xor.
- `1/2 km` is `1 / (2 km)`; write `(1/2) * km` or `0.5 km`.
- With a variable, multiply by the unit: `d * km`. Units are also values when
  no variable has that name.
- A variable named like a unit (`m = 3`) hides the unit in expressions, but
  `5 m` and `to m` still mean metres.
- Adding mismatched units (`5 km + 1 kg`) is an error. When units cancel
  (`5 km / 1 km`), you get a plain number.

| Category    | Units                                                          |
|-------------|----------------------------------------------------------------|
| Length      | `m km cm mm um nm mi yd ft in nmi au ly`                       |
| Mass        | `kg g mg ug t lb oz st`                                        |
| Time        | `s ms us ns min h d wk mo yr`, paid time `workday workwk workmo workyr` |
| Temperature | `K C F`                                                        |
| Volume      | `L mL gal qt pt cup floz tbsp tsp` (and `m^3`, `cm^3`, ...)    |
| Area        | `ha acre` (and `m^2`, `ft^2`, ...)                             |
| Speed       | `kph mph kn` (and `m/s`, `km/h`, ...)                          |
| Data        | `bit B KB MB GB TB PB KiB MiB GiB TiB PiB Kb Mb Gb`            |
| Data rate   | `bps kbps Mbps Gbps`                                           |
| Energy      | `J kJ MJ cal kcal Wh kWh eV BTU`                               |
| Power       | `W kW MW hp`                                                   |
| Pressure    | `Pa kPa MPa bar atm psi mmHg`                                  |
| Force       | `N kN lbf`                                                     |
| Angle       | `rad deg turn`                                                 |
| Currency    | 3-letter codes: `USD EUR GBP JPY CAD ...`                      |

Long names work too: `meters`, `miles`, `hours`, `celsius`, `bytes`, ...
Currency rates come from frankfurter.dev (ECB data), are fetched only when two
currencies meet (`$25/h * 40 h` never goes online), and are cached for a day.
If you're offline, the last cached rates are used.

## Money

Currency amounts show as money: `1234.5 USD` is `$1,234.50`. `$`, `€` and `£`
written before a number mean USD, EUR and GBP: `$25/h`, `-€5`.

```zil
$25/h to USD/workyr            # $52,000.00/workyr  (workyr = 2080 h; yr is calendar time)
85000 USD/workyr to USD/h      # $40.87/h
$1200 / ($40/h) to workday     # 3.75 workday
salary($25/h)                  # {hour: $25.00, day: $200.00, week: ..., month: ..., year: $52,000.00}
salary(85000 USD/yr, {hours: 37.5, weeks: 48})
```

Finance functions tell their arguments apart by unit: `$1000` is a sum,
`$500/mo` a payment, `7%/yr` a rate (a `1%/mo` rate compounds monthly),
`30 yr` a time, and a string like `"monthly"` or `"continuous"` sets how
often interest compounds.

```zil
grow($10000, 7%/yr, 30 yr)                  # $76,122.55
grow($0, 7%/yr, 30 yr, $500/mo)             # $609,985.50
payment($400000, 6.5%/yr, 30 yr)            # $2,528.27/mo
payment($400000, 6.5%/yr, 30 yr) * 30 yr    # total paid
today + payoff($5000, 22%/yr, $200/mo)      # the debt-free date
amortize($400000, 6.5%/yr, 30 yr, {extra: $300/mo})   # [{n, payment, interest, principal, balance}, ...]
cagr($1000, $2500, 8 yr)    doubling(7%/yr)    apy(5%/yr, "daily")    real_rate(7%/yr, 3%/yr)
share($100, 3)                              # [$33.34, $33.33, $33.33]  always adds up
settle({ana: $120, ben: $0, cy: $30})       # ["ben pays ana $50.00", "cy pays ana $20.00"]
change($80, $100)    margin($60, $100)    markup($60, $100)
```

## Strings

```zil
name = "ann"
"hi {name}, 2+2={2 + 2}"   # interpolation; write \{ for a literal brace
"line\n\ttab \"quoted\""   # escapes: \n \t \r \0 \\ \" \{ \}

s = "hello"
s[0]         # "h"
s[1..4]      # "ell"   (end is exclusive)
s[-3..]      # "llo"   (negative counts from the end)
s[..2]       # "he"
s.len        # 5
"ab" * 3     # "ababab"
"a" + 1      # "a1"    (+ with a string concatenates)
```

### Regex

`r"..."` is a regex literal (no escapes processed). Functions that take a
pattern accept either a plain string (matched literally) or a regex.

```zil
"a1b22c333".find_all(r"\d+")              # ["1", "22", "333"]
"2026-10-06".match(r"(\d+)-(\d+)-(\d+)")  # ["2026", "10", "06"]  (groups)
"a  b   c".replace(r"\s+", " ")           # "a b c"
"x,y;z".split(r"[,;]")                    # ["x", "y", "z"]
"price: $12".contains(r"\$\d+")           # true
```

### String functions

| Function                     | Example → result                              |
|------------------------------|-----------------------------------------------|
| `upper` `lower`              | `"Hi".upper` → `"HI"`                         |
| `trim`                       | `" hi ".trim` → `"hi"`                        |
| `capitalize`                 | `"hello world".capitalize` → `"Hello world"`  |
| `reverse`                    | `"abc".reverse` → `"cba"` (lists too)         |
| `split(sep?)`                | `"a b".split` → `["a", "b"]` (no sep: whitespace) |
| `lines` `chars`              | `"a\nb".lines` → `["a", "b"]`                 |
| `join(sep?)`                 | `["a", "b"].join("-")` → `"a-b"`              |
| `replace(pat, with)`         | regex replacements can use `$1`               |
| `contains(pat)`              | also works on lists and map keys              |
| `starts_with` `ends_with`    | `"abc".starts_with("a")` → `true`             |
| `find(pat)`                  | index of first match, or `nil`                |
| `count(pat)`                 | number of matches                             |
| `match(regex)`               | first match, or a list of its groups, or `nil` |
| `find_all(pat)`              | list of all matches                           |
| `grep(pat)`                  | lines (or list items) containing pat          |
| `repeat(n)`                  | `"ab".repeat(2)` → `"abab"`                   |
| `encode(fmt)` `decode(fmt)`  | fmt is `"base64"`, `"url"` or `"hex"`         |
| `base64`                     | `"hi".base64` → `"aGk="`, same as `encode("base64")` |
| `sha256` `md5`               | hex digest                                    |
| `ord` `chr`                  | `"A".ord` → `65`, `65.chr` → `"A"`            |
| `bytes` `from_bytes`         | `"hi".bytes` → `[104, 105]` and back          |
| `nums`                       | `"x=3, y=-2.5".nums` → `[3, -2.5]`            |

## Calling functions

All of these are the same call:

```zil
upper("hi")
"hi".upper()
"hi".upper          # no parens needed when there are no other arguments
"hi" |> upper
```

`_` in a pipe stands for the piped value, wherever it goes:

```zil
3.14159 |> round(_, 2)       # 3.14
5 |> _ * 2                   # 10
```

(Outside a pipe, `_` in the REPL is the last result.)

`x.f(a)` and `x |> f(a)` both mean `f(x, a)`, so you can chain calls:

```zil
"the quick brown fox".split.map(|w| w.capitalize).join(" ")

input
  |> lines
  |> filter(|l| l.contains("ERROR"))
  |> len
```

## Lists and maps

```zil
xs = [3, 1, 2]
xs[0]            # 3
xs[-1]           # 2
xs[0..2]         # [3, 1]
xs.push(4)       # mutates in place
1..5             # [1, 2, 3, 4]
1..=5            # [1, 2, 3, 4, 5]
(0..=20).step(5) # [0, 5, 10, 15, 20]  every 5th item
2 in xs          # true  (same as xs.contains(2); also strings and map keys)

u = {name: "ann", age: 30}
u.name           # "ann"
u["name"]        # same
u.email          # nil (missing keys are nil)
u.city = "Oslo"
```

| Function                  | Notes                                       |
|---------------------------|---------------------------------------------|
| `len` `first` `last` `step(n)` |                                             |
| `map(f)` `filter(f)`      | `[1, 2].map(\|x\| x * 10)` → `[10, 20]`  |
| `reduce(init, f)`         | `xs.reduce(0, \|acc, x\| acc + x)`       |
| `sum` `avg` `min` `max`   | work with units: `[1 m, 50 cm].sum` → `1.5 m` |
| `sort` `sort(f)`          | `words.sort(\|w\| w.len)` sorts by key   |
| `unique` `reverse` `shuffle` |                                          |
| `contains(v)` `find(v)` `count(v)` |                                    |
| `any(f)` `all(f)`        | `f` is optional: `[1, nil].any` → `true` |
| `take(n)` `drop(n)`       | also on strings                             |
| `sort_desc` `flatten` `zip(ys)` `enumerate` |                           |
| `group_by(f)` `count_by(f)` | map from each key to its items / count   |
| `chunks(n)` `windows(n)`  | `[1, 2, 3].windows(2)` → `[[1, 2], [2, 3]]` |
| `keys` `values`           | for maps                                    |
| `range(n)` `range(a, b)`  | same as `0..n` / `a..b`                     |

Lists and maps are shared: assigning one to another variable does not copy it.

## Math

`sqrt abs round floor ceil ln log sin cos tan asin acos atan`, plus `pi` and `e`.

```zil
round(pi, 2)       # 3.14
round(2.5 km)      # 3 km   (abs/round/floor/ceil keep units)
sin(30 deg)        # 0.5    (trig takes radians or an angle unit)
asin(1) to deg     # 90 deg
log(1000)          # 3      (log(x, base) for other bases)
min(3, 9, 4)       # 3
```

## Dates

Weeks start on Monday (ISO 8601). A date at local midnight displays as just the date.

### Creating

```zil
now                          # 2026-10-06 14:03:12 -05:00 (local time)
today  tomorrow  yesterday   # local midnight: 2026-10-06
date(2026, 12, 25)           # from parts; also date(y, m, d, h, min, s)
date(1791313380)             # from a unix timestamp
date("2026-12-25")           # ISO; "2026-12-25 14:00", "2026-12-25T10:00Z" also work
date("12/25/2026")           # US month/day (day/month only when the first number is > 12)
date("Dec 25 2026 5pm")      # written-out dates, RFC 2822 too
date("25.12.2026", "%d.%m.%Y")   # explicit strftime format
today.with({day: 1})         # replace fields: year month day hour minute second
```

Natural language, with an optional time (`at 5pm`, `17:30`, `noon`, `midnight`):

```zil
date("tomorrow at 5pm")      date("next friday")     date("last monday")
date("3 days ago")           date("in 2 weeks")      date("an hour from now")
date("next month")           date("end of month")    date("start of next week")
date("first day of next year")                       date("last day of the month")
```

### Arithmetic

```zil
date("2026-12-25") - now     # 79.4 d
now + 90 min
date("2026-01-31") + 1 mo    # 2026-02-28 (whole months/years follow the calendar)
now.start_of("week")         # also second minute hour day month quarter year
now.end_of("month")          # 2026-10-31 23:59:59
today.next("friday")         # strictly after; prev("mon") for before
date(2026, 11, 1).nth_weekday(4, "thu")   # Thanksgiving; -1 = last in the month
today.add_workdays(10)       # Monday-Friday only, no holidays
workdays(today, date("2026-12-25"))       # weekdays in [a, b)
```

### Questions

```zil
now.year  now.month  now.day  now.hour  now.minute  now.second
now.weekday  now.weekday_num # "Tuesday", 2 (Monday = 1)
now.day_of_year  now.iso_week  now.quarter
now.leap_year  leap_year(2028)  now.days_in_month  now.days_in_year
now.is_weekend  now.is_weekday  now.is_today  now.is_past  now.is_future
date("1990-06-15").age       # 36 (whole years)
```

### Display

```zil
diff(date("2025-08-03"), now)    # "1 yr 2 mo 3 d 14 h 3 min 12 s"
date("2026-12-25").relative      # "in 3 months"; "2 hours ago", "just now"
(date("2026-12-25") - now).parts # "79 d 9 h 36 min" (picks up to three of d h min s)
(date("2026-12-25") - now) to d h    # "79 d 9.6 h" (your choice of units; works for any unit: 5.5 ft to ft in)
now to "Asia/Tokyo"          # time zone conversion; also `to UTC`, `to local`
now to unix                  # 1791313380
now.format("%B %d, %Y")      # "October 06, 2026" (strftime codes)
calendar(2026, 12)           # month grid, like `cal`; also calendar(today)
```

## Random

```zil
rand()            # float in [0, 1)
rand(1, 6)        # int from 1 to 6 inclusive (floats give a float)
[1, 2, 3].choice
[1, 2, 3].shuffle
uuid()
```

## Control flow and functions

Everything is an expression; a block `{ ... }` evaluates to its last line.

```zil
size = if n > 100 { "big" } else if n > 0 { "small" } else { "none" }

for x in 1..4 { print(x) }
for key in {a: 1, b: 2} { print(key) }  # maps iterate keys
for ch in "abc" { print(ch) }           # strings iterate characters
for [i, x] in ["a", "b"].enumerate { print(i, x) }  # unpack each item
while n > 0 { n -= 1 }
for x in 1..100 {
  if x % 2 == 0 { continue }            # next iteration
  if x > 9 { break }                    # leave the loop
}

double = |x| x * 2                      # short lambda
area = |w, h| w * h
dist = |[x, y]| hypot(x, y)            # params can unpack lists
[a, [b, _]] = [1, [2, 3]]               # so can assignment; _ skips
[a, b] = [b, a]                         # swap
clamp = fn(x, lo, hi) {                 # block function
  if x < lo { return lo }
  min(x, hi)
}
```

A `{` at the start of an expression is a map; blocks only follow
`if`, `else`, `while`, `for` and `fn`.

## Other builtins

| Function                     | Notes                                     |
|------------------------------|-------------------------------------------|
| `print(a, b, ...)`           | prints values separated by spaces         |
| `type(v)`                    | `"int"`, `"frac"`, `"float"`, `"quantity"`, `"str"`, `"regex"`, `"date"`, `"list"`, `"map"`, `"fn"`, `"bool"`, `"nil"` |
| `str` `int` `float` `frac` `bool` | conversions; `int(s, base)` parses a base; `bool` is false only for `nil`/`false` |
| `list`                       | `list("ab")` → `["a", "b"]`; maps give `[key, value]` pairs |
| `v to str` / `int` / `float` / `list` / `bool` / `base64` | same as calling that function: `"0xff" to int` → `0xff` |
| `hex` `bin` `oct` `dec` `(v, bits?)`, `base(v, b)` | same as `v to hex(bits)` etc.: `bin(5, 8)` → `0b00000101` |
| `parse`                      | read a literal back: `"[1, 5 km]".parse`; never runs code |
| `digits` `from_digits(base?)` | `0b110.digits` → `[1, 1, 0]` and back     |
| `input`                      | all of stdin as a string (read once)      |
| `read_file(path)`            | file contents as a string                 |
| `write_file(path, v)`        | writes `v` as text                        |

## Operator precedence

Lowest to highest:

| Operator               |                                   |
|------------------------|-----------------------------------|
| `=` `+=` `-=` ...      | assignment                        |
| `to`                   | conversion                        |
| `\|>`                  | pipe                              |
| `\|\|`                 |                                   |
| `&&`                   |                                   |
| `==` `!=`              | `1 m == 100 cm` is true           |
| `<` `<=` `>` `>=` `in` | chain: `0 < x <= 10`              |
| `..` `..=`             | range                             |
| `\|`                   | bitwise or                        |
| `^`                    | bitwise xor                       |
| `&`                    | bitwise and                       |
| `<<` `>>`              |                                   |
| `+` `-`                |                                   |
| `*` `/` `//` `%`       |                                   |
| `-x` `!x` `~x`         |                                   |
| `**`                   | right-associative; `-2**2` is `-4` |
| `f()` `x[i]` `x.k`     |                                   |

`nil` and `false` are falsy; everything else is truthy.
