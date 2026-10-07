# zil

An expression calculator and scripting language with units, dates, exact fractions and big ints.

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### a taste

```zil
# day of the week
date("2026-12-25").weekday
# → "Friday"
# 4th Thursday of November
date(2026, 11, 1).nth_weekday(4, "thu")
# → 2026-11-26
# 3 business days later
date("2026-12-24").add_workdays(3)
# → 2026-12-29
# time zones
date("2026-12-25 18:30") to "Asia/Tokyo"
# → 2026-12-26 09:30:00 +09:00
# units combine and convert
100 km / 2 h to mph
# → 31.0686 mph
# split across units
1.8 m to ft in
# → "5 ft 10.8661 in"
# exact decimals
0.1 + 0.2 == 0.3
# → true
# big ints
2 ** 100
# → 1267650600228229401496703205376
# bits
0xf0 to bits
# → "0b1111_0000"
# percentages
80 + 15%
# → 92
# numbers out of text
"a1b22c333".nums.sum
# → 356
# syntax: x.f(y) is f(x, y)   |x| x * 2 is a lambda   xs |> sum pipes   # comments
```

## Modules

| module | about |
|---|---|
| [core](core.md) | values, printing, type conversions, parsing, help |
| [fs](fs.md) | files and directories, path pieces, JSON and CSV; relative paths start at the current directory |
| [sys](sys.md) | script arguments, environment variables, shell commands, HTTP GET, exit codes, network access |
| [text](text.md) | case, splitting, search and regex; layout, comparison, translation, encodings, hashes and ciphers below |
| [text.layout](text/layout.md) | pad, center, wrap, truncate, dedent; snake_case, camelCase, kebab-case, Title Case, slugs; words |
| [text.compare](text/compare.md) | edit distance, similarity, the closest of a list, line-by-line diffs |
| [text.translation](text/translation.md) | translate text between languages online; cached translations work offline |
| [text.encoding](text/encoding.md) | base64, base32, base58, URL and hex encodings, code points, UTF-8 bytes |
| [text.hash](text/hash.md) | SHA-1/256/512, MD5, HMAC and CRC-32 of strings |
| [text.ciphers](text/ciphers.md) | Caesar, ROT13, Atbash, Vigenère, Morse, Braille (for fun, not secrecy) |
| [data](data.md) | length, search, sorting and picking across strings, lists, maps and sets; lists, maps, sets and tables below |
| [data.lists](data/lists.md) | ranges, map/filter/reduce, building lists |
| [data.maps](data/maps.md) | keys, values, lookups, merging and transforming maps |
| [data.sets](data/sets.md) | unique values in insertion order: union, intersection, difference |
| [data.tables](data/tables.md) | rows under named columns: filter, pick columns, sort; list fns work on the rows |
| [data.charts](data/charts.md) | sparklines, bar charts, histograms and braille function plots, as text |
| [math](math.md) | rounding, roots, logs, constants; exact fractions and big ints |
| [math.stats](math/stats.md) | sums, averages, spread and extremes of lists; units welcome |
| [math.trig](math/trig.md) | sine, cosine and friends; angles as units |
| [math.numtheory](math/numtheory.md) | gcd, primes, factoring, factorials, modular powers; exact at any size |
| [math.bits](math/bits.md) | count, test, set, rotate and swap bits |
| [math.bases](math/bases.md) | hex, binary, octal and any base 2-36; digits of a number |
| [math.formatting](math/formatting.md) | format specs in strings, printf, fixed/sci/percent/commas, and human_bytes |
| [math.random](math/random.md) | numbers, picks, shuffles, UUIDs; passwords, passphrases, ULIDs and nano IDs from the OS's secure generator |
| [math.uncertainty](math/uncertainty.md) | measurements with an error, `5 ± 0.1 m`, propagated through arithmetic and math (first order, independent errors) |
| [math.complex](math/complex.md) | complex numbers, `2 + 3i`; a number touching `i` is imaginary |
| [math.linalg](math/linalg.md) | dot and cross products, norms, matrix multiply, transpose, determinant, inverse, linear systems; exact with fractions |
| [math.calculus](math/calculus.md) | numeric roots, derivatives and integrals of functions |
| [units](units.md) | numbers with units, combined and converted; currencies use live rates |
| [units.constants](units/constants.md) | physical constants (CODATA 2018) as quantities; `h` is still hours, so Planck's constant is `h_planck` |
| [units.money](units/money.md) | currencies shown as money, pay, interest, loans, splitting bills |
| [units.goofy](units/goofy.md) | bananas for scale, smoots, fortnights; every item also works as item_for_scale |
| [units.kitchen](units/kitchen.md) | cups to grams with ingredient densities, decibels, and CSS pixels (px) |
| [time](time.md) | dates: parsing, fields, date math, durations, relative text |
| [time.calendar_math](time/calendar_math.md) | periods, weekdays, business days, week numbers, month grids, cron schedules; weeks start Monday |
| [time.zones](time/zones.md) | the same moment in UTC, local time, any IANA zone or a city; world clocks |
| [time.sky](time/sky.md) | sunrise, sunset, day length and moon phase for a place and date; good to about a minute |
| [dev](dev.md) | developer tools: JWTs, UUIDs, URLs, semver, file permissions, IP addresses and subnets, raw bytes, colors, check digits and lookups |
| [dev.net](dev/net.md) | IP addresses, CIDR blocks, subnets |
| [dev.binary](dev/binary.md) | hex dumps and entropy of strings and byte lists |
| [dev.colors](dev/colors.md) | colors as "#rrggbb": RGB/HSL, mixing, lightening, WCAG contrast |
| [dev.codes](dev/codes.md) | check digits for cards, ISBNs and IBANs; HTTP status, port and MIME type lookups |
| [fun](fun.md) | fortune cookies, a magic 8-ball, coin flips, excuses, Roman numerals; placeholder text, mangled text, dice and zodiac, ASCII toys below |
| [fun.placeholder](fun/placeholder.md) | lorem ipsum placeholder text, random team and person names |
| [fun.mangle](fun/mangle.md) | mock case, l33t, uwu, Pig Latin, zalgo, upside-down text, NATO spelling, banners and emoji |
| [fun.games](fun/games.md) | dice notation, the birthday paradox, odds in plain words, Western and Chinese zodiac |
| [fun.toys](fun/toys.md) | ASCII Mandelbrot set, Conway's Game of Life, random mazes |
