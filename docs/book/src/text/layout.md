# text.layout

pad, center, wrap, truncate, dedent; snake_case, camelCase, kebab-case, Title Case, slugs; words

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`pad(s: any, width: int, fill?: str)`](#pad) | pad on the right to width characters (left-align) |
| [`pad_left(s: any, width: int, fill?: str)`](#pad_left) | pad on the left to width characters (right-align); numbers work too |
| [`center(s: any, width: int, fill?: str)`](#center) | center in width characters; extra fill goes right |
| [`truncate(s: str, n: int)`](#truncate) | cut to at most n characters, ending in … when cut |
| [`wrap(s: str, width: int)`](#wrap) | word-wrap each paragraph to width columns |
| [`dedent(s: str)`](#dedent) | remove the indentation every non-blank line shares |
| [`title(s: str)`](#title) | Capitalize Each Word |
| [`snake(s: str)`](#snake) | snake_case; splits on spaces, punctuation and camelCase humps |
| [`camel(s: str)`](#camel) | camelCase |
| [`kebab(s: str)`](#kebab) | kebab-case |
| [`slug(s: str)`](#slug) | a lowercase URL slug: letters and digits joined by - |
| [`words(s: str)`](#words) | the words, without punctuation (apostrophes stay) |
| [`word_count(s: str)`](#word_count) | how many words |

### pad

`pad(s: any, width: int, fill?: str)`: pad on the right to width characters (left-align)

```zil
"ab".pad(5) + "|"
# → "ab   |"
"ab".pad(5, ".")
# → "ab..."
```

See also: [pad_left](../text/layout.md#pad_left), [center](../text/layout.md#center)

### pad_left

`pad_left(s: any, width: int, fill?: str)`: pad on the left to width characters (right-align); numbers work too

```zil
42.pad_left(6, "0")
# → "000042"
```

See also: [pad](../text/layout.md#pad), [center](../text/layout.md#center)

### center

`center(s: any, width: int, fill?: str)`: center in width characters; extra fill goes right

```zil
"hi".center(8, "*")
# → "***hi***"
```

See also: [pad](../text/layout.md#pad), [pad_left](../text/layout.md#pad_left)

### truncate

`truncate(s: str, n: int)`: cut to at most n characters, ending in … when cut

```zil
"a long sentence here".truncate(10)
# → "a long se…"
```

See also: [wrap](../text/layout.md#wrap)

### wrap

`wrap(s: str, width: int)`: word-wrap each paragraph to width columns

```zil
"the quick brown fox jumps over the lazy dog".wrap(15)
# → "the quick brown\nfox jumps over\nthe lazy dog"
```

See also: [truncate](../text/layout.md#truncate), [dedent](../text/layout.md#dedent)

### dedent

`dedent(s: str)`: remove the indentation every non-blank line shares

```zil
"    a\n      b\n    c".dedent
# → "a\n  b\nc"
```

See also: [trim](../text.md#trim), [wrap](../text/layout.md#wrap)

### title

`title(s: str)`: Capitalize Each Word

```zil
"the lord of the rings".title
# → "The Lord Of The Rings"
```

See also: [capitalize](../text.md#capitalize), [snake](../text/layout.md#snake)

### snake

`snake(s: str)`: snake_case; splits on spaces, punctuation and camelCase humps

```zil
"parseHTTPRequest".snake
# → "parse_http_request"
"Total Price".snake
# → "total_price"
```

See also: [camel](../text/layout.md#camel), [kebab](../text/layout.md#kebab)

### camel

`camel(s: str)`: camelCase

```zil
"user_id_v2".camel
# → "userIdV2"
```

See also: [snake](../text/layout.md#snake), [kebab](../text/layout.md#kebab)

### kebab

`kebab(s: str)`: kebab-case

```zil
"MyComponentName".kebab
# → "my-component-name"
```

See also: [snake](../text/layout.md#snake), [slug](../text/layout.md#slug)

### slug

`slug(s: str)`: a lowercase URL slug: letters and digits joined by -

```zil
"Hello, World! It's 2026".slug
# → "hello-world-its-2026"
```

See also: [kebab](../text/layout.md#kebab)

### words

`words(s: str)`: the words, without punctuation (apostrophes stay)

```zil
"Hello, world! Don't panic.".words
# → ["Hello", "world", "Don't", "panic"]
```

See also: [word_count](../text/layout.md#word_count), [split](../text.md#split)

### word_count

`word_count(s: str)`: how many words

```zil
"Hello, world! Don't panic.".word_count
# → 4
```

See also: [words](../text/layout.md#words)

## More examples

### layout

```zil
# a receipt line
"coffee".pad(12, ".") + "$4.50".pad_left(8)
# → "coffee......   $4.50"
# a banner
"menu".upper.center(20, "=")
# → "========MENU========"
# column name to variable
"Total Price (USD)".snake
# → "total_price_usd"
# blog URL
"Hello, World! It's 2026".slug
# → "hello-world-its-2026"
# reading time
(lorem(450).word_count / 200).ceil
# → 3
```
