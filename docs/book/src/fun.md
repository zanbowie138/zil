# fun

fortune cookies, a magic 8-ball, coin flips, excuses, Roman numerals; placeholder text, mangled text, dice and zodiac, ASCII toys below

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
| [mangle](fun/mangle.md) | mock case, l33t, uwu, Pig Latin, zalgo, upside-down text, NATO spelling, banners and emoji |
| [games](fun/games.md) | dice notation, the birthday paradox, odds in plain words, Western and Chinese zodiac |
| [toys](fun/toys.md) | ASCII Mandelbrot set, Conway's Game of Life, random mazes |

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
# → "The cache is lying to you."
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
# → "tails"
coin(3)
# → ["heads", "tails", "tails"]
```

See also: [rand](math/random.md#rand)

### yes_or_no

`yes_or_no(question?: str)`: yes or no; leans yes on Fridays

```zil
yes_or_no()
# → "no"
```

See also: [eight_ball](fun.md#eight_ball)

### excuse

`excuse()`: why it doesn't work

```zil
excuse()
# → "The compiler is being weird today."
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
# → ["heads", "heads", "tails", "tails", "tails"]
# standup
excuse()
# → "It was DNS."
# should I ship it?
yes_or_no()
# → "yes"
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
