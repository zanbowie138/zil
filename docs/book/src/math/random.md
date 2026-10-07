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
# → 0.835321
rand(1, 6)
# → 5
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
# → [3, 2, 1, 4, 5]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "dbba0fea-7399-49b0-a0b4-caab92279baf"
```

See also: [rand](../math/random.md#rand), [ulid](../math/random.md#ulid)

### password

`password(len?: int)`: a password of letters, digits and symbols (default 20 characters, about 6.5 bits each), from the OS's secure generator

```zil
password()
# → "A!$MNqe=gPQcKwDe%WE3"
password(12)
# → "U@FeR6^kB-t9"
```

See also: [passphrase](../math/random.md#passphrase)

### passphrase

`passphrase(words?: int)`: random words joined by dashes (default 6, about 9.4 bits per word), from the OS's secure generator

```zil
passphrase()
# → "crawl-shark-extra-blunt-wagon-lion"
passphrase(4)
# → "panda-dizzy-salad-table"
```

See also: [password](../math/random.md#password)

### ulid

`ulid()`: a ULID: 26 characters that sort by creation time, then 80 random bits

```zil
ulid()
# → "01M4C0VRBA132EQE16TA2ZZ55Q"
```

See also: [uuid](../math/random.md#uuid), [nanoid](../math/random.md#nanoid)

### nanoid

`nanoid(len?: int)`: a URL-safe random ID (default 21 characters, like a UUID's strength)

```zil
nanoid()
# → "3Tde693CAWL9mAQaapK19"
nanoid(8)
# → "k8Ap-n1d"
```

See also: [uuid](../math/random.md#uuid), [ulid](../math/random.md#ulid)

## More examples

### random

```zil
# roll a die
rand(1, 6)
# → 4
# roll three
(1..=3).map(|_| rand(1, 6))
# → [2, 2, 3]
# float in a range
rand(1.0, 2.0)
# → 1.0059
# pick one
["rock", "paper", "scissors"].choice
# → "rock"
# shuffle
(1..=5).shuffle
# → [5, 3, 2, 1, 4]
# UUID
uuid()
# → "85502886-dd9c-4af2-a468-128731472ceb"
# a strong password
password()
# → "xHs5s&JY-XyGAYNgYtHu"
# one you can type
passphrase()
# → "rocky-jazzy-pedal-plank-stack-outer"
# sortable IDs
ulid()
# → "01M4C0VRBDFCANA83AN9XBTM26"
```
