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
# → 0.877266
rand(1, 6)
# → 2
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
# → [5, 3, 2, 1, 4]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "333f9856-24fb-4151-8472-763d123d0027"
```

See also: [rand](../math/random.md#rand), [ulid](../math/random.md#ulid)

### password

`password(len?: int)`: a password of letters, digits and symbols (default 20 characters, about 6.5 bits each), from the OS's secure generator

```zil
password()
# → "YxVhfsx9-nA+ASy2?yt+"
password(12)
# → "?4+x-T=ff%3t"
```

See also: [passphrase](../math/random.md#passphrase)

### passphrase

`passphrase(words?: int)`: random words joined by dashes (default 6, about 9.4 bits per word), from the OS's secure generator

```zil
passphrase()
# → "eagle-rogue-focal-brush-coast-gorge"
passphrase(4)
# → "float-habit-parka-angle"
```

See also: [password](../math/random.md#password)

### ulid

`ulid()`: a ULID: 26 characters that sort by creation time, then 80 random bits

```zil
ulid()
# → "01M4BZ4DGGKT5NTQ8AVKZ53B25"
```

See also: [uuid](../math/random.md#uuid), [nanoid](../math/random.md#nanoid)

### nanoid

`nanoid(len?: int)`: a URL-safe random ID (default 21 characters, like a UUID's strength)

```zil
nanoid()
# → "nKFbUjcoZaCNu7ja6SUth"
nanoid(8)
# → "9xKQoOhh"
```

See also: [uuid](../math/random.md#uuid), [ulid](../math/random.md#ulid)

## More examples

### random

```zil
# roll a die
rand(1, 6)
# → 5
# roll three
(1..=3).map(|_| rand(1, 6))
# → [1, 5, 5]
# float in a range
rand(1.0, 2.0)
# → 1.89367
# pick one
["rock", "paper", "scissors"].choice
# → "paper"
# shuffle
(1..=5).shuffle
# → [2, 1, 4, 5, 3]
# UUID
uuid()
# → "f5f47c0d-ad09-4a43-82c6-53eb361615e3"
# a strong password
password()
# → "C-dogNNBQ_Dvvafpks7#"
# one you can type
passphrase()
# → "cupid-fleet-pinch-venom-steam-actor"
# sortable IDs
ulid()
# → "01M4BZ4DGMSHRA82CB76YZQCTB"
```
