# numerals

Roman numerals, both ways

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
| [`roman(v)`](#roman) | an int 1-3999 as a Roman numeral, or a numeral back to an int; same as `n to roman` |

### roman

`roman(v)`: an int 1-3999 as a Roman numeral, or a numeral back to an int; same as `n to roman`

```zil
roman(2026)
# → "MMXXVI"
"MCMXCIX".roman
# → 1999
```

## More examples

### roman

```zil
# this year
2026 to roman
# → "MMXXVI"
# Super Bowl math
roman("LX") + 1 to roman
# → "LXI"
```
