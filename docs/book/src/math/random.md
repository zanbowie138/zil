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
# → 0.154055
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
# → [4, 1, 2, 3, 5]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "2b48cf90-5897-44eb-9ff3-5aaef185bf89"
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
# → [1, 2, 3]
# float in a range
rand(1.0, 2.0)
# → 1.59738
# pick one
["rock", "paper", "scissors"].choice
# → "rock"
# shuffle
(1..=5).shuffle
# → [3, 4, 2, 5, 1]
# UUID
uuid()
# → "d9835099-eeed-48c9-97e4-34333f4f2bd8"
```
