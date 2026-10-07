# fun

fortune cookies, a magic 8-ball, coin flips, excuses, Roman numerals

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

### conversions

```zil
# to roman
1999 to roman
# → "MCMXCIX"
```

## Functions

| function | description |
|---|---|
| [`fortune()`](#fortune) | a fortune cookie |
| [`eight_ball(question?)`](#eight_ball) | a magic 8-ball answer; the same question gets the same answer all day |
| [`coin(n?)`](#coin) | heads or tails, or a list of n flips |
| [`yes_or_no(question?)`](#yes_or_no) | yes or no; leans yes on Fridays |
| [`excuse()`](#excuse) | why it doesn't work |
| [`roman(v)`](#roman) | an int 1-3999 as a Roman numeral, or a numeral back to an int; same as `n to roman` |

### fortune

`fortune()`: a fortune cookie

```zil
fortune()
# → "Rest. The code will still be broken tomorrow."
```

See also: [eight_ball](fun.md#eight_ball)

### eight_ball

`eight_ball(question?)`: a magic 8-ball answer; the same question gets the same answer all day

```zil
eight_ball("is it Friday?")
# → "It is decidedly so."
```

See also: [yes_or_no](fun.md#yes_or_no), [fortune](fun.md#fortune)

### coin

`coin(n?)`: heads or tails, or a list of n flips

```zil
coin()
# → "heads"
coin(3)
# → ["tails", "tails", "tails"]
```

See also: [rand](math/random.md#rand)

### yes_or_no

`yes_or_no(question?)`: yes or no; leans yes on Fridays

```zil
yes_or_no()
# → "no"
```

See also: [eight_ball](fun.md#eight_ball)

### excuse

`excuse()`: why it doesn't work

```zil
excuse()
# → "It must be a timezone thing."
```

See also: [fortune](fun.md#fortune)

### roman

`roman(v)`: an int 1-3999 as a Roman numeral, or a numeral back to an int; same as `n to roman`

```zil
roman(2026)
# → "MMXXVI"
"MCMXCIX".roman
# → 1999
```

## More examples

### fortune

```zil
# ask the ball
eight_ball("will it compile?")
# → "Don't count on it."
# best of five
coin(5)
# → ["tails", "heads", "heads", "heads", "tails"]
# standup
excuse()
# → "The tests were flaky."
# should I ship it?
yes_or_no()
# → "no"
```

### roman

```zil
# this year
2026 to roman
# → "MMXXVI"
# Super Bowl math
roman("LX") + 1 to roman
# → "LXI"
```
