# text.hash

SHA-256, MD5 and CRC-32 of strings

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`sha256(s)`](#sha256) | hex SHA-256 hash |
| [`md5(s)`](#md5) | hex MD5 hash |
| [`crc32(v)`](#crc32) | CRC-32 checksum (the zip/PNG one) |

### sha256

`sha256(s)`: hex SHA-256 hash

```zil
"hello".sha256[..16]
# → "2cf24dba5fb0a30e"
```

See also: [md5](../text/hash.md#md5)

### md5

`md5(s)`: hex MD5 hash

```zil
"hello".md5
# → "5d41402abc4b2a76b9719d911017c592"
```

See also: [sha256](../text/hash.md#sha256)

### crc32

`crc32(v)`: CRC-32 checksum (the zip/PNG one)

```zil
"hello".crc32
# → 907060870
"hello".crc32 to hex
# → 0x3610a686
```

See also: [sha256](../text/hash.md#sha256), [md5](../text/hash.md#md5)

## More examples

### hash

```zil
# hashes
"hello".md5
# → "5d41402abc4b2a76b9719d911017c592"
# checksum
"hello".crc32 to hex
# → 0x3610a686
```
