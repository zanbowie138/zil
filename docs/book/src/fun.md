# fun

fortune cookies, a magic 8-ball, coin flips, excuses, Roman numerals; lorem ipsum placeholder text below

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### conversions

```zil
# to roman
1999 to roman
# → "MCMXCIX"
```

## Submodules

| module | about |
|---|---|
| [placeholder](fun/placeholder.md) | lorem ipsum placeholder text, random team and person names |

## Functions

| function | description |
|---|---|
| [`fortune()`](#fortune) | a fortune cookie |
| [`eight_ball(question?: str)`](#eight_ball) | a magic 8-ball answer; the same question gets the same answer all day |
| [`coin(n?: int)`](#coin) | heads or tails, or a list of n flips |
| [`yes_or_no(question?: str)`](#yes_or_no) | yes or no; leans yes on Fridays |
| [`excuse()`](#excuse) | why it doesn't work |
| [`roman(v: int\|str)`](#roman) | an int 1-3999 as a Roman numeral, or a numeral back to an int; same as `n to roman` |

### fortune

`fortune()`: a fortune cookie

```zil
fortune()
# → "An off-by-one error is closer than it appears."
```

See also: [eight_ball](fun.md#eight_ball)

### eight_ball

`eight_ball(question?: str)`: a magic 8-ball answer; the same question gets the same answer all day

```zil
eight_ball("is it Friday?")
# → "My sources say no."
```

See also: [yes_or_no](fun.md#yes_or_no), [fortune](fun.md#fortune)

### coin

`coin(n?: int)`: heads or tails, or a list of n flips

```zil
coin()
# → "heads"
coin(3)
# → ["heads", "tails", "heads"]
```

See also: [rand](math/random.md#rand)

### yes_or_no

`yes_or_no(question?: str)`: yes or no; leans yes on Fridays

```zil
yes_or_no()
# → "yes"
```

See also: [eight_ball](fun.md#eight_ball)

### excuse

`excuse()`: why it doesn't work

```zil
excuse()
# → "I was told the requirements would not change."
```

See also: [fortune](fun.md#fortune)

### roman

`roman(v: int|str)`: an int 1-3999 as a Roman numeral, or a numeral back to an int; same as `n to roman`

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
# → "Yes."
# best of five
coin(5)
# → ["tails", "heads", "heads", "tails", "heads"]
# standup
excuse()
# → "It's a race condition. Probably."
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
