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
| [core](core.md) | values, printing, type conversions, parsing, files, help |
| [text](text.md) | case, splitting, search and regex; encodings, hashes and ciphers below |
| [text.encoding](text/encoding.md) | base64, URL and hex encodings, code points, UTF-8 bytes |
| [text.hash](text/hash.md) | SHA-256, MD5 and CRC-32 of strings |
| [text.ciphers](text/ciphers.md) | Caesar, ROT13, Atbash, Vigenère, Morse, Braille (for fun, not secrecy) |
| [data](data.md) | length, search, sorting and picking across strings, lists, maps and sets; lists, maps and sets below |
| [data.lists](data/lists.md) | ranges, map/filter/reduce, building lists |
| [data.maps](data/maps.md) | keys, values, lookups, merging and transforming maps |
| [data.sets](data/sets.md) | unique values in insertion order: union, intersection, difference |
| [math](math.md) | rounding, roots, logs, constants; exact fractions and big ints |
| [math.stats](math/stats.md) | sums, averages, spread and extremes of lists; units welcome |
| [math.trig](math/trig.md) | sine, cosine and friends; angles as units |
| [math.numtheory](math/numtheory.md) | gcd, primes, factoring, factorials, modular powers; exact at any size |
| [math.bits](math/bits.md) | count, test, set, rotate and swap bits |
| [math.bases](math/bases.md) | hex, binary, octal and any base 2-36; digits of a number |
| [math.formatting](math/formatting.md) | format specs in strings, printf, and fixed/sci/percent/commas |
| [math.random](math/random.md) | numbers, picks, shuffles, UUIDs |
| [units](units.md) | numbers with units, combined and converted; currencies use live rates |
| [units.constants](units/constants.md) | physical constants (CODATA 2018) as quantities; `h` is still hours, so Planck's constant is `h_planck` |
| [units.money](units/money.md) | currencies shown as money, pay, interest, loans, splitting bills |
| [units.goofy](units/goofy.md) | bananas for scale, smoots, fortnights; every item also works as item_for_scale |
| [time](time.md) | dates: parsing, fields, date math, durations, relative text |
| [time.calendar_math](time/calendar_math.md) | periods, weekdays, business days, week numbers, month grids; weeks start Monday |
| [time.zones](time/zones.md) | the same moment in UTC, local time or any IANA zone |
| [dev](dev.md) | developer tools: IP addresses and subnets, raw bytes, colors |
| [dev.net](dev/net.md) | IP addresses, CIDR blocks, subnets |
| [dev.binary](dev/binary.md) | hex dumps and entropy of strings and byte lists |
| [dev.colors](dev/colors.md) | colors as "#rrggbb": RGB/HSL, mixing, lightening, WCAG contrast |
| [fun](fun.md) | fortune cookies, a magic 8-ball, coin flips, excuses, Roman numerals; lorem ipsum placeholder text below |
| [fun.placeholder](fun/placeholder.md) | lorem ipsum placeholder text, random team and person names |
