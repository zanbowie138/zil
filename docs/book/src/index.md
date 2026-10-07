# zil

An expression calculator and scripting language with units, dates, exact fractions and big ints.

> Example results generated on 2026-10-06.
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
| [strings](strings.md) | case, splitting, search and regex, encodings, hashes |
| [lists](lists.md) | ranges, higher-order fns, aggregates, sorting |
| [math](math.md) | digits, rounding, logs, trigonometry (angles as units), formatting, number theory, bits |
| [dates](dates.md) | parsing, calendar math, relative text, month grids; weeks start Monday |
| [random](random.md) | numbers, picks, shuffles, UUIDs |
| [units](units.md) | numbers with units, combined and converted; currencies use live rates |
| [goofy_units](goofy_units.md) | bananas for scale, smoots, fortnights; every item also works as item_for_scale |
| [money](money.md) | currencies shown as money, pay, interest, loans, splitting bills |
| [color](color.md) | colors as "#rrggbb": RGB/HSL, mixing, lightening, WCAG contrast |
| [net](net.md) | IP addresses, CIDR blocks, subnets |
| [binary](binary.md) | hex dumps, entropy, checksums of strings and byte lists |
| [roman](roman.md) | Roman numerals, both ways |
| [ciphers](ciphers.md) | Caesar, ROT13, Atbash, Vigenère, Morse, Braille (for fun, not secrecy) |
| [fortune](fortune.md) | fortune cookies, a magic 8-ball, coin flips, excuses |
| [general](general.md) | values, printing, conversions, number bases, parsing, files |
