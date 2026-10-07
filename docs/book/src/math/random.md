# math.random

numbers, picks, shuffles, UUIDs

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`rand() / rand(a: num, b: num)`](#rand) | float in [0, 1), or a number from a to b inclusive |
| [`choice(xs: list)`](#choice) | random item |
| [`shuffle(xs: list)`](#shuffle) | shuffled copy |
| [`uuid()`](#uuid) | random v4 UUID |

### rand

`rand() / rand(a: num, b: num)`: float in [0, 1), or a number from a to b inclusive

```zil
rand()
# → 0.141545
rand(1, 6)
# → 2
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
# → [5, 3, 4, 2, 1]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "1e19adb6-7016-4d41-91d7-fb669c11c7ba"
```

See also: [rand](../math/random.md#rand)

## More examples

### random

```zil
# roll a die
rand(1, 6)
# → 4
# roll three
(1..=3).map(|_| rand(1, 6))
# → [4, 2, 1]
# float in a range
rand(1.0, 2.0)
# → 1.88735
# pick one
["rock", "paper", "scissors"].choice
# → "paper"
# shuffle
(1..=5).shuffle
# → [2, 5, 3, 1, 4]
# UUID
uuid()
# → "0a2b9c42-1d05-43ed-979f-390c4eef2299"
```
