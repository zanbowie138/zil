# general

values, printing, conversions, number bases, parsing, files

> Example results generated on 2026-10-06.
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
# input  (stdin as a string)
```

### conversions

```zil
# to str int float frac bool list
"42" to int
# → 42
# to hex bin oct dec
255 to bin
# → 0b11111111
# to hex(bits), to base(b)
-1 to hex(16)
# → 0xffff
```

## Functions

| function | description |
|---|---|
| [`print(a, b, ...)`](#print) | print values separated by spaces |
| [`type(v)`](#type) | the type name of a value |
| [`str(v)`](#str) | convert to a string (full float precision) |
| [`int(v, base?)`](#int) | convert to an integer, parsing strings in an optional base |
| [`float(v)`](#float) | convert to a float; drops a quantity's unit |
| [`frac(v)`](#frac) | show as a fraction; floats become the simplest fraction within 1e-12 |
| [`bool(v)`](#bool) | truthiness: false only for nil and false |
| [`hex(v, bits?)`](#hex) | same as `v to hex` / `v to hex(bits)`; strings become hex bytes |
| [`bin(v, bits?)`](#bin) | same as `v to bin` / `v to bin(bits)` |
| [`oct(v, bits?)`](#oct) | same as `v to oct` |
| [`dec(v, bits?)`](#dec) | same as `v to dec`; with bits, reads two's complement as unsigned |
| [`base(v, b)`](#base) | same as `v to base(b)`, any base 2-36 |
| [`list(v)`](#list) | convert to a list: characters, a copy, or [key, value] pairs |
| [`parse(s)`](#parse) | read a zil literal (number, string, list, map, quantity); never runs code |
| [`len(v)`](#len) | length of a string, list or map |
| [`read_file(path)`](#read_file) | file contents as a string |
| [`write_file(path, v)`](#write_file) | write v to a file as text |
| [`help(topic?)`](#help) | this help; topic is a function, module, unit, or any value to list functions for its type |

### print

`print(a, b, ...)`: print values separated by spaces

See also: [str](general.md#str)

### type

`type(v)`: the type name of a value

```zil
type(5 km)
# → "quantity"
type("hi")
# → "str"
type([1])
# → "list"
```

See also: [str](general.md#str), [int](general.md#int), [float](general.md#float)

### str

`str(v)`: convert to a string (full float precision)

```zil
str(1/3)
# → "0.3333333333333333"
str(5 km)
# → "5 km"
```

See also: [int](general.md#int), [float](general.md#float)

### int

`int(v, base?)`: convert to an integer, parsing strings in an optional base

```zil
int(3.9)
# → 3
"ff".int(16)
# → 0xff
"0b101".int
# → 0b101
```

See also: [float](general.md#float), [str](general.md#str)

### float

`float(v)`: convert to a float; drops a quantity's unit

```zil
float("2.5")
# → 2.5
float(5 km)
# → 5
```

See also: [int](general.md#int), [str](general.md#str)

### frac

`frac(v)`: show as a fraction; floats become the simplest fraction within 1e-12

```zil
7/2 to frac
# → 7/2
1/3 + 1/6 to frac
# → 1/2
0.75.frac
# → 3/4
```

See also: [float](general.md#float)

### bool

`bool(v)`: truthiness: false only for nil and false

```zil
bool(0)
# → true
nil to bool
# → false
```

See also: [str](general.md#str)

### hex

`hex(v, bits?)`: same as `v to hex` / `v to hex(bits)`; strings become hex bytes

```zil
hex(255)
# → 0xff
hex(-1, 16)
# → 0xffff
hex("hi")
# → "6869"
```

See also: [bin](general.md#bin), [base](general.md#base), [int](general.md#int)

### bin

`bin(v, bits?)`: same as `v to bin` / `v to bin(bits)`

```zil
bin(10)
# → 0b1010
bin(5, 8)
# → 0b00000101
```

See also: [hex](general.md#hex), [oct](general.md#oct)

### oct

`oct(v, bits?)`: same as `v to oct`

```zil
oct(8)
# → 0o10
```

See also: [hex](general.md#hex), [bin](general.md#bin)

### dec

`dec(v, bits?)`: same as `v to dec`; with bits, reads two's complement as unsigned

```zil
dec(0xff)
# → 255
dec(-1, 8)
# → 255
```

See also: [hex](general.md#hex), [int](general.md#int)

### base

`base(v, b)`: same as `v to base(b)`, any base 2-36

```zil
base(35, 36)
# → 36#z
base(10, 3)
# → 3#101
```

See also: [hex](general.md#hex), [digits](math.md#digits)

### list

`list(v)`: convert to a list: characters, a copy, or [key, value] pairs

```zil
list("abc")
# → ["a", "b", "c"]
"abc" to list
# → ["a", "b", "c"]
{a: 1, b: 2}.list
# → [["a", 1], ["b", 2]]
```

See also: [chars](strings.md#chars), [parse](general.md#parse)

### parse

`parse(s)`: read a zil literal (number, string, list, map, quantity); never runs code

```zil
"[1, 2.5, 0xff]".parse
# → [1, 2.5, 0xff]
"5 km".parse to m
# → 5000 m
```

See also: [str](general.md#str), [nums](strings.md#nums)

### len

`len(v)`: length of a string, list or map

```zil
"héllo".len
# → 5
[1, 2, 3].len
# → 3
{a: 1}.len
# → 1
```

### read_file

`read_file(path)`: file contents as a string

See also: [write_file](general.md#write_file), [lines](strings.md#lines)

### write_file

`write_file(path, v)`: write v to a file as text

See also: [read_file](general.md#read_file)

### help

`help(topic?)`: this help; topic is a function, module, unit, or any value to list functions for its type

## More examples

### general

```zil
# type of anything
type(5 km)
# → "quantity"
# parse values from text
parse("[1, 2, 5 km]")
# → [1, 2, 5 km]
# number bases
255 to bin
# → 0b11111111
# any base back to decimal
36#zz to dec
# → 1295
# hex of a big int
factorial(20) to hex
# → 0x21c3677c82b40000
```
