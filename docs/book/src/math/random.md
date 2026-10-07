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
# → 0.506564
rand(1, 6)
# → 5
```

See also: [choice](../math/random.md#choice), [shuffle](../math/random.md#shuffle)

### choice

`choice(xs: list)`: random item

```zil
["rock", "paper", "scissors"].choice
# → "scissors"
```

See also: [rand](../math/random.md#rand), [shuffle](../math/random.md#shuffle)

### shuffle

`shuffle(xs: list)`: shuffled copy

```zil
(1..6).shuffle
# → [1, 2, 5, 3, 4]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "4b88b390-fa7b-4f07-b28e-19fe425d6654"
```

See also: [rand](../math/random.md#rand), [ulid](../math/random.md#ulid)

### password

`password(len?: int)`: a password of letters, digits and symbols (default 20 characters, about 6.5 bits each), from the OS's secure generator

```zil
password()
# → "jAyui4h@fx2BFaSHyzcL"
password(12)
# → "SS+wbf2RuUvA"
```

See also: [passphrase](../math/random.md#passphrase)

### passphrase

`passphrase(words?: int)`: random words joined by dashes (default 6, about 9.4 bits per word), from the OS's secure generator

```zil
passphrase()
# → "koala-jelly-peach-plank-champ-guide"
passphrase(4)
# → "bench-humor-blast-minty"
```

See also: [password](../math/random.md#password)

### ulid

`ulid()`: a ULID: 26 characters that sort by creation time, then 80 random bits

```zil
ulid()
# → "01M4BR3N4NC2675FW3F9EHZSPD"
```

See also: [uuid](../math/random.md#uuid), [nanoid](../math/random.md#nanoid)

### nanoid

`nanoid(len?: int)`: a URL-safe random ID (default 21 characters, like a UUID's strength)

```zil
nanoid()
# → "l3mKllUu6B4QSeHGOPECx"
nanoid(8)
# → "L1yatBKM"
```

See also: [uuid](../math/random.md#uuid), [ulid](../math/random.md#ulid)

## More examples

### random

```zil
# roll a die
rand(1, 6)
# → 6
# roll three
(1..=3).map(|_| rand(1, 6))
# → [6, 2, 5]
# float in a range
rand(1.0, 2.0)
# → 1.01534
# pick one
["rock", "paper", "scissors"].choice
# → "paper"
# shuffle
(1..=5).shuffle
# → [2, 3, 4, 5, 1]
# UUID
uuid()
# → "72268adc-f63d-46c4-a85f-8e9f747985ca"
# a strong password
password()
# → "EFS*7zFgf9kWM3xwf_Q-"
# one you can type
passphrase()
# → "harsh-paper-envoy-horse-label-siren"
# sortable IDs
ulid()
# → "01M4BR3N4RRR3SZ9HAQG6HH4HE"
```
