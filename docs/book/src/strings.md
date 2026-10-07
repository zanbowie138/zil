# strings

case, splitting, search and regex, encodings, hashes

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

### types

```zil
# str
"héllo\tworld"
# regex
r"\d+"
```

### operators

```zil
# str + v
"v" + 2
# → "v2"
# str * int
"ab" * 3
# → "ababab"
```

### conversions

```zil
# to base64
"hi" to base64
# → "aGk="
```

## Functions

| function | description |
|---|---|
| [`upper(s)`](#upper) | uppercase a string |
| [`lower(s)`](#lower) | lowercase a string |
| [`trim(s)`](#trim) | strip leading and trailing whitespace |
| [`capitalize(s)`](#capitalize) | uppercase the first character |
| [`reverse(v)`](#reverse) | reverse a string or list |
| [`split(s, sep?)`](#split) | split on sep (string or regex); whitespace if omitted |
| [`lines(s)`](#lines) | split into lines |
| [`chars(s)`](#chars) | list of characters |
| [`join(list, sep?)`](#join) | join items into a string |
| [`replace(s, pat, with)`](#replace) | replace all matches; regex replacements can use $1 |
| [`contains(v, x)`](#contains) | substring/regex in a string, item in a list, key in a map |
| [`starts_with(s, prefix)`](#starts_with) | whether s starts with prefix |
| [`ends_with(s, suffix)`](#ends_with) | whether s ends with suffix |
| [`find(v, x)`](#find) | index of the first match, or nil |
| [`count(v, x)`](#count) | number of matches in a string or list |
| [`match(s, regex)`](#match) | first match, its capture groups, or nil |
| [`find_all(s, pat)`](#find_all) | list of all matches |
| [`grep(v, pat)`](#grep) | lines of a string (or items of a list) containing pat |
| [`repeat(s, n)`](#repeat) | repeat a string n times |
| [`base64(s)`](#base64) | base64-encode a string; same as `s to base64` |
| [`encode(s, fmt)`](#encode) | encode as "base64", "url" or "hex" |
| [`decode(s, fmt)`](#decode) | decode "base64", "url" or "hex" |
| [`sha256(s)`](#sha256) | hex SHA-256 hash |
| [`md5(s)`](#md5) | hex MD5 hash |
| [`ord(c)`](#ord) | Unicode code point of a single character |
| [`chr(n)`](#chr) | character for a Unicode code point |
| [`nums(s)`](#nums) | every number in a string |
| [`bytes(s)`](#bytes) | list of the string's UTF-8 bytes |
| [`from_bytes(list)`](#from_bytes) | string from a list of UTF-8 bytes |

### upper

`upper(s)`: uppercase a string

```zil
"hello".upper
# → "HELLO"
upper("zil")
# → "ZIL"
```

See also: [lower](strings.md#lower), [capitalize](strings.md#capitalize)

### lower

`lower(s)`: lowercase a string

```zil
"HeLLo".lower
# → "hello"
```

See also: [upper](strings.md#upper), [capitalize](strings.md#capitalize)

### trim

`trim(s)`: strip leading and trailing whitespace

```zil
"  hi  ".trim
# → "hi"
```

See also: [split](strings.md#split)

### capitalize

`capitalize(s)`: uppercase the first character

```zil
"hello world".capitalize
# → "Hello world"
```

See also: [upper](strings.md#upper)

### reverse

`reverse(v)`: reverse a string or list

```zil
"abc".reverse
# → "cba"
[1, 2, 3].reverse
# → [3, 2, 1]
```

See also: [sort](lists.md#sort)

### split

`split(s, sep?)`: split on sep (string or regex); whitespace if omitted

```zil
"a b  c".split
# → ["a", "b", "c"]
"x,y;z".split(r"[,;]")
# → ["x", "y", "z"]
```

See also: [join](strings.md#join), [lines](strings.md#lines), [chars](strings.md#chars)

### lines

`lines(s)`: split into lines

```zil
"a\nb".lines
# → ["a", "b"]
```

See also: [split](strings.md#split)

### chars

`chars(s)`: list of characters

```zil
"abc".chars
# → ["a", "b", "c"]
```

See also: [split](strings.md#split)

### join

`join(list, sep?)`: join items into a string

```zil
["a", "b"].join("-")
# → "a-b"
[1, 2].join
# → "12"
```

See also: [split](strings.md#split)

### replace

`replace(s, pat, with)`: replace all matches; regex replacements can use $1

```zil
"a.b".replace(".", "-")
# → "a-b"
"a  b   c".replace(r"\s+", " ")
# → "a b c"
```

See also: [find_all](strings.md#find_all), [match](strings.md#match)

### contains

`contains(v, x)`: substring/regex in a string, item in a list, key in a map

```zil
"price: $12".contains(r"\$\d+")
# → true
[1, 2].contains(2)
# → true
```

See also: [find](strings.md#find), [starts_with](strings.md#starts_with)

### starts_with

`starts_with(s, prefix)`: whether s starts with prefix

```zil
"abc".starts_with("a")
# → true
```

See also: [ends_with](strings.md#ends_with), [contains](strings.md#contains)

### ends_with

`ends_with(s, suffix)`: whether s ends with suffix

```zil
"file.zil".ends_with(".zil")
# → true
```

See also: [starts_with](strings.md#starts_with)

### find

`find(v, x)`: index of the first match, or nil

```zil
"hello".find("l")
# → 2
[5, 6].find(6)
# → 1
"abc".find("z")
# → nil
```

See also: [contains](strings.md#contains), [count](strings.md#count)

### count

`count(v, x)`: number of matches in a string or list

```zil
"banana".count("a")
# → 3
[1, 2, 1].count(1)
# → 2
```

See also: [find](strings.md#find)

### match

`match(s, regex)`: first match, its capture groups, or nil

```zil
"2026-10-06".match(r"(\d+)-(\d+)")
# → ["2026", "10"]
"id 42".match(r"\d+")
# → "42"
```

See also: [find_all](strings.md#find_all), [replace](strings.md#replace)

### find_all

`find_all(s, pat)`: list of all matches

```zil
"a1b22c333".find_all(r"\d+")
# → ["1", "22", "333"]
```

See also: [match](strings.md#match), [count](strings.md#count)

### grep

`grep(v, pat)`: lines of a string (or items of a list) containing pat

```zil
"ok\nERROR 1\nERROR 2".grep("ERROR")
# → ["ERROR 1", "ERROR 2"]
["a1", "b", "c22"].grep(r"\d")
# → ["a1", "c22"]
```

See also: [lines](strings.md#lines), [filter](lists.md#filter), [contains](strings.md#contains)

### repeat

`repeat(s, n)`: repeat a string n times

```zil
"ab".repeat(3)
# → "ababab"
```

### base64

`base64(s)`: base64-encode a string; same as `s to base64`

```zil
base64("hi there")
# → "aGkgdGhlcmU="
"hi" to base64
# → "aGk="
```

See also: [encode](strings.md#encode), [decode](strings.md#decode)

### encode

`encode(s, fmt)`: encode as "base64", "url" or "hex"

```zil
"hi there".encode("base64")
# → "aGkgdGhlcmU="
"a b&c".encode("url")
# → "a%20b%26c"
```

See also: [decode](strings.md#decode)

### decode

`decode(s, fmt)`: decode "base64", "url" or "hex"

```zil
"aGk=".decode("base64")
# → "hi"
"6869".decode("hex")
# → "hi"
```

See also: [encode](strings.md#encode)

### sha256

`sha256(s)`: hex SHA-256 hash

```zil
"hello".sha256[..16]
# → "2cf24dba5fb0a30e"
```

See also: [md5](strings.md#md5)

### md5

`md5(s)`: hex MD5 hash

```zil
"hello".md5
# → "5d41402abc4b2a76b9719d911017c592"
```

See also: [sha256](strings.md#sha256)

### ord

`ord(c)`: Unicode code point of a single character

```zil
"A".ord
# → 65
"A".ord to hex
# → 0x41
```

See also: [chr](strings.md#chr), [bytes](strings.md#bytes)

### chr

`chr(n)`: character for a Unicode code point

```zil
97.chr
# → "a"
(65..70).map(chr).join
# → "ABCDE"
```

See also: [ord](strings.md#ord)

### nums

`nums(s)`: every number in a string

```zil
"x=3, y=-2.5; 1e3".nums
# → [3, -2.5, 1000]
```

See also: [find_all](strings.md#find_all), [parse](general.md#parse)

### bytes

`bytes(s)`: list of the string's UTF-8 bytes

```zil
"hé".bytes
# → [104, 195, 169]
```

See also: [from_bytes](strings.md#from_bytes), [ord](strings.md#ord)

### from_bytes

`from_bytes(list)`: string from a list of UTF-8 bytes

```zil
[104, 105].from_bytes
# → "hi"
```

See also: [bytes](strings.md#bytes), [chr](strings.md#chr)

## More examples

### strings

```zil
# numbers out of text
"a1b22c333".nums.sum
# → 356
# regex captures
"2026-10-06".match(r"(\d+)-(\d+)-(\d+)")
# → ["2026", "10", "06"]
# regex replace with $1
"CamelCaseName".replace(r"(\B[A-Z])", " $1")
# → "Camel Case Name"
# grep lines
"ok\nERROR disk full\nok".grep(r"ERROR")
# → ["ERROR disk full"]
# initials
"Ada Lovelace".split.map(|w| w[0]).join
# → "AL"
# hashes
"hello".md5
# → "5d41402abc4b2a76b9719d911017c592"
# base64
"hi" to base64
# → "aGk="
# URL encoding
"a b&c".encode("url")
# → "a%20b%26c"
# UTF-8 bytes
"héllo".bytes
# → [104, 195, 169, 108, 108, 111]
# code point to char
chr(9731)
# → "☃"
# printf
format("%-6s|%5.2f", "pi", pi)
# → "pi    | 3.14"
```
