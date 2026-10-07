# random

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
# → 0.123784
rand(1, 6)
# → 1
```

See also: [choice](random.md#choice), [shuffle](random.md#shuffle)

### choice

`choice(list)`: random item

```zil
["rock", "paper", "scissors"].choice
# → "paper"
```

See also: [rand](random.md#rand), [shuffle](random.md#shuffle)

### shuffle

`shuffle(list)`: shuffled copy

```zil
(1..6).shuffle
# → [1, 5, 4, 3, 2]
```

See also: [choice](random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "270ff098-2deb-4076-a957-faaa2df23767"
```

See also: [rand](random.md#rand)

## More examples

### random

```zil
# roll a die
rand(1, 6)
# → 5
# roll three
(1..=3).map(|_| rand(1, 6))
# → [2, 3, 3]
# float in a range
rand(1.0, 2.0)
# → 1.29454
# pick one
["rock", "paper", "scissors"].choice
# → "scissors"
# shuffle
(1..=5).shuffle
# → [1, 2, 3, 5, 4]
# UUID
uuid()
# → "35c6bcb5-708e-43ea-9431-e2d222586a35"
```
