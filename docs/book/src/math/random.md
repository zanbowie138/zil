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
# → 0.40401
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
# → [2, 4, 1, 5, 3]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "1ecfe7c7-e183-4f5a-a61f-e4c29fc3e8cb"
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
# → [2, 3, 6]
# float in a range
rand(1.0, 2.0)
# → 1.50094
# pick one
["rock", "paper", "scissors"].choice
# → "scissors"
# shuffle
(1..=5).shuffle
# → [2, 1, 4, 5, 3]
# UUID
uuid()
# → "bbd216b8-ca31-4028-88bd-a8073b34277f"
```
