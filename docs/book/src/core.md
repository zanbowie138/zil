# core

values, printing, type conversions, parsing, help

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### types

```zil
# nil
nil
# bool
true
# int
0xff
# float
1.5e3
# → 1500
# frac
7/2 to frac
# → 7/2
# fn
|x| x * 2
# → <fn>
```

### names

```zil
# input  (stdin as a string): input.lines.len
```

### conversions

```zil
# to str int float frac bool list
"42" to int
# → 42
```

### pretty

```zil
# long: thousands separators, decimals kept
pretty(1234567.891)
# → "1,234,567.891"
# short: K M B T, 3 significant figures
pretty(1234567, "short")
# → "1.23M"
# pages for quantity, uncertain, date, list, map, set and table show theirs
```

## Functions

| function | description |
|---|---|
| [`print(a?: any, ...)`](#print) | print values separated by spaces |
| [`type(v: any)`](#type) | the type name of a value |
| [`str(v: any)`](#str) | convert to a string (full float precision) |
| [`int(v: num\|str) / int(s: str, base: int)`](#int) | convert to an integer, parsing strings in an optional base |
| [`float(v: num\|quantity\|str)`](#float) | convert to a float; drops a quantity's unit |
| [`frac(v: num)`](#frac) | show as a fraction; floats become the simplest fraction within 1e-12 |
| [`bool(v: any)`](#bool) | truthiness: false only for nil and false |
| [`list(v: str\|list\|map\|set\|table)`](#list) | convert to a list: characters, a copy, [key, value] pairs, a set's items, or a table's rows as maps |
| [`pretty(v: any) / pretty(v: any, style: str)`](#pretty) | human-readable text; style is "long" (default) or "short"; each type's module page has its rules |
| [`parse(s: str)`](#parse) | read a zil literal (number, string, list, map, quantity); never runs code |
| [`help(topic?: any)`](#help) | this help, as text; topic is a function, module ("trig" or "math.trig"), unit, or any value to list functions for its type |

### print

`print(a?: any, ...)`: print values separated by spaces

```zil
print("total:", 5 km)
```

See also: [str](core.md#str)

### type

`type(v: any)`: the type name of a value

```zil
type(5 km)
# → "quantity"
type("hi")
# → "str"
type([1])
# → "list"
```

See also: [str](core.md#str), [int](core.md#int), [float](core.md#float)

### str

`str(v: any)`: convert to a string (full float precision)

```zil
str(1/3)
# → "0.3333333333333333"
str(5 km)
# → "5 km"
```

See also: [int](core.md#int), [float](core.md#float)

### int

`int(v: num|str) / int(s: str, base: int)`: convert to an integer, parsing strings in an optional base

```zil
int(3.9)
# → 3
"ff".int(16)
# → 0xff
"0b101".int
# → 0b101
```

See also: [float](core.md#float), [str](core.md#str)

### float

`float(v: num|quantity|str)`: convert to a float; drops a quantity's unit

```zil
float("2.5")
# → 2.5
float(5 km)
# → 5
```

See also: [int](core.md#int), [str](core.md#str)

### frac

`frac(v: num)`: show as a fraction; floats become the simplest fraction within 1e-12

```zil
7/2 to frac
# → 7/2
1/3 + 1/6 to frac
# → 1/2
0.75.frac
# → 3/4
```

See also: [float](core.md#float)

### bool

`bool(v: any)`: truthiness: false only for nil and false

```zil
bool(0)
# → true
nil to bool
# → false
```

See also: [str](core.md#str)

### list

`list(v: str|list|map|set|table)`: convert to a list: characters, a copy, [key, value] pairs, a set's items, or a table's rows as maps

```zil
list("abc")
# → ["a", "b", "c"]
"abc" to list
# → ["a", "b", "c"]
{a: 1, b: 2}.list
# → [["a", 1], ["b", 2]]
```

See also: [chars](text.md#chars), [parse](core.md#parse)

### pretty

`pretty(v: any) / pretty(v: any, style: str)`: human-readable text; style is "long" (default) or "short"; each type's module page has its rules

```zil
pretty(1234567)
# → "1,234,567"
pretty(1234567, "short")
# → "1.23M"
pretty(5000 s)
# → "1 h 23 min 20 s"
pretty(date("2026-12-25 18:30"))
# → "Friday, December 25, 2026 at 6:30 PM"
pretty([1500000 B, 2 ** 20], "short")
# → "[1.5 MB, 1.05M]"
```

See also: [str](core.md#str), [commas](math/formatting.md#commas), [simplify](units.md#simplify), [parts](time.md#parts), [format](time.md#format)

### parse

`parse(s: str)`: read a zil literal (number, string, list, map, quantity); never runs code

```zil
"[1, 2.5, 0xff]".parse
# → [1, 2.5, 0xff]
"5 km".parse to m
# → 5000 m
```

See also: [str](core.md#str), [nums](text.md#nums)

### help

`help(topic?: any)`: this help, as text; topic is a function, module ("trig" or "math.trig"), unit, or any value to list functions for its type

```zil
help(upper)
help("math.trig")
help(today)
help("text") |> grep("case")
```

## More examples

### core

```zil
# type of anything
type(5 km)
# → "quantity"
# parse values from text
parse("[1, 2, 5 km]")
# → [1, 2, 5 km]
```
