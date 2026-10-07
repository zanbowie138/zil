# oracles

fortune cookies, a magic 8-ball, coin flips, excuses

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`fortune()`](#fortune) | a fortune cookie |
| [`eight_ball(question?)`](#eight_ball) | a magic 8-ball answer; the same question gets the same answer all day |
| [`coin(n?)`](#coin) | heads or tails, or a list of n flips |
| [`yes_or_no(question?)`](#yes_or_no) | yes or no; leans yes on Fridays |
| [`excuse()`](#excuse) | why it doesn't work |

### fortune

`fortune()`: a fortune cookie

```zil
fortune()
# → "The cache is lying to you."
```

See also: [eight_ball](oracles.md#eight_ball)

### eight_ball

`eight_ball(question?)`: a magic 8-ball answer; the same question gets the same answer all day

```zil
eight_ball("is it Friday?")
# → "It is decidedly so."
```

See also: [yes_or_no](oracles.md#yes_or_no), [fortune](oracles.md#fortune)

### coin

`coin(n?)`: heads or tails, or a list of n flips

```zil
coin()
# → "tails"
coin(3)
# → ["heads", "tails", "tails"]
```

See also: [rand](random.md#rand)

### yes_or_no

`yes_or_no(question?)`: yes or no; leans yes on Fridays

```zil
yes_or_no()
# → "no"
```

See also: [eight_ball](oracles.md#eight_ball)

### excuse

`excuse()`: why it doesn't work

```zil
excuse()
# → "Mercury is in retrograde."
```

See also: [fortune](oracles.md#fortune)

## More examples

### fortune

```zil
# ask the ball
eight_ball("will it compile?")
# → "Don't count on it."
# best of five
coin(5)
# → ["tails", "heads", "heads", "heads", "heads"]
# standup
excuse()
# → "I was told the requirements would not change."
# should I ship it?
yes_or_no()
# → "yes"
```
