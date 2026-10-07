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
# → 0.781774
rand(1, 6)
# → 6
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
# → [5, 2, 3, 1, 4]
```

See also: [choice](../math/random.md#choice)

### uuid

`uuid()`: random v4 UUID

```zil
uuid()
# → "fe480666-bbcc-4e4d-9ee4-c030967b029b"
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
# → [1, 2, 3]
# float in a range
rand(1.0, 2.0)
# → 1.19599
# pick one
["rock", "paper", "scissors"].choice
# → "scissors"
# shuffle
(1..=5).shuffle
# → [1, 2, 5, 4, 3]
# UUID
uuid()
# → "bff0efb8-5100-4355-80c3-a4513cf944ae"
```
