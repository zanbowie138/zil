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
# → 0.296748
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
# → [1, 3, 5, 2, 4]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "312fb198-b897-4def-8ffe-1225b06c031c"
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
# → [2, 1, 1]
# float in a range
rand(1.0, 2.0)
# → 1.40327
# pick one
["rock", "paper", "scissors"].choice
# → "rock"
# shuffle
(1..=5).shuffle
# → [1, 4, 5, 2, 3]
# UUID
uuid()
# → "23dd2ab9-8524-4592-89f8-6e2192c8af0a"
```
