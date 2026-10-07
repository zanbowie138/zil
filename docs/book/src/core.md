# core

values, printing, type conversions, parsing, files, help

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
| [`list(v: str\|list\|map)`](#list) | convert to a list: characters, a copy, or [key, value] pairs |
| [`parse(s: str)`](#parse) | read a zil literal (number, string, list, map, quantity); never runs code |
| [`read_file(path: str)`](#read_file) | file contents as a string |
| [`write_file(path: str, v: any)`](#write_file) | write v to a file as text |
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

`list(v: str|list|map)`: convert to a list: characters, a copy, or [key, value] pairs

```zil
list("abc")
# → ["a", "b", "c"]
"abc" to list
# → ["a", "b", "c"]
{a: 1, b: 2}.list
# → [["a", 1], ["b", 2]]
```

See also: [chars](text.md#chars), [parse](core.md#parse)

### parse

`parse(s: str)`: read a zil literal (number, string, list, map, quantity); never runs code

```zil
"[1, 2.5, 0xff]".parse
# → [1, 2.5, 0xff]
"5 km".parse to m
# → 5000 m
```

See also: [str](core.md#str), [nums](text.md#nums)

### read_file

`read_file(path: str)`: file contents as a string

```zil
read_file("notes.txt").lines.len
```

See also: [write_file](core.md#write_file), [lines](text.md#lines)

### write_file

`write_file(path: str, v: any)`: write v to a file as text

```zil
write_file("out.txt", [1, 2, 3])
```

See also: [read_file](core.md#read_file)

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
