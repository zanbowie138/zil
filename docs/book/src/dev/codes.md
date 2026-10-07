# dev.codes

check digits for cards, ISBNs and IBANs; HTTP status, port and MIME type lookups

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`luhn(s: str\|int)`](#luhn) | whether the Luhn check digit is right, as on credit cards and IMEIs; spaces and dashes are ignored |
| [`isbn(s: str)`](#isbn) | whether an ISBN-10 or ISBN-13 has a valid check digit |
| [`iban(s: str)`](#iban) | whether an IBAN passes its mod-97 check |
| [`http_status(code: int)`](#http_status) | the reason phrase for an HTTP status code, or nil |
| [`port(n: int) / port(name: str)`](#port) | the service on a well-known port, or the port for a service; nil if unknown |
| [`mime(name: str)`](#mime) | the MIME type for a file name or extension, or nil |

### luhn

`luhn(s: str|int)`: whether the Luhn check digit is right, as on credit cards and IMEIs; spaces and dashes are ignored

```zil
luhn("4111 1111 1111 1111")
# → true
luhn(79927398710)
# → false
```

See also: [isbn](../dev/codes.md#isbn), [iban](../dev/codes.md#iban)

### isbn

`isbn(s: str)`: whether an ISBN-10 or ISBN-13 has a valid check digit

```zil
isbn("0-306-40615-2")
# → true
isbn("9780306406157")
# → true
```

See also: [luhn](../dev/codes.md#luhn)

### iban

`iban(s: str)`: whether an IBAN passes its mod-97 check

```zil
iban("GB82 WEST 1234 5698 7654 32")
# → true
```

See also: [luhn](../dev/codes.md#luhn)

### http_status

`http_status(code: int)`: the reason phrase for an HTTP status code, or nil

```zil
http_status(404)
# → "Not Found"
(200..=204).map(http_status)
# → ["OK", "Created", "Accepted", "Non-Authoritative Information", "No Content"]
```

See also: [port](../dev/codes.md#port)

### port

`port(n: int) / port(name: str)`: the service on a well-known port, or the port for a service; nil if unknown

```zil
port(22)
# → "ssh"
port("postgresql")
# → 5432
```

See also: [http_status](../dev/codes.md#http_status), [mime](../dev/codes.md#mime)

### mime

`mime(name: str)`: the MIME type for a file name or extension, or nil

```zil
mime("png")
# → "image/png"
mime("data.json")
# → "application/json"
```

See also: [ext](../fs.md#ext)

## More examples

### codes

```zil
# a typo in a card number?
luhn("4111 1111 1111 1112")
# → false
# valid ISBN?
isbn("978-0-306-40615-7")
# → true
# what's a 418?
http_status(418)
# → "I'm a teapot"
# what runs on 5432?
port(5432)
# → "postgresql"
# Content-Type for a file
mime("report.pdf")
# → "application/pdf"
```
