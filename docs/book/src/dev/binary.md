# dev.binary

hex dumps and entropy of strings and byte lists

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`hexdump(v)`](#hexdump) | xxd-style dump of a string's UTF-8 bytes or a list of bytes |
| [`entropy(v)`](#entropy) | Shannon entropy in bits per byte: 0 is constant, 8 is random |

### hexdump

`hexdump(v)`: xxd-style dump of a string's UTF-8 bytes or a list of bytes

```zil
hexdump("hi\n")
# → "00000000: 6869 0a                                  hi."
hexdump([0, 255, 65])
# → "00000000: 00ff 41                                  ..A"
```

See also: [bytes](../text/encoding.md#bytes), [entropy](../dev/binary.md#entropy)

### entropy

`entropy(v)`: Shannon entropy in bits per byte: 0 is constant, 8 is random

```zil
"aaaa".entropy
# → 0
"abcd".entropy
# → 2
```

See also: [hexdump](../dev/binary.md#hexdump)

## More examples

### binary

```zil
# look inside a string
print(hexdump("héllo\tworld\n"))
# → nil
# how random is it?
"aaaaaaaa".entropy
# → 0
```
