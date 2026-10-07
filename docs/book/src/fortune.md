# fortune

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
# → "A wise developer once said: it works on my machine."
```

See also: [eight_ball](fortune.md#eight_ball)

### eight_ball

`eight_ball(question?)`: a magic 8-ball answer; the same question gets the same answer all day

```zil
eight_ball("is it Friday?")
# → "It is decidedly so."
```

See also: [yes_or_no](fortune.md#yes_or_no), [fortune](fortune.md#fortune)

### coin

`coin(n?)`: heads or tails, or a list of n flips

```zil
coin()
# → "tails"
coin(3)
# → ["tails", "heads", "heads"]
```

See also: [rand](random.md#rand)

### yes_or_no

`yes_or_no(question?)`: yes or no; leans yes on Fridays

```zil
yes_or_no()
# → "yes"
```

See also: [eight_ball](fortune.md#eight_ball)

### excuse

`excuse()`: why it doesn't work

```zil
excuse()
# → "It must be a timezone thing."
```

See also: [fortune](fortune.md#fortune)

## More examples

### fortune

```zil
# ask the ball
eight_ball("will it compile?")
# → "Don't count on it."
# best of five
coin(5)
# → ["heads", "tails", "heads", "heads", "tails"]
# standup
excuse()
# → "The intern had root access."
# should I ship it?
yes_or_no()
# → "yes"
```
