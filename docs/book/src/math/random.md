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
# → 0.498433
rand(1, 6)
# → 1
```

See also: [choice](../math/random.md#choice), [shuffle](../math/random.md#shuffle)

### choice

`choice(xs: list)`: random item

```zil
["rock", "paper", "scissors"].choice
# → "rock"
```

See also: [rand](../math/random.md#rand), [shuffle](../math/random.md#shuffle)

### shuffle

`shuffle(xs: list)`: shuffled copy

```zil
(1..6).shuffle
# → [4, 5, 3, 1, 2]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "0ae20a1c-6ac2-4fbe-9fc8-a349aef046af"
```

See also: [rand](../math/random.md#rand), [ulid](../math/random.md#ulid)

### password

`password(len?: int)`: a password of letters, digits and symbols (default 20 characters, about 6.5 bits each), from the OS's secure generator

```zil
password()
# → "!z+V7y%cx*D+=J4_2bw@"
password(12)
# → "4YNUtyUf7doF"
```

See also: [passphrase](../math/random.md#passphrase)

### passphrase

`passphrase(words?: int)`: random words joined by dashes (default 6, about 9.4 bits per word), from the OS's secure generator

```zil
passphrase()
# → "sedan-gusto-greed-chunk-trail-arena"
passphrase(4)
# → "gummy-hazel-hurry-shell"
```

See also: [password](../math/random.md#password)

### ulid

`ulid()`: a ULID: 26 characters that sort by creation time, then 80 random bits

```zil
ulid()
# → "01M4BTASTN8A027KCVEBV22E64"
```

See also: [uuid](../math/random.md#uuid), [nanoid](../math/random.md#nanoid)

### nanoid

`nanoid(len?: int)`: a URL-safe random ID (default 21 characters, like a UUID's strength)

```zil
nanoid()
# → "sRFxdGozVMSAuMnO3j2a8"
nanoid(8)
# → "ePakqboG"
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
# → [6, 5, 6]
# float in a range
rand(1.0, 2.0)
# → 1.8354
# pick one
["rock", "paper", "scissors"].choice
# → "scissors"
# shuffle
(1..=5).shuffle
# → [4, 3, 1, 2, 5]
# UUID
uuid()
# → "e57e7bfe-99af-4f19-a404-f0da240a9e49"
# a strong password
password()
# → "$j+B7x4MtP5e@9?Hqqu2"
# one you can type
passphrase()
# → "brain-lotus-adobe-smile-yacht-angel"
# sortable IDs
ulid()
# → "01M4BTASTR9496ETC2YE98T63F"
```
