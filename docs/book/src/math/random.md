# math.random

numbers, picks, shuffles, UUIDs

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`rand() / rand(a, b)`](#rand) | float in [0, 1), or a number from a to b inclusive |
| [`choice(list)`](#choice) | random item |
| [`shuffle(list)`](#shuffle) | shuffled copy |
| [`uuid()`](#uuid) | random v4 UUID |

### rand

`rand() / rand(a, b)`: float in [0, 1), or a number from a to b inclusive

```zil
rand()
# → 0.759919
rand(1, 6)
# → 6
```

See also: [choice](../math/random.md#choice), [shuffle](../math/random.md#shuffle)

### choice

`choice(list)`: random item

```zil
["rock", "paper", "scissors"].choice
# → "paper"
```

See also: [rand](../math/random.md#rand), [shuffle](../math/random.md#shuffle)

### shuffle

`shuffle(list)`: shuffled copy

```zil
(1..6).shuffle
# → [3, 2, 1, 5, 4]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "f9a63cf1-a3fe-4654-b0f2-5ee75dae7782"
```

See also: [rand](../math/random.md#rand)

## More examples

### random

```zil
# roll a die
rand(1, 6)
# → 5
# roll three
(1..=3).map(|_| rand(1, 6))
# → [5, 2, 1]
# float in a range
rand(1.0, 2.0)
# → 1.9238
# pick one
["rock", "paper", "scissors"].choice
# → "rock"
# shuffle
(1..=5).shuffle
# → [5, 1, 4, 2, 3]
# UUID
uuid()
# → "b237106e-cc50-46d1-88ef-476571a7896b"
```
