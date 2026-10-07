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
# → 0.0114526
rand(1, 6)
# → 2
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
# → [5, 4, 1, 2, 3]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "717d1e6b-1b3e-48ab-9869-0b677721d6e3"
```

See also: [rand](../math/random.md#rand)

## More examples

### random

```zil
# roll a die
rand(1, 6)
# → 2
# roll three
(1..=3).map(|_| rand(1, 6))
# → [1, 2, 5]
# float in a range
rand(1.0, 2.0)
# → 1.37103
# pick one
["rock", "paper", "scissors"].choice
# → "paper"
# shuffle
(1..=5).shuffle
# → [3, 5, 2, 1, 4]
# UUID
uuid()
# → "18ce2942-970c-4eea-b105-538c295cc90b"
```
