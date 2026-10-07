# math.formatting

format specs in strings, printf, and fixed/sci/percent/commas

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

### format specs

```zil
# "{x:.2f}"  fixed decimals
"{pi:.2f}"
# → "3.14"
# "{x:>8}"  width, align < > ^
"[{42:>8}]"
# → "[      42]"
# "{n:08x}"  zero pad; x X b o d
"{255:08x}"
# → "000000ff"
# "{n:#06x}"  # adds 0x 0b 0o
"{255:#06x}"
# → "0x00ff"
# "{x:,}"  commas; also + e %
"{1234567:,}"
# → "1,234,567"
# format("%5.2f", x)  printf
format("%5.2f|%-4d|", pi, 7)
# → " 3.14|7   |"
```

## Functions

| function | description |
|---|---|
| [`fixed(x, digits)`](#fixed) | string with exactly that many decimals; keeps units |
| [`sci(x, digits?)`](#sci) | string in scientific notation |
| [`percent(x, digits?)`](#percent) | string as a percentage |
| [`commas(x, digits?)`](#commas) | string with thousands separators |

### fixed

`fixed(x, digits)`: string with exactly that many decimals; keeps units

```zil
pi.fixed(2)
# → "3.14"
(5 km to mi).fixed(1)
# → "3.1 mi"
```

See also: [sci](../math/formatting.md#sci), [commas](../math/formatting.md#commas), [round](../math.md#round)

### sci

`sci(x, digits?)`: string in scientific notation

```zil
123456.sci
# → "1.23456e5"
123456.sci(2)
# → "1.23e5"
```

See also: [fixed](../math/formatting.md#fixed)

### percent

`percent(x, digits?)`: string as a percentage

```zil
0.256.percent
# → "25.6%"
(1/3).percent(1)
# → "33.3%"
```

See also: [fixed](../math/formatting.md#fixed)

### commas

`commas(x, digits?)`: string with thousands separators

```zil
1234567.commas
# → "1,234,567"
1234.5.commas(2)
# → "1,234.50"
```

See also: [fixed](../math/formatting.md#fixed)

## More examples

### formatting

```zil
# format specs
"{1234567.891:,.2f}"
# → "1,234,567.89"
# hex with a prefix
"{255:#06x}"
# → "0x00ff"
# 32-bit binary
"{0xdeadbeef:032b}"
# → "11011110101011011011111011101111"
```
