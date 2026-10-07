# dev

developer tools: JWTs, UUIDs, URLs, semver, file permissions, IP addresses and subnets, raw bytes, colors

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
| [`url_parse(s: str)`](#url_parse) | split a URL into {scheme, host, port, path, query, fragment}; the query becomes a map, and repeated keys a list |
| [`url_build(m: map)`](#url_build) | the URL for a map like the one url_parse makes; missing parts are left out |
| [`semver(s: str)`](#semver) | parse a semantic version into {major, minor, patch, pre, build}; compares by version precedence, also against strings |
| [`bump(v: str\|map, part: str)`](#bump) | next "major", "minor" or "patch" version; a pre-release bumps to its own release |
| [`perm(mode: int\|str)`](#perm) | Unix permissions between octal and rwx: 755 or 0o755 gives "rwxr-xr-x", and back |

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

### url_parse

`url_parse(s: str)`: split a URL into {scheme, host, port, path, query, fragment}; the query becomes a map, and repeated keys a list

```zil
url_parse("https://example.com:8080/a/b?x=1&y=two%20words#top")
# → {scheme: "https", host: "example.com", port: 8080, path: "/a/b", query: {x: "1", y: "two words"}, fragment: "top"}
```

See also: [url_build](dev.md#url_build), [encode](text/encoding.md#encode)

### url_build

`url_build(m: map)`: the URL for a map like the one url_parse makes; missing parts are left out

```zil
url_build({scheme: "https", host: "example.com", path: "/search", query: {q: "a b"}})
# → "https://example.com/search?q=a%20b"
```

See also: [url_parse](dev.md#url_parse)

### semver

`semver(s: str)`: parse a semantic version into {major, minor, patch, pre, build}; compares by version precedence, also against strings

```zil
semver("1.2.3") < semver("1.10.0")
# → true
semver("1.0.0-beta") < "1.0.0"
# → true
semver("v2.1.0-rc.1+build5")
# → {major: 2, minor: 1, patch: 0, pre: "rc.1", build: "build5"}
```

See also: [bump](dev.md#bump)

### bump

`bump(v: str|map, part: str)`: next "major", "minor" or "patch" version; a pre-release bumps to its own release

```zil
bump("1.2.3", "minor")
# → "1.3.0"
bump("2.0.0-rc.1", "major")
# → "2.0.0"
```

See also: [semver](dev.md#semver)

### perm

`perm(mode: int|str)`: Unix permissions between octal and rwx: 755 or 0o755 gives "rwxr-xr-x", and back

```zil
perm(755)
# → "rwxr-xr-x"
perm("rw-r--r--")
# → 0o644
perm(0o4755)
# → "rwsr-xr-x"
```

## More examples

### dev

```zil
# what's in this token?
jwt("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZGEiLCJpYXQiOjE1MTYyMzkwMjJ9.c2ln").payload
# → {sub: "ada", iat: 2018-01-17 19:30:22 -06:00}
# when was this UUID made?
uuid_info("01890a5d-ac96-774b-bcce-b302099a8057").timestamp
# → 2023-06-29 22:34:18 -05:00
# query string of a URL
url_parse("https://example.com/search?q=zil&page=2").query
# → {q: "zil", page: "2"}
# is 1.10 newer than 1.9?
semver("1.10.0") > "1.9.3"
# → true
# chmod 640 means
perm(640)
# → "rw-r-----"
```
