# text

case, splitting, search and regex; layout, comparison, translation, encodings, hashes and ciphers below

> Example results generated on 2026-10-07.
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

## Submodules

| module | about |
|---|---|
| [layout](text/layout.md) | pad, center, wrap, truncate, dedent; snake_case, camelCase, kebab-case, Title Case, slugs; words |
| [compare](text/compare.md) | edit distance, similarity, the closest of a list, line-by-line diffs |
| [translation](text/translation.md) | translate text between languages online; cached translations work offline |
| [encoding](text/encoding.md) | base64, base32, base58, URL and hex encodings, code points, UTF-8 bytes |
| [hash](text/hash.md) | SHA-1/256/512, MD5, HMAC and CRC-32 of strings |
| [ciphers](text/ciphers.md) | Caesar, ROT13, Atbash, Vigenère, Morse, Braille (for fun, not secrecy) |

## Functions

| function | description |
|---|---|
| [`upper(s: str)`](#upper) | uppercase a string |
| [`lower(s: str)`](#lower) | lowercase a string |
| [`trim(s: str)`](#trim) | strip leading and trailing whitespace |
| [`capitalize(s: str)`](#capitalize) | uppercase the first character |
| [`split(s: str, sep?: str\|regex)`](#split) | split on sep (string or regex); whitespace if omitted |
| [`lines(s: str)`](#lines) | split into lines |
| [`chars(s: str)`](#chars) | list of characters |
| [`join(xs: list, sep?: str)`](#join) | join items into a string |
| [`replace(s: str, pat: str\|regex, with: str)`](#replace) | replace all matches; regex replacements can use $1 |
| [`starts_with(s: str, prefix: str)`](#starts_with) | whether s starts with prefix |
| [`ends_with(s: str, suffix: str)`](#ends_with) | whether s ends with suffix |
| [`match(s: str, re: regex)`](#match) | first match, its capture groups, or nil |
| [`find_all(s: str, pat: str\|regex)`](#find_all) | list of all matches |
| [`grep(v: str\|list, pat: str\|regex)`](#grep) | lines of a string (or items of a list) containing pat |
| [`repeat(s: str, n: int)`](#repeat) | repeat a string n times |
| [`nums(s: str)`](#nums) | every number in a string |

### upper

`upper(s: str)`: uppercase a string

```zil
"hello".upper
# → "HELLO"
upper("zil")
# → "ZIL"
```

See also: [lower](text.md#lower), [capitalize](text.md#capitalize)

### lower

`lower(s: str)`: lowercase a string

```zil
"HeLLo".lower
# → "hello"
```

See also: [upper](text.md#upper), [capitalize](text.md#capitalize)

### trim

`trim(s: str)`: strip leading and trailing whitespace

```zil
"  hi  ".trim
# → "hi"
```

See also: [split](text.md#split)

### capitalize

`capitalize(s: str)`: uppercase the first character

```zil
"hello world".capitalize
# → "Hello world"
```

See also: [upper](text.md#upper)

### split

`split(s: str, sep?: str|regex)`: split on sep (string or regex); whitespace if omitted

```zil
"a b  c".split
# → ["a", "b", "c"]
"x,y;z".split(r"[,;]")
# → ["x", "y", "z"]
```

See also: [join](text.md#join), [lines](text.md#lines), [chars](text.md#chars)

### lines

`lines(s: str)`: split into lines

```zil
"a\nb".lines
# → ["a", "b"]
```

See also: [split](text.md#split)

### chars

`chars(s: str)`: list of characters

```zil
"abc".chars
# → ["a", "b", "c"]
```

See also: [split](text.md#split)

### join

`join(xs: list, sep?: str)`: join items into a string

```zil
["a", "b"].join("-")
# → "a-b"
[1, 2].join
# → "12"
```

See also: [split](text.md#split)

### replace

`replace(s: str, pat: str|regex, with: str)`: replace all matches; regex replacements can use $1

```zil
"a.b".replace(".", "-")
# → "a-b"
"a  b   c".replace(r"\s+", " ")
# → "a b c"
```

See also: [find_all](text.md#find_all), [match](text.md#match)

### starts_with

`starts_with(s: str, prefix: str)`: whether s starts with prefix

```zil
"abc".starts_with("a")
# → true
```

See also: [ends_with](text.md#ends_with), [contains](data.md#contains)

### ends_with

`ends_with(s: str, suffix: str)`: whether s ends with suffix

```zil
"file.zil".ends_with(".zil")
# → true
```

See also: [starts_with](text.md#starts_with)

### match

`match(s: str, re: regex)`: first match, its capture groups, or nil

```zil
"2026-10-06".match(r"(\d+)-(\d+)")
# → ["2026", "10"]
"id 42".match(r"\d+")
# → "42"
```

See also: [find_all](text.md#find_all), [replace](text.md#replace)

### find_all

`find_all(s: str, pat: str|regex)`: list of all matches

```zil
"a1b22c333".find_all(r"\d+")
# → ["1", "22", "333"]
```

See also: [match](text.md#match), [count](data.md#count)

### grep

`grep(v: str|list, pat: str|regex)`: lines of a string (or items of a list) containing pat

```zil
"ok\nERROR 1\nERROR 2".grep("ERROR")
# → ["ERROR 1", "ERROR 2"]
["a1", "b", "c22"].grep(r"\d")
# → ["a1", "c22"]
```

See also: [lines](text.md#lines), [filter](data/lists.md#filter), [contains](data.md#contains)

### repeat

`repeat(s: str, n: int)`: repeat a string n times

```zil
"ab".repeat(3)
# → "ababab"
```

### nums

`nums(s: str)`: every number in a string

```zil
"x=3, y=-2.5; 1e3".nums
# → [3, -2.5, 1000]
```

See also: [find_all](text.md#find_all), [parse](core.md#parse)

## More examples

### text

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
# printf
format("%-6s|%5.2f", "pi", pi)
# → "pi    | 3.14"
```
