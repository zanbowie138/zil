# binary

hex dumps, entropy, checksums of strings and byte lists

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`hexdump(v)`](#hexdump) | xxd-style dump of a string's UTF-8 bytes or a list of bytes |
| [`entropy(v)`](#entropy) | Shannon entropy in bits per byte: 0 is constant, 8 is random |
| [`crc32(v)`](#crc32) | CRC-32 checksum (the zip/PNG one) |
| [`byte_len(s)`](#byte_len) | length in UTF-8 bytes rather than characters |

### hexdump

`hexdump(v)`: xxd-style dump of a string's UTF-8 bytes or a list of bytes

```zil
hexdump("hi\n")
# → "00000000: 6869 0a                                  hi."
hexdump([0, 255, 65])
# → "00000000: 00ff 41                                  ..A"
```

See also: [bytes](strings.md#bytes), [entropy](binary.md#entropy)

### entropy

`entropy(v)`: Shannon entropy in bits per byte: 0 is constant, 8 is random

```zil
"aaaa".entropy
# → 0
"abcd".entropy
# → 2
```

See also: [hexdump](binary.md#hexdump)

### crc32

`crc32(v)`: CRC-32 checksum (the zip/PNG one)

```zil
"hello".crc32
# → 907060870
"hello".crc32 to hex
# → 0x3610a686
```

See also: [sha256](strings.md#sha256), [md5](strings.md#md5)

### byte_len

`byte_len(s)`: length in UTF-8 bytes rather than characters

```zil
"héllo".byte_len
# → 6
```

See also: [len](general.md#len), [bytes](strings.md#bytes)

## More examples

### binary

```zil
# look inside a string
print(hexdump("héllo\tworld\n"))
# → nil
# how random is it?
"aaaaaaaa".entropy
# → 0
# checksum
"hello".crc32 to hex
# → 0x3610a686
# bytes vs characters
["😀".byte_len, "😀".len]
# → [4, 1]
```
