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
# → 0.222698
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
# → [5, 4, 2, 3, 1]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "1f61ea1f-0a3e-403e-b0d8-c53fdd5b177e"
```

See also: [rand](../math/random.md#rand)

## More examples

### random

```zil
# roll a die
rand(1, 6)
# → 6
# roll three
(1..=3).map(|_| rand(1, 6))
# → [2, 2, 4]
# float in a range
rand(1.0, 2.0)
# → 1.67029
# pick one
["rock", "paper", "scissors"].choice
# → "rock"
# shuffle
(1..=5).shuffle
# → [2, 3, 5, 4, 1]
# UUID
uuid()
# → "7285d884-451b-4dab-be9c-77463be0fc72"
```
