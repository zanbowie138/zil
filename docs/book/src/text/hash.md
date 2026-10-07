# text.hash

SHA-1/256/512, MD5, HMAC and CRC-32 of strings

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`sha1(s: str)`](#sha1) | hex SHA-1 hash |
| [`sha256(s: str)`](#sha256) | hex SHA-256 hash |
| [`sha512(s: str)`](#sha512) | hex SHA-512 hash |
| [`md5(s: str)`](#md5) | hex MD5 hash |
| [`hmac(s: str, key: str, alg?: str)`](#hmac) | hex HMAC of s; alg is "sha256" (default), "sha1", "sha512" or "md5" |
| [`crc32(v: str\|list)`](#crc32) | CRC-32 checksum (the zip/PNG one) |

### sha1

`sha1(s: str)`: hex SHA-1 hash

```zil
"hello".sha1
# → "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
```

See also: [sha256](../text/hash.md#sha256)

### sha256

`sha256(s: str)`: hex SHA-256 hash

```zil
"hello".sha256[..16]
# → "2cf24dba5fb0a30e"
```

See also: [md5](../text/hash.md#md5), [hmac](../text/hash.md#hmac)

### sha512

`sha512(s: str)`: hex SHA-512 hash

```zil
"hello".sha512[..16]
# → "9b71d224bd62f378"
```

See also: [sha256](../text/hash.md#sha256)

### md5

`md5(s: str)`: hex MD5 hash

```zil
"hello".md5
# → "5d41402abc4b2a76b9719d911017c592"
```

See also: [sha256](../text/hash.md#sha256)

### hmac

`hmac(s: str, key: str, alg?: str)`: hex HMAC of s; alg is "sha256" (default), "sha1", "sha512" or "md5"

```zil
hmac("msg", "secret")
# → "fe4f9c418f683f034f6af90d1dd5b86ac0355dd96332c59cc74598d0736107f6"
hmac("msg", "secret", "sha1")
# → "ca3efb15cfaf530e0d771e5cbe707104f006b257"
```

See also: [sha256](../text/hash.md#sha256)

### crc32

`crc32(v: str|list)`: CRC-32 checksum (the zip/PNG one)

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
# sign a message
hmac("msg", "secret")
# → "fe4f9c418f683f034f6af90d1dd5b86ac0355dd96332c59cc74598d0736107f6"
```
