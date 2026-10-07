# dev

developer tools: JWTs, UUIDs, IP addresses and subnets, raw bytes, colors

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Submodules

| module | about |
|---|---|
| [net](dev/net.md) | IP addresses, CIDR blocks, subnets |
| [binary](dev/binary.md) | hex dumps and entropy of strings and byte lists |
| [colors](dev/colors.md) | colors as "#rrggbb": RGB/HSL, mixing, lightening, WCAG contrast |

## Functions

| function | description |
|---|---|
| [`jwt(s: str)`](#jwt) | decode a JWT into {header, payload, signature} without verifying it; exp, iat and nbf become dates |
| [`uuid_info(s: str)`](#uuid_info) | version and variant of a UUID, plus its timestamp for v1 and v7 |

### jwt

`jwt(s: str)`: decode a JWT into {header, payload, signature} without verifying it; exp, iat and nbf become dates

```zil
jwt("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZGEiLCJleHAiOjE5MjQ5OTIwMDB9.c2ln").payload.exp
# → 2030-12-31 18:00:00 -06:00
```

See also: [decode](text/encoding.md#decode), [hmac](text/hash.md#hmac)

### uuid_info

`uuid_info(s: str)`: version and variant of a UUID, plus its timestamp for v1 and v7

```zil
uuid_info(uuid())
# → {version: 4, variant: "RFC 9562"}
uuid_info("c232ab00-9414-11ec-b3c8-9f6bdeced846")
# → {version: 1, variant: "RFC 9562", timestamp: 2022-02-22 13:22:22 -06:00}
```

See also: [uuid](math/random.md#uuid)

## More examples

### dev

```zil
# what's in this token?
jwt("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZGEiLCJpYXQiOjE1MTYyMzkwMjJ9.c2ln").payload
# → {sub: "ada", iat: 2018-01-17 19:30:22 -06:00}
# when was this UUID made?
uuid_info("01890a5d-ac96-774b-bcce-b302099a8057").timestamp
# → 2023-06-29 22:34:18 -05:00
```
