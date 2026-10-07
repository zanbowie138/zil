# math.random

numbers, picks, shuffles, UUIDs; passwords, passphrases, ULIDs and nano IDs from the OS's secure generator

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`rand() / rand(a: num, b: num)`](#rand) | float in [0, 1), or a number from a to b inclusive |
| [`choice(xs: list)`](#choice) | random item |
| [`shuffle(xs: list)`](#shuffle) | shuffled copy |
| [`uuid()`](#uuid) | random v4 UUID |
| [`password(len?: int)`](#password) | a password of letters, digits and symbols (default 20 characters, about 6.5 bits each), from the OS's secure generator |
| [`passphrase(words?: int)`](#passphrase) | random words joined by dashes (default 6, about 9.4 bits per word), from the OS's secure generator |
| [`ulid()`](#ulid) | a ULID: 26 characters that sort by creation time, then 80 random bits |
| [`nanoid(len?: int)`](#nanoid) | a URL-safe random ID (default 21 characters, like a UUID's strength) |

### rand

`rand() / rand(a: num, b: num)`: float in [0, 1), or a number from a to b inclusive

```zil
rand()
# → 0.936038
rand(1, 6)
# → 1
```

See also: [choice](../math/random.md#choice), [shuffle](../math/random.md#shuffle)

### choice

`choice(xs: list)`: random item

```zil
["rock", "paper", "scissors"].choice
# → "paper"
```

See also: [rand](../math/random.md#rand), [shuffle](../math/random.md#shuffle)

### shuffle

`shuffle(xs: list)`: shuffled copy

```zil
(1..6).shuffle
# → [1, 2, 3, 4, 5]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "07268028-826e-4ea6-8130-face52cef237"
```

See also: [rand](../math/random.md#rand), [ulid](../math/random.md#ulid)

### password

`password(len?: int)`: a password of letters, digits and symbols (default 20 characters, about 6.5 bits each), from the OS's secure generator

```zil
password()
# → "3yzMss2imZs5NiqG^LV%"
password(12)
# → "p5MufeA^D9Uk"
```

See also: [passphrase](../math/random.md#passphrase)

### passphrase

`passphrase(words?: int)`: random words joined by dashes (default 6, about 9.4 bits per word), from the OS's secure generator

```zil
passphrase()
# → "vocal-kettle-valve-ankle-jumpy-vague"
passphrase(4)
# → "party-input-avoid-birch"
```

See also: [password](../math/random.md#password)

### ulid

`ulid()`: a ULID: 26 characters that sort by creation time, then 80 random bits

```zil
ulid()
# → "01M4BS9T61819V9VXSFGP7NC01"
```

See also: [uuid](../math/random.md#uuid), [nanoid](../math/random.md#nanoid)

### nanoid

`nanoid(len?: int)`: a URL-safe random ID (default 21 characters, like a UUID's strength)

```zil
nanoid()
# → "-NREFl6GBjL8OWUYO3dM_"
nanoid(8)
# → "h2rNSkZ7"
```

See also: [uuid](../math/random.md#uuid), [ulid](../math/random.md#ulid)

## More examples

### random

```zil
# roll a die
rand(1, 6)
# → 2
# roll three
(1..=3).map(|_| rand(1, 6))
# → [3, 6, 2]
# float in a range
rand(1.0, 2.0)
# → 1.59226
# pick one
["rock", "paper", "scissors"].choice
# → "paper"
# shuffle
(1..=5).shuffle
# → [3, 2, 1, 4, 5]
# UUID
uuid()
# → "8e7cb704-1b01-48d3-86f6-3215aff5dd58"
# a strong password
password()
# → "k=uR3NqfQc2^m!!AQfS9"
# one you can type
passphrase()
# → "clay-flint-diver-quota-brine-polar"
# sortable IDs
ulid()
# → "01M4BS9T64T004H232BT71YETY"
```
