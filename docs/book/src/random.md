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
# → 0.343707
rand(1, 6)
# → 2
```

See also: [choice](random.md#choice), [shuffle](random.md#shuffle)

### choice

`choice(list)`: random item

```zil
["rock", "paper", "scissors"].choice
# → "rock"
```

See also: [rand](random.md#rand), [shuffle](random.md#shuffle)

### shuffle

`shuffle(list)`: shuffled copy

```zil
(1..6).shuffle
# → [3, 2, 1, 5, 4]
```

See also: [choice](random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "35e602f2-81f0-46dd-a064-24adb6c11d88"
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
# → [6, 3, 5]
# float in a range
rand(1.0, 2.0)
# → 1.19531
# pick one
["rock", "paper", "scissors"].choice
# → "paper"
# shuffle
(1..=5).shuffle
# → [3, 5, 4, 1, 2]
# UUID
uuid()
# → "5687ed62-6ed1-40fd-8984-4905efae603b"
```
