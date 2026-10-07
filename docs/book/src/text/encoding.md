# text.encoding

base64, URL and hex encodings, code points, UTF-8 bytes

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

### conversions

```zil
# to base64
"hi" to base64
# → "aGk="
```

## Functions

| function | description |
|---|---|
| [`base64(s)`](#base64) | base64-encode a string; same as `s to base64` |
| [`encode(s, fmt)`](#encode) | encode as "base64", "url" or "hex" |
| [`decode(s, fmt)`](#decode) | decode "base64", "url" or "hex" |
| [`ord(c)`](#ord) | Unicode code point of a single character |
| [`chr(n)`](#chr) | character for a Unicode code point |
| [`bytes(s)`](#bytes) | list of the string's UTF-8 bytes |
| [`from_bytes(list)`](#from_bytes) | string from a list of UTF-8 bytes |
| [`byte_len(s)`](#byte_len) | length in UTF-8 bytes rather than characters |

### base64

`base64(s)`: base64-encode a string; same as `s to base64`

```zil
base64("hi there")
# → "aGkgdGhlcmU="
"hi" to base64
# → "aGk="
```

See also: [encode](../text/encoding.md#encode), [decode](../text/encoding.md#decode)

### encode

`encode(s, fmt)`: encode as "base64", "url" or "hex"

```zil
"hi there".encode("base64")
# → "aGkgdGhlcmU="
"a b&c".encode("url")
# → "a%20b%26c"
```

See also: [decode](../text/encoding.md#decode)

### decode

`decode(s, fmt)`: decode "base64", "url" or "hex"

```zil
"aGk=".decode("base64")
# → "hi"
"6869".decode("hex")
# → "hi"
```

See also: [encode](../text/encoding.md#encode)

### ord

`ord(c)`: Unicode code point of a single character

```zil
"A".ord
# → 65
"A".ord to hex
# → 0x41
```

See also: [chr](../text/encoding.md#chr), [bytes](../text/encoding.md#bytes)

### chr

`chr(n)`: character for a Unicode code point

```zil
97.chr
# → "a"
(65..70).map(chr).join
# → "ABCDE"
```

See also: [ord](../text/encoding.md#ord)

### bytes

`bytes(s)`: list of the string's UTF-8 bytes

```zil
"hé".bytes
# → [104, 195, 169]
```

See also: [from_bytes](../text/encoding.md#from_bytes), [ord](../text/encoding.md#ord)

### from_bytes

`from_bytes(list)`: string from a list of UTF-8 bytes

```zil
[104, 105].from_bytes
# → "hi"
```

See also: [bytes](../text/encoding.md#bytes), [chr](../text/encoding.md#chr)

### byte_len

`byte_len(s)`: length in UTF-8 bytes rather than characters

```zil
"héllo".byte_len
# → 6
```

See also: [len](../data.md#len), [bytes](../text/encoding.md#bytes)

## More examples

### encoding

```zil
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
# bytes vs characters
["😀".byte_len, "😀".len]
# → [4, 1]
```
